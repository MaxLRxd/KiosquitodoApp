//! Inicialización del logging con rotación diaria y retención limitada.

use crate::errors::AppResult;
use crate::config::defaults;
use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Logger, Naming};
use std::path::Path;

/// Inicia el logging rotativo en `log_dir`.
///
/// - Nivel por defecto `info` (sobreescribible vía `RUST_LOG`).
/// - Un archivo por día, borrando los que superen `LOG_RETENCION_DIAS`.
pub fn init_logging(log_dir: &Path) -> AppResult<()> {
    Logger::try_with_str("warn, kiosco_app=info")
        .map_err(|e| std::io::Error::other(e.to_string()))?
        .log_to_file(FileSpec::default().directory(log_dir).basename("kiosco"))
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(defaults::LOG_RETENCION_DIAS as usize),
        )
        .start()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}