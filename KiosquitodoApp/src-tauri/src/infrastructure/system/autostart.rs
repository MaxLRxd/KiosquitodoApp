//! Control del inicio automático de la app con Windows (autostart).
//!
//! El plugin se registra en `lib.rs`; estos helpers exponen un contrato simple
//! para los comandos de configuración.

use crate::errors::{AppError, AppResult};
use tauri_plugin_autostart::ManagerExt;

/// ¿Está habilitado el arranque automático?
pub fn autostart_habilitado(app: &tauri::AppHandle) -> AppResult<bool> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| AppError::negocio(e.to_string()))
}

/// Habilita el arranque automático al iniciar sesión.
pub fn habilitar_autostart(app: &tauri::AppHandle) -> AppResult<()> {
    app.autolaunch()
        .enable()
        .map_err(|e| AppError::negocio(e.to_string()))
}

/// Deshabilita el arranque automático.
pub fn deshabilitar_autostart(app: &tauri::AppHandle) -> AppResult<()> {
    app.autolaunch()
        .disable()
        .map_err(|e| AppError::negocio(e.to_string()))
}