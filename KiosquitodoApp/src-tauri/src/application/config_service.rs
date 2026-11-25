//! Casos de uso de configuración: defaults tipados, lectura y escritura.
//!
//! El repositorio guarda textos; este servicio es el único que conoce el
//! significado y el tipo de cada clave. El DTO viaja al frontend por IPC.

use crate::config::defaults;
use crate::errors::{AppError, AppResult};
use crate::ports::config_repo::ConfigRepo;
use serde::{Deserialize, Serialize};

const CLAVE_SYNC: &str = "sync_intervalo_minutos";
const CLAVE_INACTIVIDAD: &str = "inactividad_minutos";
const CLAVE_SONIDOS: &str = "sonidos_activos";
const CLAVE_BACKUP_DIR: &str = "backup_dir";

/// Configuración de la app expuesta al frontend (espejo en `lib/types`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConfiguracionApp {
    /// Minutos entre sincronizaciones automáticas con MP.
    pub sync_intervalo_minutos: u64,
    /// Minutos sin interacción antes de bloquear. `0` = desactivado.
    pub inactividad_minutos: u64,
    /// Beeps de feedback (escaneo, venta, errores).
    pub sonidos_activos: bool,
    /// Directorio configurado para los backups automáticos.
    pub backup_dir: String,
}

impl ConfiguracionApp {
    /// Valores por defecto. La carpeta se resuelve desde `AppConfig` en la
    /// frontera IPC para no acoplar el servicio al sistema de archivos.
    pub fn por_defecto(backup_dir_default: &str) -> Self {
        Self {
            sync_intervalo_minutos: defaults::SYNC_INTERVALO_MINUTOS,
            inactividad_minutos: defaults::INACTIVIDAD_MINUTOS_DEFAULT,
            sonidos_activos: defaults::SONIDOS_ACTIVOS_DEFAULT,
            backup_dir: backup_dir_default.to_string(),
        }
    }
}

/// Servicio de configuración sobre un `ConfigRepo`.
pub struct ConfigService<R: ConfigRepo> {
    repo: R,
    backup_dir_default: String,
}

impl<R: ConfigRepo> ConfigService<R> {
    pub fn new(repo: R, backup_dir_default: impl Into<String>) -> Self {
        Self {
            repo,
            backup_dir_default: backup_dir_default.into(),
        }
    }

    /// Lee la configuración; las claves ausentes toman su default tipado.
    pub fn obtener(&self) -> AppResult<ConfiguracionApp> {
        let por_defecto = ConfiguracionApp::por_defecto(&self.backup_dir_default);
        Ok(ConfiguracionApp {
            sync_intervalo_minutos: self
                .leer_u64(CLAVE_SYNC, por_defecto.sync_intervalo_minutos)?,
            inactividad_minutos: self
                .leer_u64(CLAVE_INACTIVIDAD, por_defecto.inactividad_minutos)?,
            sonidos_activos: self.leer_bool(CLAVE_SONIDOS, por_defecto.sonidos_activos)?,
            backup_dir: self
                .repo
                .obtener(CLAVE_BACKUP_DIR)?
                .unwrap_or(por_defecto.backup_dir),
        })
    }

    /// Persiste la configuración y devuelve el estado ya normalizado.
    pub fn guardar(&self, configuracion: &ConfiguracionApp) -> AppResult<ConfiguracionApp> {
        if configuracion.sync_intervalo_minutos == 0 {
            return Err(AppError::negocio(
                "El intervalo de sync debe ser al menos 1 minuto",
            ));
        }
        if configuracion.backup_dir.trim().is_empty() {
            return Err(AppError::negocio(
                "La carpeta de backups no puede estar vacía",
            ));
        }

        self.repo.guardar(
            CLAVE_SYNC,
            &configuracion.sync_intervalo_minutos.to_string(),
        )?;
        self.repo.guardar(
            CLAVE_INACTIVIDAD,
            &configuracion.inactividad_minutos.to_string(),
        )?;
        self.repo.guardar(
            CLAVE_SONIDOS,
            if configuracion.sonidos_activos {
                "true"
            } else {
                "false"
            },
        )?;
        self.repo
            .guardar(CLAVE_BACKUP_DIR, configuracion.backup_dir.trim())?;

        self.obtener()
    }

    fn leer_u64(&self, clave: &str, por_defecto: u64) -> AppResult<u64> {
        match self.repo.obtener(clave)? {
            Some(valor) => valor
                .trim()
                .parse::<u64>()
                .map_err(|_| AppError::negocio(format!("Valor inválido para '{clave}': {valor}"))),
            None => Ok(por_defecto),
        }
    }

    fn leer_bool(&self, clave: &str, por_defecto: bool) -> AppResult<bool> {
        match self.repo.obtener(clave)? {
            Some(valor) => match valor.trim().to_ascii_lowercase().as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(AppError::negocio(format!(
                    "Valor inválido para '{clave}': {valor}"
                ))),
            },
            None => Ok(por_defecto),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RepoMemoria {
        valores: Mutex<HashMap<String, String>>,
    }

    impl ConfigRepo for RepoMemoria {
        fn obtener(&self, clave: &str) -> AppResult<Option<String>> {
            Ok(self.valores.lock().unwrap().get(clave).cloned())
        }

        fn guardar(&self, clave: &str, valor: &str) -> AppResult<()> {
            self.valores
                .lock()
                .unwrap()
                .insert(clave.to_string(), valor.to_string());
            Ok(())
        }

        fn eliminar(&self, clave: &str) -> AppResult<()> {
            self.valores.lock().unwrap().remove(clave);
            Ok(())
        }
    }

    fn servicio() -> ConfigService<RepoMemoria> {
        ConfigService::new(RepoMemoria::default(), "C:/backups")
    }

    #[test]
    fn defaults_cuando_no_hay_valores_guardados() {
        let configuracion = servicio().obtener().unwrap();
        assert_eq!(configuracion.sync_intervalo_minutos, 15);
        assert_eq!(configuracion.inactividad_minutos, 5);
        assert!(configuracion.sonidos_activos);
        assert_eq!(configuracion.backup_dir, "C:/backups");
    }

    #[test]
    fn guardar_persiste_y_devuelve_el_estado() {
        let svc = servicio();
        let cambios = ConfiguracionApp {
            sync_intervalo_minutos: 30,
            inactividad_minutos: 0,
            sonidos_activos: false,
            backup_dir: "D:/KioscoBackups".to_string(),
        };
        assert_eq!(svc.guardar(&cambios).unwrap(), cambios);
        assert_eq!(svc.obtener().unwrap(), cambios);
    }

    #[test]
    fn rechaza_sync_cero_y_carpeta_vacia() {
        let svc = servicio();
        let mut cambios = ConfiguracionApp::por_defecto("C:/backups");
        cambios.sync_intervalo_minutos = 0;
        assert!(svc.guardar(&cambios).is_err());

        cambios.sync_intervalo_minutos = 15;
        cambios.backup_dir = "   ".to_string();
        assert!(svc.guardar(&cambios).is_err());
    }

    #[test]
    fn valores_corruptos_se_reportan_con_la_clave() {
        let svc = servicio();
        svc.repo.guardar(CLAVE_SYNC, "no-es-numero").unwrap();
        let error = svc.obtener().unwrap_err().to_string();
        assert!(error.contains(CLAVE_SYNC));
    }
}
