//! Commands Tauri del auto-update.
//!
//! El endpoint vive en las preferencias (`actualizaciones_url`) para poder
//! cambiarlo sin recompilar. La verificación de firma la hace el plugin con la
//! clave pública embebida en `tauri.conf.json`.

use crate::application::config_service::ConfiguracionApp;
use crate::infrastructure::db::DbConn;
use crate::infrastructure::system::actualizacion::{self, InfoActualizacion};
use serde::Serialize;
use tauri::{AppHandle, State};

/// Estado que muestra Configuración: versión instalada y si hay endpoint.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EstadoActualizacion {
    pub version_actual: String,
    pub endpoint: String,
    pub endpoint_configurado: bool,
    pub auto_verificar: bool,
}

fn configuracion(db: &DbConn) -> Result<ConfiguracionApp, String> {
    super::config_cmd::servicio_config(db)?
        .obtener()
        .map_err(|e| e.to_string())
}

fn endpoint_configurado(db: &DbConn) -> Result<String, String> {
    let endpoint = configuracion(db)?.actualizaciones_url;
    if endpoint.trim().is_empty() {
        return Err("Configurá la URL de actualizaciones en Configuración".to_string());
    }
    Ok(endpoint)
}

/// Versión actual y estado de la configuración de auto-update.
#[tauri::command]
pub fn estado_actualizacion(
    app: AppHandle,
    db: State<'_, DbConn>,
) -> Result<EstadoActualizacion, String> {
    let configuracion = configuracion(db.inner())?;
    let endpoint = configuracion.actualizaciones_url.trim().to_string();
    Ok(EstadoActualizacion {
        version_actual: app.package_info().version.to_string(),
        endpoint_configurado: !endpoint.is_empty(),
        endpoint,
        auto_verificar: configuracion.actualizaciones_auto,
    })
}

/// Consulta el manifiesto. `None` = ya se está en la última versión.
#[tauri::command]
pub async fn buscar_actualizacion(
    app: AppHandle,
    db: State<'_, DbConn>,
) -> Result<Option<InfoActualizacion>, String> {
    let endpoint = endpoint_configurado(db.inner())?;
    actualizacion::buscar(&app, &endpoint)
        .await
        .map_err(String::from)
}

/// Descarga, verifica e instala la versión disponible. En Windows la app se
/// cierra y relanza sola al terminar el instalador.
#[tauri::command]
pub async fn instalar_actualizacion(
    app: AppHandle,
    db: State<'_, DbConn>,
) -> Result<String, String> {
    let endpoint = endpoint_configurado(db.inner())?;
    let version = actualizacion::instalar(&app, &endpoint)
        .await
        .map_err(String::from)?;
    log::info!("Actualización {version} descargada e instalada; la app se reinicia");
    Ok(version)
}
