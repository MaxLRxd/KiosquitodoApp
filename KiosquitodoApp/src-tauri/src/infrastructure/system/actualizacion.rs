//! Auto-actualización sobre `tauri-plugin-updater`.
//!
//! El endpoint se toma de las preferencias (no de `tauri.conf.json`) para que
//! el kiosco pueda apuntar a un hosting propio sin recompilar. La clave pública
//! de firma sí queda embebida en la configuración del plugin: es la que valida
//! que el instalador descargado fue firmado por nosotros.

use crate::errors::{AppError, AppResult};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// Evento IPC con el avance de la descarga.
pub const EVENTO_PROGRESO: &str = "actualizacion:progreso";

/// Datos de una versión disponible en el servidor de actualizaciones.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InfoActualizacion {
    pub version_actual: String,
    pub version_nueva: String,
    pub notas: Option<String>,
    pub fecha: Option<String>,
}

/// Progreso de descarga emitido al frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProgresoActualizacion {
    pub descargado: u64,
    pub total: Option<u64>,
    pub porcentaje: Option<u8>,
}

/// Valida el manifiesto configurado y lo convierte en `Url`.
pub fn validar_endpoint(endpoint: &str) -> AppResult<tauri::Url> {
    let limpio = endpoint.trim();
    if limpio.is_empty() {
        return Err(AppError::negocio(
            "Configurá la URL de actualizaciones en Configuración",
        ));
    }
    limpio
        .parse()
        .map_err(|e| AppError::negocio(format!("URL de actualizaciones inválida: {e}")))
}

/// Construye el actualizador con el endpoint de las preferencias.
fn construir(app: &AppHandle, endpoint: &str) -> AppResult<tauri_plugin_updater::Updater> {
    let url = validar_endpoint(endpoint)?;
    app.updater_builder()
        .endpoints(vec![url])
        .map_err(|e| AppError::Tauri(format!("Endpoint de actualizaciones inválido: {e}")))?
        .build()
        .map_err(|e| AppError::Tauri(format!("No se pudo preparar el actualizador: {e}")))
}

/// Consulta el manifiesto y devuelve la versión nueva, si existe.
pub async fn buscar(
    app: &AppHandle,
    endpoint: &str,
) -> AppResult<Option<InfoActualizacion>> {
    let updater = construir(app, endpoint)?;
    let update = updater.check().await.map_err(|e| {
        AppError::Tauri(format!("No se pudo consultar actualizaciones: {e}"))
    })?;
    Ok(update.map(|actualizacion| InfoActualizacion {
        version_actual: actualizacion.current_version,
        version_nueva: actualizacion.version.to_string(),
        notas: actualizacion.body,
        fecha: actualizacion.date.map(|fecha| fecha.to_string()),
    }))
}

/// Descarga, verifica la firma e instala la actualización disponible.
///
/// En Windows el instalador cierra la app y la relanza al terminar; por eso el
/// command puede no llegar a responder. El frontend muestra el progreso por
/// eventos y asume que la app se reinicia.
pub async fn instalar(app: &AppHandle, endpoint: &str) -> AppResult<String> {
    let updater = construir(app, endpoint)?;
    let actualizacion = updater
        .check()
        .await
        .map_err(|e| AppError::Tauri(format!("No se pudo consultar actualizaciones: {e}")))?
        .ok_or_else(|| AppError::negocio("No hay actualizaciones disponibles"))?;

    let version = actualizacion.version.to_string();
    let app_progreso = app.clone();
    let mut descargado = 0u64;

    actualizacion
        .download_and_install(
            move |chunk, total| {
                descargado = descargado.saturating_add(chunk as u64);
                let porcentaje = total
                    .filter(|tamano| *tamano > 0)
                    .map(|tamano| (descargado.saturating_mul(100) / tamano).min(100) as u8);
                let _ = app_progreso.emit(
                    EVENTO_PROGRESO,
                    ProgresoActualizacion {
                        descargado,
                        total,
                        porcentaje,
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|e| {
            AppError::Tauri(format!("No se pudo descargar o instalar la actualización: {e}"))
        })?;

    Ok(version)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valida_endpoints_http_y_https() {
        assert!(validar_endpoint("https://example.com/latest.json").is_ok());
        assert!(validar_endpoint("http://localhost:8080/latest.json").is_ok());
    }

    #[test]
    fn rechaza_endpoint_vacio_o_invalido() {
        assert!(validar_endpoint("   ").is_err());
        assert!(validar_endpoint("no-es-una-url").is_err());
    }
}
