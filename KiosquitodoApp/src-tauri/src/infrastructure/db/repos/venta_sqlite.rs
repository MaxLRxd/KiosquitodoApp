//! Implementación SQLite del repositorio de ventas, ítems y devoluciones.

use crate::domain::venta::{MedioPago, NuevaVenta, Venta, VentaItem};
use crate::errors::{AppError, AppResult};
use crate::infrastructure::db::DbConn;
use crate::ports::venta_repo::VentaRepo;
use rusqlite::{params, Connection, OptionalExtension, Row};

fn medio_pago_a_db(mp: MedioPago) -> &'static str {
    match mp {
        MedioPago::Efectivo => "efectivo",
        MedioPago::MercadoPago => "mercado_pago",
        MedioPago::Mixto => "mixto",
    }
}

fn fila_a_venta(row: &Row) -> rusqlite::Result<Venta> {
    let medio: String = row.get(2)?;
    Ok(Venta {
        id: row.get(0)?,
        items: Vec::new(), // se llenan con obtener_items_venta
        total: row.get(1)?,
        medio_pago: match medio.as_str() {
            "efectivo" => MedioPago::Efectivo,
            "mercado_pago" => MedioPago::MercadoPago,
            "mixto" => MedioPago::Mixto,
            _ => return Err(rusqlite::Error::InvalidColumnIndex(2)),
        },
        monto_efectivo: row.get(3)?,
        monto_mp: row.get(4)?,
        descripcion_mp: row.get(5)?,
        hora: row.get(6)?,
    })
}

/// Adaptador SQLite que implementa `VentaRepo`.
pub struct VentaSqlite {
    db: DbConn,
}

impl VentaSqlite {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    /// Inserta los ítems de una venta dentro de la misma transacción (R9).
    fn insertar_items(tx: &Connection, venta_id: i64, items: &[VentaItem]) -> AppResult<()> {
        let mut stmt = tx.prepare(
            "INSERT INTO venta_items (venta_id, producto_id, cantidad, precio_unitario) \
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        for item in items {
            stmt.execute(params![venta_id, item.producto_id, item.cantidad, item.precio_unitario])?;
        }
        Ok(())
    }

    /// Descuenta el stock de cada producto dentro de la transacción (R9).
    fn descontar_stock(tx: &Connection, items: &[VentaItem]) -> AppResult<()> {
        let mut stmt =
            tx.prepare("UPDATE productos SET stock = stock - ?1 WHERE id = ?2")?;
        for item in items {
            // Verificar stock disponible antes de descontar
            let stock: i64 = tx
                .query_row(
                    "SELECT stock FROM productos WHERE id = ?1",
                    params![item.producto_id],
                    |r| r.get(0),
                )
                .optional()?
                .ok_or_else(|| AppError::from(format!("Producto {} no existe", item.producto_id)))?;

            if stock < item.cantidad {
                return Err(AppError::from(format!(
                    "Stock insuficiente del producto {} (disponible: {stock})",
                    item.producto_id
                )));
            }
            stmt.execute(params![item.cantidad, item.producto_id])?;

            // Registrar el movimiento de stock de tipo 'venta'
            tx.execute(
                "INSERT INTO movimientos_stock (producto_id, stock_anterior, stock_nuevo, motivo) \
                 VALUES (?1, ?2, ?3, 'venta')",
                params![item.producto_id, stock, stock - item.cantidad],
            )?;
        }
        Ok(())
    }
}

impl VentaRepo for VentaSqlite {
    fn crear_venta_transaccional(&self, venta: &NuevaVenta) -> AppResult<Venta> {
        let mut conn = self.db.conn.lock().expect("mutex de BD envenenado");
        let tx = conn.transaction()?;

        // Insertar cabecera
        let venta_id = {
            tx.execute(
                "INSERT INTO ventas (total, medio_pago, monto_efectivo, monto_mp, descripcion_mp) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    venta.items.iter().map(|i| i.subtotal()).sum::<i64>(),
                    medio_pago_a_db(venta.medio_pago),
                    venta.monto_efectivo,
                    venta.monto_mp,
                    venta.descripcion_mp,
                ],
            )?;
            tx.last_insert_rowid()
        };

        // Insertar ítems y descontar stock (R9)
        Self::insertar_items(&tx, venta_id, &venta.items)?;
        Self::descontar_stock(&tx, &venta.items)?;

        // Capturar la hora asignada por la BD antes del commit
        let hora: String = tx
            .query_row("SELECT hora FROM ventas WHERE id = ?1", params![venta_id], |r| r.get(0))?;

        tx.commit()?;

        Ok(Venta {
            id: venta_id,
            items: venta.items.clone(),
            total: venta.items.iter().map(|i| i.subtotal()).sum(),
            medio_pago: venta.medio_pago,
            monto_efectivo: venta.monto_efectivo,
            monto_mp: venta.monto_mp,
            descripcion_mp: venta.descripcion_mp.clone(),
            hora,
        })
    }

    fn obtener_venta(&self, id: i64) -> AppResult<Option<Venta>> {
        let mut venta = {
            let conn = self.db.conn.lock().expect("lock");
            conn.query_row(
                "SELECT id, total, medio_pago, monto_efectivo, monto_mp, descripcion_mp, hora \
                 FROM ventas WHERE id = ?1",
                params![id],
                fila_a_venta,
            )
            .optional()?
        };

        if let Some(v) = venta.as_mut() {
            v.items = self.obtener_items_venta(id)?;
        }
        Ok(venta)
    }

    fn listar_ventas_dia(&self, fecha: &str) -> AppResult<Vec<Venta>> {
        let conn = self.db.conn.lock().expect("lock");
        let mut stmt = conn.prepare(
            "SELECT id, total, medio_pago, monto_efectivo, monto_mp, descripcion_mp, hora \
             FROM ventas WHERE date(hora) = ?1 ORDER BY hora DESC",
        )?;
        let mut ventas = stmt
            .query_map(params![fecha], fila_a_venta)?
            .collect::<Result<Vec<_>, _>>()?;

        drop(stmt);
        drop(conn);

        // Cargar ítems para cada venta
        for v in ventas.iter_mut() {
            v.items = self.obtener_items_venta(v.id)?;
        }
        Ok(ventas)
    }

    fn obtener_items_venta(&self, venta_id: i64) -> AppResult<Vec<VentaItem>> {
        let conn = self.db.conn.lock().expect("lock");
        let mut stmt = conn.prepare(
            "SELECT producto_id, cantidad, precio_unitario \
             FROM venta_items WHERE venta_id = ?1 ORDER BY id",
        )?;
        let items = stmt
            .query_map(params![venta_id], |row| {
                Ok(VentaItem {
                    producto_id: row.get(0)?,
                    cantidad: row.get(1)?,
                    precio_unitario: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    fn crear_devolucion(
        &self,
        venta_id: i64,
        items: &[VentaItem],
        motivo: Option<&str>,
    ) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("lock");
        let tx = conn.unchecked_transaction()?;

        // Insertar cabecera de devolución
        tx.execute(
            "INSERT INTO devoluciones (venta_id, motivo) VALUES (?1, ?2)",
            params![venta_id, motivo],
        )?;
        let devolucion_id = tx.last_insert_rowid();

        // Insertar ítems y devolver stock
        {
            let mut dev_items = tx.prepare(
                "INSERT INTO devolucion_items (devolucion_id, producto_id, cantidad, monto_devuelto) \
                 VALUES (?1, ?2, ?3, ?4)",
            )?;
            let mut upd = tx.prepare(
                "UPDATE productos SET stock = stock + ?1 WHERE id = ?2",
            )?;
            let mut mov = tx.prepare(
                "INSERT INTO movimientos_stock (producto_id, stock_anterior, stock_nuevo, motivo) \
                 VALUES (?1, ?2, ?3, 'devolucion')",
            )?;

            for item in items {
                let monto = item.cantidad.saturating_mul(item.precio_unitario);
                dev_items.execute(params![devolucion_id, item.producto_id, item.cantidad, monto])?;

                let stock_actual: i64 = tx
                    .query_row(
                        "SELECT stock FROM productos WHERE id = ?1",
                        params![item.producto_id],
                        |r| r.get(0),
                    )
                    .optional()?
                    .unwrap_or(0);
                upd.execute(params![item.cantidad, item.producto_id])?;
                mov.execute(
                    params![item.producto_id, stock_actual, stock_actual + item.cantidad],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::producto::NuevoProducto;
    use crate::infrastructure::db::repos::producto_sqlite::ProductoSqlite;
    use crate::ports::producto_repo::ProductoRepo;
    use crate::ports::venta_repo::VentaRepo;

    fn setup() -> (VentaSqlite, ProductoSqlite) {
        let db = DbConn::abrir_en_memoria().unwrap();
        let ventas = VentaSqlite::new(db.clone());
        let productos = ProductoSqlite::new(db);
        (ventas, productos)
    }

    fn crear_producto(p: &ProductoSqlite, id_seed: i64) -> crate::domain::producto::Producto {
        p.crear_producto(&NuevoProducto {
            nombre: format!("Producto {id_seed}"),
            barcode: Some(format!("7790000000{id_seed:0>3}")),
            precio_venta: 1500,
            precio_costo: Some(800),
            stock_inicial: 10,
            stock_minimo: 2,
            categoria_id: None,
        })
        .unwrap()
    }

    fn nueva_venta(items: Vec<VentaItem>) -> NuevaVenta {
        let total: i64 = items.iter().map(|i| i.subtotal()).sum();
        NuevaVenta {
            items,
            medio_pago: MedioPago::Efectivo,
            monto_efectivo: total,
            monto_mp: 0,
            descripcion_mp: None,
        }
    }

    #[test]
    fn crear_venta_descuenta_stock_en_transaccion() {
        let (ventas, productos) = setup();
        let p1 = crear_producto(&productos, 1);
        let p2 = crear_producto(&productos, 2);

        let items = vec![
            VentaItem { producto_id: p1.id, cantidad: 3, precio_unitario: 1000 },
            VentaItem { producto_id: p2.id, cantidad: 1, precio_unitario: 1500 },
        ];

        let venta = ventas.crear_venta_transaccional(&nueva_venta(items)).unwrap();
        assert!(venta.id > 0);
        assert_eq!(venta.total, 4500);

        // Stock descontado
        let p1_act = productos.buscar_por_id(p1.id).unwrap().unwrap();
        let p2_act = productos.buscar_por_id(p2.id).unwrap().unwrap();
        assert_eq!(p1_act.stock, 7); // 10 - 3
        assert_eq!(p2_act.stock, 9); // 10 - 1
    }

    #[test]
    fn crear_venta_con_stock_insuficiente_rollback() {
        let (ventas, productos) = setup();
        let p1 = crear_producto(&productos, 1); // stock 10

        let items = vec![VentaItem { producto_id: p1.id, cantidad: 999, precio_unitario: 100 }];
        let res = ventas.crear_venta_transaccional(&nueva_venta(items));
        assert!(res.is_err());

        // El stock NO cambió (rollback)
        let p1_act = productos.buscar_por_id(p1.id).unwrap().unwrap();
        assert_eq!(p1_act.stock, 10);

        // No se creó ninguna venta
        assert!(ventas.obtener_venta(1).unwrap().is_none());
    }

    #[test]
    fn obtener_venta_incluye_items() {
        let (ventas, productos) = setup();
        let p1 = crear_producto(&productos, 1);

        let items = vec![VentaItem { producto_id: p1.id, cantidad: 2, precio_unitario: 750 }];
        let venta = ventas.crear_venta_transaccional(&nueva_venta(items)).unwrap();

        let obtenida = ventas.obtener_venta(venta.id).unwrap().unwrap();
        assert_eq!(obtenida.items.len(), 1);
        assert_eq!(obtenida.items[0].cantidad, 2);
        assert_eq!(obtenida.items[0].precio_unitario, 750);
    }

    #[test]
    fn listar_ventas_dia() {
        let (ventas, productos) = setup();
        let p1 = crear_producto(&productos, 1);

        for _ in 0..2 {
            let _ = ventas
                .crear_venta_transaccional(
                    &nueva_venta(vec![VentaItem { producto_id: p1.id, cantidad: 1, precio_unitario: 500 }]),
                )
                .unwrap();
        }

        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        let del_dia = ventas.listar_ventas_dia(&hoy).unwrap();
        assert_eq!(del_dia.len(), 2);
    }

    #[test]
    fn devolucion_recupera_stock() {
        let (ventas, productos) = setup();
        let p1 = crear_producto(&productos, 1); // stock 10

        let items = vec![VentaItem { producto_id: p1.id, cantidad: 4, precio_unitario: 1000 }];
        let venta = ventas.crear_venta_transaccional(&nueva_venta(items)).unwrap();

        // Devolver 2 unidades
        let dev_items = vec![VentaItem { producto_id: p1.id, cantidad: 2, precio_unitario: 1000 }];
        ventas.crear_devolucion(venta.id, &dev_items, Some("no le gustó")).unwrap();

        let p1_act = productos.buscar_por_id(p1.id).unwrap().unwrap();
        assert_eq!(p1_act.stock, 8); // 10 - 4 + 2
    }

    #[test]
    fn devolucion_registra_movimiento() {
        let (ventas, productos) = setup();
        let p1 = crear_producto(&productos, 1);

        let venta = ventas
            .crear_venta_transaccional(
                &nueva_venta(vec![VentaItem { producto_id: p1.id, cantidad: 2, precio_unitario: 100 }]),
            )
            .unwrap();

        ventas
            .crear_devolucion(
                venta.id,
                &[VentaItem { producto_id: p1.id, cantidad: 1, precio_unitario: 100 }],
                None,
            )
            .unwrap();

        let conn = ventas.db.conn.lock().unwrap();
        let motivo: String = conn
            .query_row(
                "SELECT motivo FROM movimientos_stock WHERE producto_id = ?1 ORDER BY id DESC",
                params![p1.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(motivo, "devolucion");
    }
}