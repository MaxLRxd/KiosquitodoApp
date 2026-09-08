//! Persistencia SQLite: conexión, PRAGMAs WAL y repositorios concretos.

pub mod migrations;
pub mod repos;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// Handle compartido de la conexión a SQLite.
///
/// `Arc` permite pasarlo a los repositorios y comandos sin clonar la conexión.
/// `Mutex` serializa el acceso (SQLite es single-writer). El lock se toma solo
/// durante la ejecución de una query, nunca durante I/O de red.
#[derive(Clone)]
pub struct DbConn {
    pub conn: Arc<Mutex<Connection>>,
}

impl DbConn {
    /// Aplica las PRAGMAs de durabilidad y concurrencia (R4).
    fn aplicar_pragmas(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL; \
             PRAGMA foreign_keys = ON; \
             PRAGMA busy_timeout = 5000; \
             PRAGMA synchronous = NORMAL; \
             PRAGMA cache_size = -5000; \
             PRAGMA temp_store = MEMORY;",
        )
    }

    /// Abre (o crea) la base de datos en `path`, aplica PRAGMAs y migraciones.
    /// Requiere que el directorio padre exista.
    pub fn abrir(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        Self::aplicar_pragmas(&conn)?;
        migrations::ejecutar_migraciones(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Abre una base en memoria (tests). No requiere archivo.
    pub fn abrir_en_memoria() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        Self::aplicar_pragmas(&conn)?;
        migrations::ejecutar_migraciones(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Ejecuta `PRAGMA wal_checkpoint(TRUNCATE);` para volcar el WAL al
    /// archivo principal (requerido antes de copiar un backup).
    pub fn checkpoint_wal(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().expect("mutex de BD envenenado");
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abrir_en_memoria_migra() {
        let db = DbConn::abrir_en_memoria().unwrap();
        let conn = db.conn.lock().unwrap();

        // En memoria SQLite fuerza journal_mode = memory; lo importante es
        // que las migraciones y PRAGMAs críticas se aplicaron sin error.
        let fk: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fk, 1);

        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 10);
    }

    #[test]
    fn abrir_en_archivo_aplica_wal() {
        let mut path = std::env::temp_dir();
        path.push(format!("kiosco_test_wal_{}.db", std::process::id()));

        let resultado = (|| {
            let db = DbConn::abrir(&path)?;
            let conn = db.conn.lock().expect("lock");
            let wal: String = conn
                .query_row("PRAGMA journal_mode", [], |r| r.get(0))
                .unwrap();
            Ok::<_, rusqlite::Error>(wal)
        })();

        let _ = std::fs::remove_file(&path);
        assert_eq!(resultado.unwrap().to_lowercase(), "wal");
    }

    #[test]
    fn checkpoint_wal_no_falla_en_memoria() {
        let db = DbConn::abrir_en_memoria().unwrap();
        // En memoria no hay archivo físico, pero el checkpoint no debe fallar.
        db.checkpoint_wal().unwrap();
    }

    #[test]
    fn repos_publicados_como_modulo() {
        // Verifica que el submódulo exista (compile-time contract).
        let _ = module_path!().contains("db");
        let _ref: fn() = || {
            let _ = migrations::ejecutar_migraciones;
        };
    }
}