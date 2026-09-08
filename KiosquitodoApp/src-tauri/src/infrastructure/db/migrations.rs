//! Migraciones del esquema versionado.
//!
//! Cada migración es un par `(version, SQL)`. Se aplican en orden ascendente
//! y se registran en la tabla `schema_version`. Una BD recién creada ejecuta
//! todas; una existente solo las pendientes.

use rusqlite::Connection;

/// Lista ordenada de migraciones. La versión es el número de migración.
/// **No reordenar ni reescribir migraciones ya publicadas**: agregar una nueva
/// al final.
const MIGRATIONS: &[(i32, &str)] = &[
    (
        1,
        r#"
        CREATE TABLE IF NOT EXISTS productos (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre      TEXT NOT NULL,
            barcode     TEXT UNIQUE,
            precio_venta  INTEGER NOT NULL,  -- centavos
            precio_costo  INTEGER,           -- centavos
            stock         INTEGER NOT NULL DEFAULT 0,
            stock_minimo  INTEGER NOT NULL DEFAULT 5,
            activo        INTEGER NOT NULL DEFAULT 1,
            creado_en     TEXT DEFAULT (datetime('now', 'localtime')),
            actualizado_en TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_productos_barcode ON productos(barcode);
        CREATE INDEX IF NOT EXISTS idx_productos_nombre  ON productos(nombre);
        "#,
    ),
    (
        2,
        r#"
        CREATE TABLE IF NOT EXISTS ventas (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            total         INTEGER NOT NULL,  -- centavos
            medio_pago    TEXT NOT NULL CHECK(medio_pago IN ('efectivo', 'mercado_pago', 'mixto')),
            monto_efectivo   INTEGER DEFAULT 0,  -- centavos
            monto_mp         INTEGER DEFAULT 0,  -- centavos
            descripcion_mp   TEXT,
            hora          TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            cerrada       INTEGER NOT NULL DEFAULT 0
        );
        "#,
    ),
    (
        3,
        r#"
        CREATE TABLE IF NOT EXISTS venta_items (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            venta_id    INTEGER NOT NULL REFERENCES ventas(id),
            producto_id INTEGER NOT NULL REFERENCES productos(id),
            cantidad    INTEGER NOT NULL,
            precio_unitario INTEGER NOT NULL  -- centavos
        );
        CREATE INDEX IF NOT EXISTS idx_venta_items_venta ON venta_items(venta_id);
        "#,
    ),
    (
        4,
        r#"
        CREATE TABLE IF NOT EXISTS mp_pagos (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            mp_id            TEXT UNIQUE NOT NULL,
            fecha_aprobacion TEXT,
            monto            INTEGER NOT NULL,  -- centavos
            descripcion      TEXT,
            venta_id         INTEGER REFERENCES ventas(id),
            importado_en     TEXT DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_mp_pagos_fecha ON mp_pagos(fecha_aprobacion);
        CREATE INDEX IF NOT EXISTS idx_mp_pagos_monto ON mp_pagos(monto);
        "#,
    ),
    (
        5,
        r#"
        CREATE TABLE IF NOT EXISTS configuracion (
            clave TEXT PRIMARY KEY,
            valor TEXT
        );
        "#,
    ),
    (
        6,
        r#"
        CREATE TABLE IF NOT EXISTS cierres_caja (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha            TEXT NOT NULL,
            total_efectivo   INTEGER DEFAULT 0,  -- centavos
            total_mp         INTEGER DEFAULT 0,  -- centavos
            total_ventas     INTEGER DEFAULT 0,
            cantidad_ventas  INTEGER DEFAULT 0,
            margen_bruto     INTEGER DEFAULT 0,  -- centavos
            delta_mp         INTEGER DEFAULT 0,  -- centavos
            creado_en        TEXT DEFAULT (datetime('now', 'localtime'))
        );
        "#,
    ),
    (
        7,
        r#"
        CREATE TABLE IF NOT EXISTS categorias (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre  TEXT NOT NULL UNIQUE,
            color   TEXT   -- opcional: código hex para identificación visual
        );
        "#,
    ),
    (
        8,
        r#"
        ALTER TABLE productos ADD COLUMN categoria_id INTEGER REFERENCES categorias(id);
        CREATE INDEX IF NOT EXISTS idx_productos_categoria ON productos(categoria_id);
        "#,
    ),
    (
        9,
        r#"
        CREATE TABLE IF NOT EXISTS devoluciones (
            id       INTEGER PRIMARY KEY AUTOINCREMENT,
            venta_id INTEGER NOT NULL REFERENCES ventas(id),
            fecha    TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            motivo   TEXT,
            creado_en TEXT DEFAULT (datetime('now', 'localtime'))
        );
        CREATE TABLE IF NOT EXISTS devolucion_items (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            devolucion_id   INTEGER NOT NULL REFERENCES devoluciones(id),
            producto_id     INTEGER NOT NULL REFERENCES productos(id),
            cantidad        INTEGER NOT NULL,
            monto_devuelto  INTEGER NOT NULL  -- centavos
        );
        "#,
    ),
    (
        10,
        r#"
        CREATE TABLE IF NOT EXISTS movimientos_stock (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            producto_id  INTEGER NOT NULL REFERENCES productos(id),
            stock_anterior INTEGER NOT NULL,
            stock_nuevo  INTEGER NOT NULL,
            motivo       TEXT NOT NULL CHECK(motivo IN ('recuento_fisico', 'rotura', 'vencimiento', 'error_carga', 'otro', 'venta', 'devolucion')),
            operador     TEXT,
            creado_en    TEXT DEFAULT (datetime('now', 'localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_movimientos_producto ON movimientos_stock(producto_id);
        "#,
    ),
];

/// Aplica las migraciones pendientes sobre la conexión. Idempotente.
pub fn ejecutar_migraciones(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version    INTEGER PRIMARY KEY,
            aplicado_en TEXT DEFAULT (datetime('now', 'localtime'))
        );",
    )?;

    // La migración 8 altera productos con una columna que en la 1 ya no existe.
    // Se guarda el max version antes de aplicar para decidir correctamente.
    let version_actual: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )?;

    for (version, sql) in MIGRATIONS {
        if *version > version_actual {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [*version],
            )?;
        }
    }
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn bd_en_memoria() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn tablas(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master \
                 WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
                 ORDER BY name",
            )
            .unwrap();
        let names = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        names
    }

    #[test]
    fn migrar_bd_vacia_crea_esquema_completo() {
        let conn = bd_en_memoria();
        ejecutar_migraciones(&conn).unwrap();

        let tablas_totales = tablas(&conn);
        for esperada in [
            "schema_version",
            "productos",
            "ventas",
            "venta_items",
            "mp_pagos",
            "configuracion",
            "cierres_caja",
            "categorias",
            "devoluciones",
            "devolucion_items",
            "movimientos_stock",
        ] {
            assert!(
                tablas_totales.contains(&esperada.to_string()),
                "Falta la tabla {esperada}"
            );
        }
    }

    #[test]
    fn migraciones_son_idempotentes() {
        let conn = bd_en_memoria();
        ejecutar_migraciones(&conn).unwrap();
        ejecutar_migraciones(&conn).unwrap();

        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 10);
    }

    #[test]
    fn bd_reciente_solo_aplica_pendientes() {
        let conn = bd_en_memoria();
        // BD al día (versión 10)
        ejecutar_migraciones(&conn).unwrap();

        // Simular un backup que se quedó en la versión 8: faltan las tablas
        // creadas por las migraciones 9 y 10 y sus registros de versión.
        conn.execute_batch(
            "DELETE FROM schema_version WHERE version IN (9, 10); \
             DROP TABLE movimientos_stock; \
             DROP TABLE devolucion_items; \
             DROP TABLE devoluciones;",
        )
        .unwrap();

        ejecutar_migraciones(&conn).unwrap();

        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 10);

        // Las tablas de las migraciones pendientes fueron reconstruidas
        assert!(tablas(&conn).contains(&"devoluciones".to_string()));
        assert!(tablas(&conn).contains(&"devolucion_items".to_string()));
        assert!(tablas(&conn).contains(&"movimientos_stock".to_string()));
    }

    #[test]
    fn productos_tiene_categoria_y_restricciones() {
        let conn = bd_en_memoria();
        ejecutar_migraciones(&conn).unwrap();

        // tipo de medio_pago con CHECK
        let res = conn.execute(
            "INSERT INTO ventas (total, medio_pago, monto_efectivo, monto_mp) \
             VALUES (100, 'tarjeta', 100, 0)",
            [],
        );
        assert!(res.is_err(), "medio_pago desconocido debe violar el CHECK");

        // categoria_id es columna válida de productos
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta, stock) VALUES ('Prueba', 100, 5)",
            [],
        )
        .unwrap();
    }
}