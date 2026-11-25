//! Configuración de la app: rutas (data, backups, logs) y valores por defecto.
//!
//! Las rutas se resuelven una sola vez al inicio y se comparten como `AppConfig`.

use std::path::PathBuf;

/// Constantes de negocio extraídas de la SPEC. Centralizadas aquí para
/// que todos los módulos consuman la misma fuente de verdad.
pub mod defaults {
    /// Tolerancia de matching en centavos (±$1 = ±100 centavos).
    pub const MATCHING_MONTO_TOLERANCIA: i64 = 100;

    /// Ventana de matching en segundos (±10 minutos = ±600 segundos).
    pub const MATCHING_VENTANA_SEGUNDOS: i64 = 600;

    /// Intervalo de sync automático con MP en minutos.
    pub const SYNC_INTERVALO_MINUTOS: u64 = 15;

    /// Minutos sin interacción antes del bloqueo por PIN (0 = desactivado).
    pub const INACTIVIDAD_MINUTOS_DEFAULT: u64 = 5;

    /// Feedback sonoro activo por defecto.
    pub const SONIDOS_ACTIVOS_DEFAULT: bool = true;

    /// Manifiesto de actualizaciones publicado con cada release.
    /// Se puede cambiar desde Configuración (p. ej. para hosting propio).
    pub const ACTUALIZACIONES_URL_DEFAULT: &str =
        "https://github.com/MaxLRxd/KiosquitodoApp/releases/latest/download/latest.json";

    /// Verificar actualizaciones al iniciar la app.
    pub const ACTUALIZACIONES_AUTO_DEFAULT: bool = true;

    /// Stock mínimo por defecto para nuevos productos.
    pub const STOCK_MINIMO_DEFAULT: i64 = 5;

    /// Máximo de días de logs a conservar.
    pub const LOG_RETENCION_DIAS: u32 = 30;

    /// Máximo de días de backups a conservar.
    pub const BACKUP_RETENCION_DIAS: u32 = 30;

    /// Timeout de HTTP a la API de MP en segundos.
    pub const HTTP_TIMEOUT_SECS: u64 = 15;

    /// Cantidad máxima de resultados por request a la API de MP.
    pub const MP_PAGE_LIMIT: u32 = 50;
}

/// Carpeta de datos en la raíz del proyecto (`<repo>/datos-pruebas/`).
///
/// En modo desarrollo la BD vive aquí para poder inspeccionarla a mano y
/// corroborar la persistencia al correr los tests o `tauri dev`.
/// `CARGO_MANIFEST_DIR` apunta a `.../KiosquitodoApp/src-tauri`, así que
/// subimos dos niveles hasta la raíz del repositorio.
pub fn carpeta_datos_proyecto() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|raiz| raiz.join("datos-pruebas"))
        .unwrap_or_else(|| PathBuf::from("datos-pruebas"))
}

/// Rutas resueltas del sistema de archivos. Se crean al inicio y no cambian.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Directorio de datos de la app. En dev: `<repo>/datos-pruebas/`
    /// (temporalmente; en release debería volver a `%APPDATA%/KiosquitodoApp/`).
    pub data_dir: PathBuf,
    /// Directorio de backups (`%USERPROFILE%/Documents/KioscoBackups/`)
    pub backup_dir: PathBuf,
    /// Directorio de logs (`%USERPROFILE%/KiosquitodoApp/logs/`)
    pub log_dir: PathBuf,
    /// Ruta completa al archivo de base de datos
    pub db_path: PathBuf,
}

impl AppConfig {
    /// Resuelve las rutas del sistema operativo y crea los directorios
    /// si no existen.
    pub fn resolver() -> Result<Self, std::io::Error> {
        let home = dirs_home();

        // Modo desarrollo: la BD vive en la raíz del proyecto para poder
        // inspeccionarla y verificar la persistencia (ver `carpeta_datos_proyecto`).
        let data_dir = carpeta_datos_proyecto();
        let backup_dir = home.join("Documents").join("KioscoBackups");
        let log_dir = home.join("KiosquitodoApp").join("logs");

        // Crear directorios si no existen
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&backup_dir)?;
        std::fs::create_dir_all(&log_dir)?;

        let db_path = data_dir.join("kiosco.db");

        Ok(Self {
            data_dir,
            backup_dir,
            log_dir,
            db_path,
        })
    }
}

/// Obtiene el directorio home del usuario. En Windows: `C:\Users\<usuario>`.
fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:/Users/default"))
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_coherentes() {
        // La tolerancia de matching debe ser 100 centavos ($1)
        assert_eq!(defaults::MATCHING_MONTO_TOLERANCIA, 100);
        // La ventana debe ser 600 segundos (10 minutos)
        assert_eq!(defaults::MATCHING_VENTANA_SEGUNDOS, 600);
        // El sync cada 15 minutos
        assert_eq!(defaults::SYNC_INTERVALO_MINUTOS, 15);
        // Bloqueo por inactividad a los 5 minutos, con sonidos activos
        assert_eq!(defaults::INACTIVIDAD_MINUTOS_DEFAULT, 5);
        assert!(defaults::SONIDOS_ACTIVOS_DEFAULT);
        // Auto-update apuntando al manifiesto de releases del repositorio
        assert!(defaults::ACTUALIZACIONES_URL_DEFAULT.starts_with("https://"));
        assert!(defaults::ACTUALIZACIONES_URL_DEFAULT.ends_with("/latest.json"));
        assert!(defaults::ACTUALIZACIONES_AUTO_DEFAULT);
    }

    #[test]
    fn resolver_rutas_no_panic() {
        // Solo verificamos que no panickea; los directorios pueden
        // no existir en un entorno de CI, pero crear_dir_all los crea.
        let config = AppConfig::resolver().expect("No se pudieron resolver rutas");
        assert!(config.db_path.to_string_lossy().contains("kiosco.db"));
        assert!(config.log_dir.to_string_lossy().contains("logs"));
    }

    #[test]
    fn home_dir_resuelto() {
        let home = dirs_home();
        assert!(home.exists(), "El directorio home debe existir");
    }
}
