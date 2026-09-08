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

/// Rutas resueltas del sistema de archivos. Se crean al inicio y no cambian.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Directorio de datos de la app (`%APPDATA%/KiosquitodoApp/`)
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

        let data_dir = home.join("AppData").join("Roaming").join("KiosquitodoApp");
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
