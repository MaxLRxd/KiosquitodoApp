//! Implementación SQLite del repositorio de cierres de caja.

use crate::domain::cierre::ResumenCierre;
use crate::errors::AppResult;
use crate::infrastructure::db::DbConn;
use crate::ports::cierre_repo::{CierrePersistido, CierreRepo};
use rusqlite::{params, Row};

fn fila_a_cierre(row: &Row) -> rusqlite::Result<CierrePersistido> {
    Ok(CierrePersistido {
        id: row.get(0)?,
        fecha: row.get(1)?,
        resumen: ResumenCierre::nuevo(
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        ),
    })
}

/// Adaptador SQLite que implementa `CierreRepo`.
pub struct CierreSqlite {
    db: DbConn,
}

impl CierreSqlite {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl CierreRepo for CierreSqlite {
    /// Resumen del día por SQL: sumas de ventas + delta MP.
    fn resumen_dia(&self, fecha: &str) -> AppResult<ResumenCierre> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");

        // Totales de ventas del día
        let (total_efectivo, total_mp, total_ventas, cantidad_ventas): (i64, i64, i64, i64) =
            conn.query_row(
                "SELECT \
                   COALESCE(SUM(monto_efectivo), 0), \
                   COALESCE(SUM(monto_mp), 0), \
                   COALESCE(SUM(total), 0), \
                   COUNT(*) \
                 FROM ventas \
                 WHERE date(hora) = ?1",
                params![fecha],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;

        // Margen bruto: (precio_venta - COALESCE(precio_costo, 0)) × cantidad
        let margen_bruto: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM( \
                    (vi.precio_unitario - COALESCE(p.precio_costo, 0)) * vi.cantidad \
                 ), 0) \
                 FROM venta_items vi \
                 JOIN ventas v ON vi.venta_id = v.id \
                 JOIN productos p ON vi.producto_id = p.id \
                 WHERE date(v.hora) = ?1",
                params![fecha],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Delta MP: montos acreditados según API − montos MP registrados por el operador
        let delta_mp: i64 = conn
            .query_row(
                "SELECT ( \
                    SELECT COALESCE(SUM(monto), 0) FROM mp_pagos \
                    WHERE date(fecha_aprobacion) = ?1 \
                 ) - ( \
                    SELECT COALESCE(SUM(monto_mp), 0) FROM ventas \
                    WHERE date(hora) = ?1 AND medio_pago IN ('mercado_pago', 'mixto') \
                 )",
                params![fecha],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(ResumenCierre::nuevo(
            total_efectivo,
            total_mp,
            total_ventas,
            cantidad_ventas,
            margen_bruto,
            delta_mp,
        ))
    }

    fn guardar_cierre(&self, fecha: &str, resumen: &ResumenCierre) -> AppResult<i64> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        conn.execute(
            "INSERT INTO cierres_caja \
             (fecha, total_efectivo, total_mp, total_ventas, cantidad_ventas, margen_bruto, delta_mp) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                fecha,
                resumen.total_efectivo,
                resumen.total_mp,
                resumen.total_ventas,
                resumen.cantidad_ventas,
                resumen.margen_bruto,
                resumen.delta_mp,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn obtener_cierres(&self) -> AppResult<Vec<CierrePersistido>> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        let mut stmt = conn.prepare(
            "SELECT id, fecha, total_efectivo, total_mp, total_ventas, \
                    cantidad_ventas, margen_bruto, delta_mp \
             FROM cierres_caja ORDER BY fecha DESC",
        )?;
        let cierres = stmt
            .query_map([], fila_a_cierre)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(cierres)
    }

    /// `VACUUM` reconstruye el archivo de la BD y recupera espacio.
    /// No corre dentro de una transacción (SQLite lo rechaza).
    fn vacuar(&self) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        conn.execute_batch("VACUUM")?;
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::producto::NuevoProducto;
    use crate::domain::venta::{MedioPago, NuevaVenta, VentaItem};
    use crate::infrastructure::db::repos::pago_sqlite::PagoSqlite;
    use crate::infrastructure::db::repos::producto_sqlite::ProductoSqlite;
    use crate::infrastructure::db::repos::venta_sqlite::VentaSqlite;
    use crate::ports::cierre_repo::CierreRepo;
    use crate::ports::pago_repo::PagoRepo;
    use crate::ports::producto_repo::ProductoRepo;
    use crate::ports::venta_repo::VentaRepo;

    fn setup() -> (CierreSqlite, VentaSqlite, ProductoSqlite, PagoSqlite) {
        let db = DbConn::abrir_en_memoria().unwrap();
        (
            CierreSqlite::new(db.clone()),
            VentaSqlite::new(db.clone()),
            ProductoSqlite::new(db.clone()),
            PagoSqlite::new(db),
        )
    }

    fn crear_producto(p: &ProductoSqlite, nombre: &str, precio: i64) -> crate::domain::producto::Producto {
        p.crear_producto(&NuevoProducto {
            nombre: nombre.to_string(),
            barcode: None,
            precio_venta: precio,
            precio_costo: Some(precio / 2),
            stock_inicial: 100,
            stock_minimo: 0,
            categoria_id: None,
        })
        .unwrap()
    }

    #[test]
    fn resumen_dia_con_ventas_y_pagos_mp() {
        let (cierres, ventas, productos, _pagos) = setup();

        let p1 = crear_producto(&productos, "Coca", 1500);
        let p2 = crear_producto(&productos, "Pepsi", 2000);

        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();

        // Venta en efectivo
        ventas
            .crear_venta_transaccional(&NuevaVenta {
                items: vec![VentaItem { producto_id: p1.id, cantidad: 2, precio_unitario: 1500 }],
                medio_pago: MedioPago::Efectivo,
                monto_efectivo: 3000,
                monto_mp: 0,
                descripcion_mp: None,
            })
            .unwrap();

        // Venta en MP
        ventas
            .crear_venta_transaccional(&NuevaVenta {
                items: vec![VentaItem { producto_id: p2.id, cantidad: 1, precio_unitario: 2000 }],
                medio_pago: MedioPago::MercadoPago,
                monto_efectivo: 0,
                monto_mp: 2000,
                descripcion_mp: None,
            })
            .unwrap();

        let resumen = cierres.resumen_dia(&hoy).unwrap();

        assert_eq!(resumen.total_efectivo, 3000);
        assert_eq!(resumen.total_mp, 2000);
        assert_eq!(resumen.total_ventas, 5000);
        assert_eq!(resumen.cantidad_ventas, 2);

        // Margen: (1500-750)*2 + (2000-1000)*1 = 1500 + 1000 = 2500
        assert_eq!(resumen.margen_bruto, 2500);
    }

    #[test]
    fn resumen_dia_sin_ventas() {
        let (cierres, _, _, _) = setup();
        let resumen = cierres.resumen_dia("2020-01-01").unwrap();
        assert_eq!(resumen.total_ventas, 0);
        assert_eq!(resumen.cantidad_ventas, 0);
        assert_eq!(resumen.delta_mp, 0);
    }

    #[test]
    fn guardar_y_leer_cierres() {
        let (cierres, _, _, _) = setup();
        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        let r = ResumenCierre::nuevo(1000, 2000, 3000, 5, 800, 0);

        let id = cierres.guardar_cierre(&hoy, &r).unwrap();
        assert!(id > 0);

        let lista = cierres.obtener_cierres().unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].fecha, hoy);
        assert_eq!(lista[0].resumen.total_efectivo, 1000);
    }

    #[test]
    fn delta_mp_positivo_en_resumen() {
        let (cierres, ventas, productos, pagos) = setup();
        let p = crear_producto(&productos, "Snack", 1000);
        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();

        // Venta registrada en MP por 1000
        ventas
            .crear_venta_transaccional(&NuevaVenta {
                items: vec![VentaItem { producto_id: p.id, cantidad: 1, precio_unitario: 1000 }],
                medio_pago: MedioPago::MercadoPago,
                monto_efectivo: 0,
                monto_mp: 1000,
                descripcion_mp: None,
            })
            .unwrap();

        // Insertar un pago en MP por 1500 (la API ve 500 más de lo registrado)
        pagos
            .upsert_pago(&crate::domain::pago::PagoMp {
                id: 0,
                mp_id: "delta_test".to_string(),
                fecha_aprobacion: Some(
                    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S.000-03:00").to_string(),
                ),
                monto: 1500,
                descripcion: None,
                venta_id: None,
            })
            .unwrap();

        // delta = Σ pagos API (1500) − Σ ventas MP registradas (1000) = +500
        let resumen = cierres.resumen_dia(&hoy).unwrap();
        assert_eq!(resumen.delta_mp, 500);
    }

    #[test]
    fn vacuar_no_falla_y_mantiene_datos() {
        let (cierres, _, _, _) = setup();
        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        let r = ResumenCierre::nuevo(1000, 0, 1000, 1, 400, 0);
        let id = cierres.guardar_cierre(&hoy, &r).unwrap();

        cierres.vacuar().unwrap();

        let lista = cierres.obtener_cierres().unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].id, id);
        assert_eq!(lista[0].resumen.total_efectivo, 1000);
    }
}