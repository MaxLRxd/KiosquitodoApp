//! Trait de almacenamiento seguro del Access Token (keyring).

use crate::errors::AppResult;

/// Contrato del almacenamiento cifrado de credenciales (R10).
///
/// La implementación real usa el Windows Credential Store vía `keyring`.
pub trait CredencialRepo: Send + Sync {
    /// Guarda (o reemplaza) el Access Token de MP cifrado.
    fn guardar_token(&self, token: &str) -> AppResult<()>;

    /// Recupera el Access Token almacenado. `Ok(None)` si no existe.
    fn obtener_token(&self) -> AppResult<Option<String>>;

    /// Elimina el Access Token almacenado.
    fn eliminar_token(&self) -> AppResult<()>;
}