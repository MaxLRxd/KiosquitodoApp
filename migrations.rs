// src-tauri/src/infrastructure/db/migrations.rs
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

// Cada migración vive en su propio archivo .sql, versionado en git.
// La 1 es el esquema completo (001_schema.sql); las siguientes son incrementales.
const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("migrations/001_schema.sql")),
    // (2, include_str!("migrations/002_xxx.sql")),
];

/// Debe llamarse con una conexión que ya tenga `PRAGMA foreign_keys=ON`.
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
