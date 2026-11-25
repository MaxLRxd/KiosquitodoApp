//! Migraciones del esquema versionado.
//!
//! Cada migración vive en su propio archivo `.sql`, versionado en git.
//! La 1 es el esquema completo (`001_schema.sql`); las siguientes son
//! incrementales. Se aplican en orden ascendente, una transacción por
//! migración (DDL + registro de versión atómicos), y se registran en
//! `schema_version`. Una BD recién creada ejecuta todas; una existente solo
//! las pendientes.

use rusqlite::Connection;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("error de base de datos: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("migración {version} falló: {source}")]
    Failed { version: i32, source: rusqlite::Error },
    #[error("no se pudo crear el backup previo: {0}")]
    Backup(rusqlite::Error),
}

impl From<MigrationError> for crate::errors::AppError {
    fn from(e: MigrationError) -> Self {
        crate::errors::AppError::Migracion(e.to_string())
    }
}

/// Lista ordenada de migraciones. **No reordenar ni reescribir migraciones ya
/// publicadas**: agregar una nueva al final.
const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("migrations/001_schema.sql")),
];

/// Aplica las migraciones pendientes. Idempotente.
///
/// Debe llamarse con una conexión que ya tenga `PRAGMA foreign_keys=ON`.
/// `db_path` se usa para dejar un backup consistente (VACUUM INTO) antes de
/// aplicar cambios sobre una base que ya tenía datos.
pub fn ejecutar_migraciones(conn: &mut Connection, db_path: &Path) -> Result<(), MigrationError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version     INTEGER PRIMARY KEY,
            aplicado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        ) STRICT;",
    )?;

    // Sin unwrap_or: si falla la lectura, el error sube.
    let version_actual: i32 =
        conn.query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))?;

    let pendientes: Vec<_> = MIGRATIONS.iter().filter(|(v, _)| *v > version_actual).collect();
    if pendientes.is_empty() {
        return Ok(());
    }

    // Backup consistente antes de tocar nada (solo si ya había datos).
    if version_actual > 0 {
        let backup = db_path.with_extension(format!("pre-v{}.bak", version_actual));
        let _ = std::fs::remove_file(&backup);
        conn.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])
            .map_err(MigrationError::Backup)?;
    }

    for (version, sql) in pendientes {
        // Una transacción por migración: DDL + registro de versión son atómicos.
        let tx = conn.transaction()?;
        tx.execute_batch(sql)
            .map_err(|source| MigrationError::Failed { version: *version, source })?;
        tx.execute("INSERT INTO schema_version (version) VALUES (?1)", [*version])?;
        tx.commit()?;
    }
    Ok(())
}

/// Aplicar al abrir CADA conexión.
pub fn configurar_conexion(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")
}

/// Backup diario compacto y consistente, sin bloquear la app.
/// Reemplaza al VACUUM + auto_vacuum del cierre de caja.
pub fn backup_diario(conn: &Connection, destino: &Path) -> Result<(), rusqlite::Error> {
    let _ = std::fs::remove_file(destino); // VACUUM INTO falla si el archivo existe
    conn.execute("VACUUM INTO ?1", [destino.to_string_lossy().as_ref()])?;
    Ok(())
}

// En el caso de uso `close_cash`, llamar a backup_diario a través de un port
// (p. ej. trait BackupPort) en lugar de un #[tauri::command] que toque la Connection.

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
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    #[test]
    fn migrar_bd_vacia_crea_esquema_completo() {
        let mut conn = bd_en_memoria();
        ejecutar_migraciones(&mut conn, Path::new(":memory:")).unwrap();

        let tablas_totales = tablas(&conn);
        for esperada in [
            "schema_version",
            "categorias",
            "productos",
            "cierres_caja",
            "ventas",
            "venta_items",
            "devoluciones",
            "devolucion_items",
            "mp_pagos",
            "configuracion",
            "movimientos_stock",
        ] {
            assert!(
                tablas_totales.contains(&esperada.to_string()),
                "Falta la tabla {esperada}"
            );
        }

        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn migraciones_son_idempotentes() {
        let mut conn = bd_en_memoria();
        ejecutar_migraciones(&mut conn, Path::new(":memory:")).unwrap();
        ejecutar_migraciones(&mut conn, Path::new(":memory:")).unwrap();

        let filas: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(filas, 1, "no debe volver a registrar la migración 1");
    }

    #[test]
    fn esquema_aplica_restricciones() {
        let mut conn = bd_en_memoria();
        ejecutar_migraciones(&mut conn, Path::new(":memory:")).unwrap();

        // medio_pago desconocido viola el CHECK
        let res = conn.execute(
            "INSERT INTO ventas (total, medio_pago, monto_efectivo, monto_mp) \
             VALUES (100, 'tarjeta', 100, 0)",
            [],
        );
        assert!(res.is_err(), "medio_pago desconocido debe violar el CHECK");

        // total debe cuadrar con los montos parciales
        let res = conn.execute(
            "INSERT INTO ventas (total, medio_pago, monto_efectivo, monto_mp) \
             VALUES (100, 'efectivo', 80, 0)",
            [],
        );
        assert!(res.is_err(), "total incoherente debe violar el CHECK");

        // Fechas por defecto en UTC ISO 8601 con 'Z'
        conn.execute(
            "INSERT INTO ventas (total, medio_pago, monto_efectivo, monto_mp) \
             VALUES (100, 'efectivo', 100, 0)",
            [],
        )
        .unwrap();
        let hora: String = conn
            .query_row("SELECT hora FROM ventas", [], |r| r.get(0))
            .unwrap();
        assert!(hora.ends_with('Z'), "hora debe guardarse en UTC (sufijo Z): {hora}");
    }
}
