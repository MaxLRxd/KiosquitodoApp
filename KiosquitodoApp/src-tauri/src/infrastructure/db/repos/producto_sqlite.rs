//! Implementación SQLite del repositorio de productos.

use crate::domain::producto::{NuevoProducto, Producto};
use crate::errors::{AppError, AppResult};
use crate::infrastructure::db::DbConn;
use crate::ports::producto_repo::ProductoRepo;
use rusqlite::{params, OptionalExtension, Row};

/// Convierte una fila SQLite a `Producto`.
fn fila_a_producto(row: &Row) -> rusqlite::Result<Producto> {
    Ok(Producto {
        id: row.get(0)?,
        nombre: row.get(1)?,
        barcode: row.get(2)?,
        precio_venta: row.get(3)?,
        precio_costo: row.get(4)?,
        stock: row.get(5)?,
        stock_minimo: row.get(6)?,
        activo: row.get::<_, i64>(7)? != 0,
        categoria_id: row.get(8)?,
    })
}

const COLUMNAS_PRODUCTO: &str = "id, nombre, barcode, precio_venta, precio_costo, stock, stock_minimo, activo, categoria_id";

/// Adaptador SQLite que implementa `ProductoRepo`.
pub struct ProductoSqlite {
    db: DbConn,
}

impl ProductoSqlite {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl ProductoRepo for ProductoSqlite {
    fn buscar_por_barcode(&self, barcode: &str) -> AppResult<Option<Producto>> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        let resultado = conn
            .query_row(
                &format!(
                    "SELECT {COLUMNAS_PRODUCTO} FROM productos \
                     WHERE barcode = ?1 AND activo = 1"
                ),
                params![barcode],
                fila_a_producto,
            )
            .optional()?;
        Ok(resultado)
    }

    fn buscar_por_id(&self, id: i64) -> AppResult<Option<Producto>> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        let resultado = conn
            .query_row(
                &format!("SELECT {COLUMNAS_PRODUCTO} FROM productos WHERE id = ?1"),
                params![id],
                fila_a_producto,
            )
            .optional()?;
        Ok(resultado)
    }

    fn listar_productos(
        &self,
        filtro: &str,
        categoria_id: Option<i64>,
        limite: i64,
        offset: i64,
    ) -> AppResult<Vec<Producto>> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");

        let filtro_like = format!("%{}%", filtro.trim());
        // ?2 IS NULL actúa como "todas las categorías" cuando llega None,
        // manteniendo el conteo de placeholders estable (4 siempre).
        let sql = format!(
            "SELECT {COLUMNAS_PRODUCTO} FROM productos \
             WHERE activo = 1 \
               AND (nombre LIKE ?1 OR COALESCE(barcode, '') LIKE ?1) \
               AND (?2 IS NULL OR categoria_id = ?2) \
             ORDER BY nombre LIMIT ?3 OFFSET ?4"
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            params![filtro_like, categoria_id, limite, offset],
            fila_a_producto,
        )?;
        let productos = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(productos)
    }

    fn crear_producto(&self, nuevo: &NuevoProducto) -> AppResult<Producto> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");

        // Barcode único: si ya existe uno activo/inactivo con ese código, falla.
        conn.execute(
            "INSERT INTO productos \
             (nombre, barcode, precio_venta, precio_costo, stock, stock_minimo, categoria_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                nuevo.nombre.trim(),
                nuevo.barcode.clone(),
                nuevo.precio_venta,
                nuevo.precio_costo,
                nuevo.stock_inicial,
                nuevo.stock_minimo,
                nuevo.categoria_id,
            ],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::from(
                    "Ya existe un producto con ese código de barras".to_string(),
                )
            }
            otros => AppError::from(otros),
        })?;

        let id = conn.last_insert_rowid();
        drop(conn);

        Ok(Producto {
            id,
            nombre: nuevo.nombre.trim().to_string(),
            barcode: nuevo.barcode.clone(),
            precio_venta: nuevo.precio_venta,
            precio_costo: nuevo.precio_costo,
            stock: nuevo.stock_inicial,
            stock_minimo: nuevo.stock_minimo,
            activo: true,
            categoria_id: nuevo.categoria_id,
        })
    }

    fn actualizar_producto(&self, producto: &Producto) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        conn.execute(
            "UPDATE productos SET \
                nombre = ?1, barcode = ?2, precio_venta = ?3, precio_costo = ?4, \
                stock_minimo = ?5, categoria_id = ?6, actualizado_en = datetime('now', 'localtime') \
             WHERE id = ?7",
            params![
                producto.nombre.trim(),
                producto.barcode,
                producto.precio_venta,
                producto.precio_costo,
                producto.stock_minimo,
                producto.categoria_id,
                producto.id,
            ],
        )?;
        Ok(())
    }

    fn eliminar_producto(&self, id: i64) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        conn.execute(
            "UPDATE productos SET activo = 0, actualizado_en = datetime('now', 'localtime') \
             WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn ajustar_stock(
        &self,
        id: i64,
        stock_nuevo: i64,
        motivo: &str,
        operador: Option<&str>,
    ) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        let tx = conn.unchecked_transaction()?;

        // Leer stock anterior antes de actualizar
        let stock_anterior: i64 = tx
            .query_row(
                "SELECT stock FROM productos WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| AppError::from(format!("Producto {id} no existe")))?;

        tx.execute(
            "UPDATE productos SET stock = ?1 WHERE id = ?2",
            params![stock_nuevo, id],
        )?;

        tx.execute(
            "INSERT INTO movimientos_stock (producto_id, stock_anterior, stock_nuevo, motivo, operador) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, stock_anterior, stock_nuevo, motivo, operador],
        )?;

        tx.commit()?;
        Ok(())
    }

    fn costos_de_productos(&self, ids: &[i64]) -> AppResult<Vec<(i64, i64)>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");

        // Construir placeholders (?1, ?2, ...)
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "SELECT id, precio_costo FROM productos WHERE id IN ({})",
            placeholders.join(",")
        );

        // Pasar los ids como slice de params
        let params_ref: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_ref.iter()), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;
        let costos = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(costos)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::producto_repo::ProductoRepo;

    fn repo() -> ProductoSqlite {
        ProductoSqlite::new(DbConn::abrir_en_memoria().unwrap())
    }

    fn nuevo(nombre: &str, barcode: &str, precio: i64) -> NuevoProducto {
        NuevoProducto {
            nombre: nombre.to_string(),
            barcode: if barcode.is_empty() {
                None
            } else {
                Some(barcode.to_string())
            },
            precio_venta: precio,
            precio_costo: Some(precio / 2),
            stock_inicial: 10,
            stock_minimo: 2,
            categoria_id: None,
        }
    }

    #[test]
    fn crear_y_buscar_por_barcode() {
        let r = repo();
        let creado = r.crear_producto(&nuevo("Coca 500ml", "7790001234567", 1800)).unwrap();
        assert!(creado.id > 0);

        let encontrado = r.buscar_por_barcode("7790001234567").unwrap().unwrap();
        assert_eq!(encontrado.id, creado.id);
        assert_eq!(encontrado.nombre, "Coca 500ml");
        assert_eq!(encontrado.precio_venta, 1800);
        assert_eq!(encontrado.stock, 10);
    }

    #[test]
    fn buscar_barcode_inexistente_devuelve_none() {
        let r = repo();
        assert!(r.buscar_por_barcode("9999999999999").unwrap().is_none());
    }

    #[test]
    fn buscar_por_id() {
        let r = repo();
        let creado = r.crear_producto(&nuevo("Agua", "7790001234567", 1200)).unwrap();
        let encontrado = r.buscar_por_id(creado.id).unwrap().unwrap();
        assert_eq!(encontrado.nombre, "Agua");
    }

    #[test]
    fn barcode_unico_rechaza_duplicado() {
        let r = repo();
        r.crear_producto(&nuevo("A", "7790001234567", 100)).unwrap();
        let res = r.crear_producto(&nuevo("B", "7790001234567", 200));
        assert!(res.is_err());
    }

    #[test]
    fn listar_paginado_con_filtro() {
        let r = repo();
        for i in 0..5 {
            let nombre = format!("Producto {i:02}");
            // barcode único por producto
            let code = format!("7790000000{i}0{}", i);
            let _ = r.crear_producto(&nuevo(&nombre, &code, 100 + i)).unwrap();
        }

        // Filtro por nombre
        let result = r.listar_productos("Producto", None, 10, 0).unwrap();
        assert_eq!(result.len(), 5);

        // Paginación: 2 por página
        let p1 = r.listar_productos("Producto", None, 2, 0).unwrap();
        let p2 = r.listar_productos("Producto", None, 2, 2).unwrap();
        assert_eq!(p1.len(), 2);
        assert_eq!(p2.len(), 2);
        assert_ne!(p1[0].id, p2[0].id);
    }

    #[test]
    fn eliminar_baja_logica() {
        let r = repo();
        let creado = r.crear_producto(&nuevo("Snack", "7790001234567", 300)).unwrap();

        r.eliminar_producto(creado.id).unwrap();

        // Ya no aparece por barcode (activo = 0)
        assert!(r.buscar_por_barcode("7790001234567").unwrap().is_none());
        // Aún se puede leer por id (activo = 0 lo lee igual)
        let p = r.buscar_por_id(creado.id).unwrap().unwrap();
        assert!(!p.activo);
    }

    #[test]
    fn ajustar_stock_registra_movimiento() {
        let r = repo();
        let creado = r.crear_producto(&nuevo("Rotura", "7790001234567", 100)).unwrap();

        r.ajustar_stock(creado.id, 6, "rotura", Some("operador")).unwrap();

        let p = r.buscar_por_id(creado.id).unwrap().unwrap();
        assert_eq!(p.stock, 6);

        // Verificar el movimiento registrado
        let conn = r.db.conn.lock().unwrap();
        let (anterior, nuevo_s) = conn
            .query_row(
                "SELECT stock_anterior, stock_nuevo FROM movimientos_stock \
                 WHERE producto_id = ?1",
                params![creado.id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .unwrap();
        assert_eq!(anterior, 10);
        assert_eq!(nuevo_s, 6);
    }

    #[test]
    fn costos_de_productos_devuelve_pares() {
        let r = repo();
        let p1 = r.crear_producto(&nuevo("A", "7790001234511", 1000)).unwrap();
        let p2 = r.crear_producto(&nuevo("B", "7790001234522", 2000)).unwrap();

        let costos = r.costos_de_productos(&[p1.id, p2.id]).unwrap();
        assert_eq!(costos.len(), 2);
        assert!(costos.contains(&(p1.id, 500)));
        assert!(costos.contains(&(p2.id, 1000)));
    }
}