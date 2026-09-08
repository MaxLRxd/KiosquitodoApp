//! Implementación SQLite del repositorio de pagos Mercado Pago.

use crate::config::defaults;
use crate::domain::pago::PagoMp;
use crate::errors::{AppError, AppResult};
use crate::infrastructure::db::DbConn;
use crate::ports::pago_repo::PagoRepo;
use rusqlite::{params, OptionalExtension, Row};

fn fila_a_pago(row: &Row) -> rusqlite::Result<PagoMp> {
    Ok(PagoMp {
        id: row.get(0)?,
        mp_id: row.get(1)?,
        fecha_aprobacion: row.get(2)?,
        monto: row.get(3)?,
        descripcion: row.get(4)?,
        venta_id: row.get(5)?,
    })
}

/// Adaptador SQLite que implementa `PagoRepo`.
pub struct PagoSqlite {
    db: DbConn,
}

impl PagoSqlite {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl PagoRepo for PagoSqlite {
    /// R5: `INSERT OR IGNORE` sobre `mp_id` único — idempotente.
    fn upsert_pago(&self, pago: &PagoMp) -> AppResult<bool> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        let affected = conn.execute(
            "INSERT OR IGNORE INTO mp_pagos (mp_id, fecha_aprobacion, monto, descripcion) \
             VALUES (?1, ?2, ?3, ?4)",
            params![pago.mp_id, pago.fecha_aprobacion, pago.monto, pago.descripcion],
        )?;
        Ok(affected > 0)
    }

    fn pagos_en_rango(&self, desde: &str, hasta: &str) -> AppResult<Vec<PagoMp>> {
        let conn = self.db.conn.lock().expect("lock");
        let mut stmt = conn.prepare(
            "SELECT id, mp_id, fecha_aprobacion, monto, descripcion, venta_id \
             FROM mp_pagos \
             WHERE fecha_aprobacion >= ?1 AND fecha_aprobacion <= ?2 \
             ORDER BY fecha_aprobacion DESC",
        )?;
        let pagos = stmt
            .query_map(params![desde, hasta], fila_a_pago)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pagos)
    }

    fn obtener_pago_por_mp_id(&self, mp_id: &str) -> AppResult<Option<PagoMp>> {
        let conn = self.db.conn.lock().expect("lock");
        let pago = conn
            .query_row(
                "SELECT id, mp_id, fecha_aprobacion, monto, descripcion, venta_id \
                 FROM mp_pagos WHERE mp_id = ?1",
                params![mp_id],
                fila_a_pago,
            )
            .optional()?;
        Ok(pago)
    }

    fn vincular_pago_a_venta(&self, mp_id: &str, venta_id: i64) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("lock");
        let updated = conn.execute(
            "UPDATE mp_pagos SET venta_id = ?1 WHERE mp_id = ?2",
            params![venta_id, mp_id],
        )?;
        if updated == 0 {
            return Err(AppError::from(format!("No se encontró el pago con mp_id: {mp_id}")));
        }
        Ok(())
    }

    /// R7: matching automático — vincula pagos huérfanos con ventas MP de
    /// monto ±100 centavos dentro de ±600 segundos (10 minutos).
    fn aplicar_matching_automatico(&self) -> AppResult<usize> {
        let conn = self.db.conn.lock().expect("lock");
        let actualizados = conn.execute(
            "UPDATE mp_pagos \
             SET venta_id = ( \
                 SELECT v.id FROM ventas v \
                 WHERE v.medio_pago IN ('mercado_pago', 'mixto') \
                   AND ABS(v.monto_mp - mp_pagos.monto) <= ?1 \
                   AND ABS(strftime('%s', v.hora) - strftime('%s', mp_pagos.fecha_aprobacion)) <= ?2 \
                 LIMIT 1 \
             ) \
             WHERE venta_id IS NULL \
               AND ( \
                 SELECT v.id FROM ventas v \
                 WHERE v.medio_pago IN ('mercado_pago', 'mixto') \
                   AND ABS(v.monto_mp - mp_pagos.monto) <= ?1 \
                   AND ABS(strftime('%s', v.hora) - strftime('%s', mp_pagos.fecha_aprobacion)) <= ?2 \
                 LIMIT 1 \
               ) IS NOT NULL",
            params![defaults::MATCHING_MONTO_TOLERANCIA, defaults::MATCHING_VENTANA_SEGUNDOS],
        )?;
        Ok(actualizados)
    }

    fn existe_pago(&self, mp_id: &str) -> AppResult<bool> {
        let conn = self.db.conn.lock().expect("lock");
        let existe: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM mp_pagos WHERE mp_id = ?1",
                params![mp_id],
                |r| r.get(0),
            )?;
        Ok(existe > 0)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::producto::NuevoProducto;
    use crate::domain::venta::{MedioPago, NuevaVenta, Venta, VentaItem};
    use crate::infrastructure::db::repos::producto_sqlite::ProductoSqlite;
    use crate::infrastructure::db::repos::venta_sqlite::VentaSqlite;
    use crate::ports::pago_repo::PagoRepo;
    use crate::ports::producto_repo::ProductoRepo;
    use crate::ports::venta_repo::VentaRepo;

    fn setup_pagos() -> PagoSqlite {
        let db = DbConn::abrir_en_memoria().unwrap();
        PagoSqlite::new(db)
    }

    fn setup_completo() -> (PagoSqlite, VentaSqlite, ProductoSqlite) {
        let db = DbConn::abrir_en_memoria().unwrap();
        (
            PagoSqlite::new(db.clone()),
            VentaSqlite::new(db.clone()),
            ProductoSqlite::new(db),
        )
    }

    fn pago(mp_id: &str, monto: i64) -> PagoMp {
        PagoMp {
            id: 0,
            mp_id: mp_id.to_string(),
            fecha_aprobacion: Some("2026-09-07T10:30:00.000-03:00".to_string()),
            monto,
            descripcion: Some("Test".to_string()),
            venta_id: None,
        }
    }

    #[test]
    fn upsert_idempotente() {
        let pagos = setup_pagos();
        let p = pago("42", 1500);
        assert!(pagos.upsert_pago(&p).unwrap()); // inserta
        assert!(!pagos.upsert_pago(&p).unwrap()); // ignora duplicado
    }

    #[test]
    fn existe_pago() {
        let pagos = setup_pagos();
        assert!(!pagos.existe_pago("42").unwrap());
        pagos.upsert_pago(&pago("42", 1000)).unwrap();
        assert!(pagos.existe_pago("42").unwrap());
    }

    #[test]
    fn obtener_por_mp_id() {
        let pagos = setup_pagos();
        pagos.upsert_pago(&pago("99", 2000)).unwrap();
        let encontrado = pagos.obtener_pago_por_mp_id("99").unwrap().unwrap();
        assert_eq!(encontrado.monto, 2000);
    }

    #[test]
    fn pagos_en_rango_filtro() {
        let pagos = setup_pagos();
        pagos.upsert_pago(&PagoMp {
            id: 0,
            mp_id: "100".to_string(),
            fecha_aprobacion: Some("2026-09-07T10:00:00".to_string()),
            monto: 500,
            descripcion: None,
            venta_id: None,
        })
        .unwrap();
        pagos.upsert_pago(&PagoMp {
            id: 0,
            mp_id: "101".to_string(),
            fecha_aprobacion: Some("2026-09-08T10:00:00".to_string()),
            monto: 800,
            descripcion: None,
            venta_id: None,
        })
        .unwrap();

        let del_07 = pagos
            .pagos_en_rango("2026-09-07T00:00:00", "2026-09-07T23:59:59")
            .unwrap();
        assert_eq!(del_07.len(), 1);
        assert_eq!(del_07[0].mp_id, "100");
    }

    #[test]
    fn vincular_pago_a_venta() {
        let (pagos, ventas, productos) = setup_completo();
        let pid = crear_producto_para_venta(&productos, "3333333333333");
        let venta = crear_venta_mp(&ventas, pid, 1500);

        pagos.upsert_pago(&pago("abc", 1500)).unwrap();
        pagos.vincular_pago_a_venta("abc", venta.id).unwrap();

        let p = pagos.obtener_pago_por_mp_id("abc").unwrap().unwrap();
        assert_eq!(p.venta_id, Some(venta.id));
    }

    fn crear_producto_para_venta(productos: &ProductoSqlite, barcode: &str) -> i64 {
        productos
            .crear_producto(&NuevoProducto {
                nombre: "Test".to_string(),
                barcode: Some(barcode.to_string()),
                precio_venta: 1500,
                precio_costo: None,
                stock_inicial: 100,
                stock_minimo: 0,
                categoria_id: None,
            })
            .unwrap()
            .id
    }

    fn crear_venta_mp(ventas: &VentaSqlite, producto_id: i64, monto: i64) -> Venta {
        ventas
            .crear_venta_transaccional(&NuevaVenta {
                items: vec![VentaItem {
                    producto_id,
                    cantidad: 1,
                    precio_unitario: monto,
                }],
                medio_pago: MedioPago::MercadoPago,
                monto_efectivo: 0,
                monto_mp: monto,
                descripcion_mp: None,
            })
            .unwrap()
    }

    #[test]
    fn matching_automatico_une_por_monto_y_tiempo() {
        let (pagos, ventas, productos) = setup_completo();
        let pid = crear_producto_para_venta(&productos, "1111111111111");
        let venta = crear_venta_mp(&ventas, pid, 1500);

        pagos
            .upsert_pago(&PagoMp {
                id: 0,
                mp_id: "match_01".to_string(),
                fecha_aprobacion: Some(venta.hora.clone()),
                monto: 1500,
                descripcion: None,
                venta_id: None,
            })
            .unwrap();

        let vinculados = pagos.aplicar_matching_automatico().unwrap();
        assert_eq!(vinculados, 1);

        let p = pagos.obtener_pago_por_mp_id("match_01").unwrap().unwrap();
        assert_eq!(p.venta_id, Some(venta.id));
    }

    #[test]
    fn matching_no_vincula_monto_muy_distinto() {
        let (pagos, ventas, productos) = setup_completo();
        let pid = crear_producto_para_venta(&productos, "2222222222222");
        let venta = crear_venta_mp(&ventas, pid, 1500);

        // Pago con monto muy diferente (500 centavos vs 1500)
        pagos
            .upsert_pago(&PagoMp {
                id: 0,
                mp_id: "distant_01".to_string(),
                fecha_aprobacion: Some(venta.hora.clone()),
                monto: 500,
                descripcion: None,
                venta_id: None,
            })
            .unwrap();

        let vinculados = pagos.aplicar_matching_automatico().unwrap();
        assert_eq!(vinculados, 0);

        let p = pagos.obtener_pago_por_mp_id("distant_01").unwrap().unwrap();
        assert_eq!(p.venta_id, None);
    }
}