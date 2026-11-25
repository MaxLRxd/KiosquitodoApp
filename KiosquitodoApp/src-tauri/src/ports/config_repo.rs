//! Trait del repositorio de configuración (pares clave/valor).

use crate::errors::AppResult;

/// Contrato de persistencia de los ajustes de la app.
///
/// La tabla `configuracion(clave, valor)` ya existe en la migración 1. El
/// servicio de aplicación es el único que interpreta el significado de cada
/// clave; acá solo se guardan textos.
pub trait ConfigRepo: Send + Sync {
    /// Devuelve el valor de `clave`, o `None` si no existe.
    fn obtener(&self, clave: &str) -> AppResult<Option<String>>;

    /// Inserta o reemplaza el valor de `clave`.
    fn guardar(&self, clave: &str, valor: &str) -> AppResult<()>;

    /// Elimina `clave`. No falla si la clave no existe.
    fn eliminar(&self, clave: &str) -> AppResult<()>;
}
