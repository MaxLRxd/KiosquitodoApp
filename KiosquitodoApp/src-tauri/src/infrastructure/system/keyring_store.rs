//! Almacenamiento cifrado del Access Token de MP (R10) vía el keyring del SO
//! (en Windows: Credenciales de Windows).

use crate::errors::AppResult;
use crate::ports::credencial_repo::CredencialRepo;

const SERVICIO: &str = "KiosquitodoApp";
const CUENTA: &str = "mercado_pago_access_token";

/// Adaptador del Windows Credential Store que implementa `CredencialRepo`.
pub struct KeyringCredenciales {
    entry: keyring::Entry,
}

impl KeyringCredenciales {
    pub fn new() -> AppResult<Self> {
        let entry = keyring::Entry::new(SERVICIO, CUENTA)?;
        Ok(Self { entry })
    }
}

impl CredencialRepo for KeyringCredenciales {
    fn guardar_token(&self, token: &str) -> AppResult<()> {
        self.entry.set_password(token)?;
        Ok(())
    }

    fn obtener_token(&self) -> AppResult<Option<String>> {
        match self.entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn eliminar_token(&self) -> AppResult<()> {
        match self.entry.delete_password() {
            Ok(()) => Ok(()),
            // Borrar una credencial inexistente no es un error.
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}