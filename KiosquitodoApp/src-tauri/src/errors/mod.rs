//! `AppError`, alias `AppResult<T>` y conversiones `From` de errores externos.
//!
//! Todos los módulos internos retornan `AppResult<T>`. La conversión a `String`
//! ocurre únicamente en la frontera IPC (comandos Tauri).

use std::fmt;

/// Error tipado de la aplicación. Cada variante encapsula un dominio de error
/// distinto, manteniendo la trazabilidad sin perder información.
#[derive(Debug)]
pub enum AppError {
    /// Error de base de datos (rusqlite)
    Db(rusqlite::Error),
    /// Error de E/S de archivos (backup, logs, restore)
    Io(std::io::Error),
    /// Error de HTTP / red (reqwest)
    Http(reqwest::Error),
    /// Error de keyring (credential store del SO)
    Keyring(keyring::Error),
    /// Error de serialización JSON
    Json(serde_json::Error),
    /// Violation de una regla de negocio (mensaje descriptivo)
    Negocio(String),
    /// Error genérico de Tauri
    Tauri(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Db(e) => write!(f, "Error de base de datos: {e}"),
            Self::Io(e) => write!(f, "Error de E/S: {e}"),
            Self::Http(e) => write!(f, "Error de red: {e}"),
            Self::Keyring(e) => write!(f, "Error de credenciales: {e}"),
            Self::Json(e) => write!(f, "Error de serialización: {e}"),
            Self::Negocio(msg) => write!(f, "Regla de negocio: {msg}"),
            Self::Tauri(msg) => write!(f, "Error de Tauri: {msg}"),
        }
    }
}

/// Alias conveniente para Result con AppError.
pub type AppResult<T> = Result<T, AppError>;

// ── Conversiones From ────────────────────────────────────────────────
// Permiten usar el operador `?` con errores externos dentro de `AppResult`.

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        Self::Keyring(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        Self::Negocio(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        Self::Negocio(msg.to_string())
    }
}

// ── Conversión a String (solo para frontera IPC) ─────────────────────

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_db_error() {
        let err = AppError::from(rusqlite::Error::QueryReturnedNoRows);
        assert!(err.to_string().contains("base de datos"));
    }

    #[test]
    fn display_negocio() {
        let err = AppError::from("stock insuficiente");
        assert_eq!(err.to_string(), "Regla de negocio: stock insuficiente");
    }

    #[test]
    fn conversion_a_string_ipc() {
        let err: AppError = "test".into();
        let msg: String = err.into();
        assert_eq!(msg, "Regla de negocio: test");
    }

    #[test]
    fn app_result_ok() {
        let result: AppResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }
}
