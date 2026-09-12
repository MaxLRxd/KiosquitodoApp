//! Commands Tauri del mÃ³dulo de configuraciÃ³n del sistema: rutas, autostart
//! y backups. La integraciÃ³n con Mercado Pago (token, keyring y sync) queda
//! DIFERIDA por decisiÃ³n del equipo (P16 sin MP).

use crate::config::AppConfig;
use crate::errors::{AppError, AppResult};
use crate::infrastructure::db::DbConn;
use crate::infrastructure::system::{autostart, backup};
use crate::ports::config_repo::ConfigRepo;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, State};

/// DTO con las rutas resueltas (data, backups, logs y BD).
#[derive(Debug, Clone, Serialize)]
pub struct RutasApp {
    pub data_dir: String,
    pub backup_dir: String,
    pub log_dir: String,
    pub db_path: String,
}

impl From<AppConfig> for RutasApp {
    fn from(c: AppConfig) -> Self {
        let AppConfig { data_dir, backup_dir, log_dir, db_path } = c;
        Self {
            data_dir: data_dir.to_string_lossy().into_owned(),
            backup_dir: backup_dir.to_string_lossy().into_owned(),
            log_dir: log_dir.to_string_lossy().into_owned(),
            db_path: db_path.to_string_lossy().into_owned(),
        }
    }
}

/// Devuelve las rutas del sistema para el frontend (aprox. `/actuator/env`).
#[tauri::command]
pub fn obtener_rutas() -> Result<RutasApp, String> {
    let cfg = AppConfig::resolver()?;
    Ok(cfg.into())
}

/// Â¿EstÃ¡ habilitado el arranque automÃ¡tico al iniciar sesiÃ³n?
#[tauri::command]
pub fn autostart_estado(app: AppHandle) -> Result<bool, String> {
    autostart::autostart_habilitado(&app).map_err(|e| e.to_string())
}

/// Habilita el arranque automÃ¡tico con Windows.
#[tauri::command]
pub fn habilitar_autostart(app: AppHandle) -> Result<(), String> {
    autostart::habilitar_autostart(&app).map_err(|e| e.to_string())
}

/// Deshabilita el arranque automÃ¡tico.
#[tauri::command]
pub fn deshabilitar_autostart(app: AppHandle) -> Result<(), String> {
    autostart::deshabilitar_autostart(&app).map_err(|e| e.to_string())
}

/// Crea un backup manual (VACUUM INTO) y devuelve la ruta generada.
#[tauri::command]
pub fn crear_backup_manual(db: State<'_, DbConn>) -> Result<String, String> {
    let cfg = AppConfig::resolver()?;
    let ruta = backup::crear_backup(db.inner(), &cfg.backup_dir)
        .map_err(|e| e.to_string())?;
    Ok(ruta.to_string_lossy().into_owned())
}

/// Elimina backups mÃ¡s viejos que `retencion_dias`; devuelve cuÃ¡ntos borrÃ³.
#[tauri::command]
pub fn limpiar_backups_viejos(retencion_dias: u32) -> Result<usize, String> {
    let cfg = AppConfig::resolver()?;
    backup::limpiar_backups_viejos(&cfg.backup_dir, retencion_dias)
        .map_err(|e| e.to_string())
}
