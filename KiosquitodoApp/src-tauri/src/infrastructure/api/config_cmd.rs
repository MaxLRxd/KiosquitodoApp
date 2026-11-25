//! Commands Tauri del módulo de configuración del sistema: rutas, preferencias,
//! PIN, autostart y backups. La integración con Mercado Pago (token, keyring y
//! sync) queda DIFERIDA por decisión del equipo (P16 sin MP).

use crate::application::config_service::{ConfigService, ConfiguracionApp};
use crate::application::pin_service::PinService;
use crate::config::AppConfig;
use crate::infrastructure::db::repos::ConfigSqlite;
use crate::infrastructure::db::DbConn;
use crate::infrastructure::system::{autostart, backup};
use serde::Serialize;
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
    let cfg = AppConfig::resolver().map_err(|e| e.to_string())?;
    Ok(cfg.into())
}

/// Arma el servicio de configuración con la carpeta de backups por defecto.
fn servicio_config(db: &DbConn) -> Result<ConfigService<ConfigSqlite>, String> {
    let cfg = AppConfig::resolver().map_err(|e| e.to_string())?;
    Ok(ConfigService::new(
        ConfigSqlite::new(db.clone()),
        cfg.backup_dir.to_string_lossy().into_owned(),
    ))
}

/// Lee las preferencias persistidas, con defaults para las claves ausentes.
#[tauri::command]
pub fn obtener_configuracion(db: State<'_, DbConn>) -> Result<ConfiguracionApp, String> {
    servicio_config(db.inner())?
        .obtener()
        .map_err(|e| e.to_string())
}

/// Guarda las preferencias y devuelve el estado ya normalizado.
#[tauri::command]
pub fn guardar_configuracion(
    db: State<'_, DbConn>,
    configuracion: ConfiguracionApp,
) -> Result<ConfiguracionApp, String> {
    let guardada = servicio_config(db.inner())?
        .guardar(&configuracion)
        .map_err(|e| e.to_string())?;
    log::info!(
        "Configuración guardada: sync {} min, inactividad {} min, sonidos {}",
        guardada.sync_intervalo_minutos,
        guardada.inactividad_minutos,
        guardada.sonidos_activos
    );
    Ok(guardada)
}

/// Arma el servicio de PIN sobre la misma BD de configuración.
fn servicio_pin(db: &DbConn) -> PinService<ConfigSqlite> {
    PinService::new(ConfigSqlite::new(db.clone()))
}

/// ¿Hay un PIN configurado? Nunca devuelve el hash ni el salt.
#[tauri::command]
pub fn pin_configurado(db: State<'_, DbConn>) -> Result<bool, String> {
    servicio_pin(db.inner())
        .configurado()
        .map_err(|e| e.to_string())
}

/// Guarda un PIN nuevo (4-6 dígitos) como hash + salt. Devuelve solo el estado.
#[tauri::command]
pub fn establecer_pin(db: State<'_, DbConn>, pin: String) -> Result<bool, String> {
    let svc = servicio_pin(db.inner());
    svc.establecer(&pin).map_err(|e| e.to_string())?;
    log::info!("PIN configurado (hash SHA-256 con salt)");
    svc.configurado().map_err(|e| e.to_string())
}

/// Compara el PIN ingresado. Devuelve `true` solo si coincide; nunca el hash.
#[tauri::command]
pub fn verificar_pin(db: State<'_, DbConn>, pin: String) -> Result<bool, String> {
    let correcto = servicio_pin(db.inner())
        .verificar(&pin)
        .map_err(|e| e.to_string())?;
    if correcto {
        log::info!("Desbloqueo por PIN correcto");
    } else {
        log::warn!("Intento de desbloqueo con PIN incorrecto");
    }
    Ok(correcto)
}

/// Quita el PIN y, con él, el bloqueo por inactividad.
#[tauri::command]
pub fn eliminar_pin(db: State<'_, DbConn>) -> Result<(), String> {
    servicio_pin(db.inner())
        .eliminar()
        .map_err(|e| e.to_string())?;
    log::info!("PIN eliminado");
    Ok(())
}

/// ¿Está habilitado el arranque automático al iniciar sesión?
#[tauri::command]
pub fn autostart_estado(app: AppHandle) -> Result<bool, String> {
    autostart::autostart_habilitado(&app).map_err(|e| e.to_string())
}

/// Habilita el arranque automático con Windows.
#[tauri::command]
pub fn habilitar_autostart(app: AppHandle) -> Result<(), String> {
    autostart::habilitar_autostart(&app).map_err(|e| e.to_string())
}

/// Deshabilita el arranque automático.
#[tauri::command]
pub fn deshabilitar_autostart(app: AppHandle) -> Result<(), String> {
    autostart::deshabilitar_autostart(&app).map_err(|e| e.to_string())
}

/// Crea un backup manual (VACUUM INTO) y devuelve la ruta generada.
#[tauri::command]
pub fn crear_backup_manual(db: State<'_, DbConn>) -> Result<String, String> {
    let cfg = AppConfig::resolver().map_err(|e| e.to_string())?;
    let ruta = backup::crear_backup(db.inner(), &cfg.backup_dir).map_err(|e| e.to_string())?;
    let ruta = ruta.to_string_lossy().into_owned();
    log::info!("Backup manual creado en {}", ruta);
    Ok(ruta)
}

/// Elimina backups más viejos que `retencion_dias`; devuelve cuántos borró.
#[tauri::command]
pub fn limpiar_backups_viejos(retencion_dias: u32) -> Result<usize, String> {
    let cfg = AppConfig::resolver().map_err(|e| e.to_string())?;
    backup::limpiar_backups_viejos(&cfg.backup_dir, retencion_dias)
        .map_err(|e| e.to_string())
}
