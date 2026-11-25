//! Implementación SQLite del repositorio de configuración clave/valor.

use crate::errors::AppResult;
use crate::infrastructure::db::DbConn;
use crate::ports::config_repo::ConfigRepo;
use rusqlite::{params, OptionalExtension};

/// Adaptador SQLite que implementa `ConfigRepo` sobre `configuracion`.
pub struct ConfigSqlite {
    db: DbConn,
}

impl ConfigSqlite {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl ConfigRepo for ConfigSqlite {
    fn obtener(&self, clave: &str) -> AppResult<Option<String>> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        Ok(conn
            .query_row(
                "SELECT valor FROM configuracion WHERE clave = ?1",
                params![clave],
                |row| row.get(0),
            )
            .optional()?)
    }

    fn guardar(&self, clave: &str, valor: &str) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        conn.execute(
            "INSERT INTO configuracion (clave, valor) VALUES (?1, ?2) \
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![clave, valor],
        )?;
        Ok(())
    }

    fn eliminar(&self, clave: &str) -> AppResult<()> {
        let conn = self.db.conn.lock().expect("mutex de BD envenenado");
        conn.execute("DELETE FROM configuracion WHERE clave = ?1", params![clave])?;
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> ConfigSqlite {
        ConfigSqlite::new(DbConn::abrir_en_memoria().expect("BD en memoria"))
    }

    #[test]
    fn clave_inexistente_devuelve_none() {
        assert_eq!(repo().obtener("no_existe").unwrap(), None);
    }

    #[test]
    fn guardar_y_obtener() {
        let r = repo();
        r.guardar("tema", "oscuro").unwrap();
        assert_eq!(r.obtener("tema").unwrap().as_deref(), Some("oscuro"));
    }

    #[test]
    fn guardar_reemplaza_valor_existente() {
        let r = repo();
        r.guardar("sync", "15").unwrap();
        r.guardar("sync", "30").unwrap();
        assert_eq!(r.obtener("sync").unwrap().as_deref(), Some("30"));
    }

    #[test]
    fn eliminar_es_idempotente() {
        let r = repo();
        r.guardar("pin", "hash").unwrap();
        r.eliminar("pin").unwrap();
        r.eliminar("pin").unwrap();
        assert_eq!(r.obtener("pin").unwrap(), None);
    }
}
