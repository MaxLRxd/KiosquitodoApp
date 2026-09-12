//! Backup de la base de datos y limpieza de copias viejas.
//!
//! Los backups se generan con `VACUUM INTO`, que produce una copia
//! consistente y compacta aunque la BD esté en modo WAL.

use crate::errors::AppResult;
use crate::infrastructure::db::DbConn;
use std::path::{Path, PathBuf};

fn path_sql_literal(path: &str) -> String {
    path.replace('\'', "''")
}

/// Crea un backup en `backup_dir` con sello de tiempo. Devuelve la ruta.
pub fn crear_backup(db: &DbConn, backup_dir: &Path) -> AppResult<PathBuf> {
    std::fs::create_dir_all(backup_dir)?;

    let sello = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let destino = backup_dir.join(format!("kiosco_{sello}.db"));
    let sql = format!("VACUUM INTO '{}'", path_sql_literal(&destino.to_string_lossy()));

    let conn = db.conn.lock().expect("mutex de BD envenenado");
    conn.execute_batch(&sql)?;
    Ok(destino)
}

/// Elimina los backups más antiguos que `retencion_dias`. Devuelve cuántos
/// se borraron.
pub fn limpiar_backups_viejos(backup_dir: &Path, retencion_dias: u32) -> AppResult<usize> {
    let retension = std::time::Duration::from_secs(retencion_dias as u64 * 86400);
    let ahora = std::time::SystemTime::now();

    let mut eliminados = 0;
    if !backup_dir.exists() {
        return Ok(0);
    }

    for entrada in std::fs::read_dir(backup_dir)? {
        let entrada = entrada?;
        let metadata = entrada.metadata()?;
        if !metadata.is_file() {
            continue;
        }
        let edad = ahora
            .duration_since(metadata.modified().unwrap_or(ahora))
            .unwrap_or_default();
        if edad > retension {
            std::fs::remove_file(entrada.path())?;
            eliminados += 1;
        }
    }
    Ok(eliminados)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::defaults;

    #[test]
    fn literales_de_ruta_se_sanean() {
        assert_eq!(path_sql_literal("C:/a/b.db"), "C:/a/b.db");
        assert_eq!(path_sql_literal("C:/it's/b.db"), "C:/it''s/b.db");
    }

    #[test]
    fn limpiar_sin_carpeta_no_falla() {
        assert_eq!(limpiar_backups_viejos(Path::new("no_existe"), defaults::BACKUP_RETENCION_DIAS).unwrap(), 0);
    }

    #[test]
    fn crear_backup_genera_archivo_consistente() {
        let dir = std::env::temp_dir().join(format!("kiosco_bk_{}", std::process::id()));
        let db = DbConn::abrir_en_memoria().unwrap();

        // Inserta un producto para que el backup tenga contenido.
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, barcode, precio_venta, precio_costo, stock, stock_minimo, activo) \
             VALUES ('Test', NULL, 1000, 500, 10, 0, 1)",
            [],
        )
        .unwrap();
        drop(conn);

        let ruta = crear_backup(&db, &dir).unwrap();
        assert!(ruta.exists());

        // El archivo es una BD SQLite válida con los datos.
        let conn = rusqlite::Connection::open(&ruta).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM productos", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);

        std::fs::remove_dir_all(&dir).ok();
    }
}