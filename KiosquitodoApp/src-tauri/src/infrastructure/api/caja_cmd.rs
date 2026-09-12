//! Commands Tauri del módulo de caja: resumen diario, cierre y export.

use crate::application::cierre_service::CierreService;
use crate::domain::cierre::ResumenCierre;
use crate::ports::cierre_repo::CierrePersistido;
use crate::infrastructure::db::DbConn;
use crate::infrastructure::db::repos::CierreSqlite;
use crate::ports::cierre_repo::CierreRepo;
use tauri::State;

/// Calcula el resumen del día sin persistir (previsualización del cierre).
#[tauri::command]
pub fn resumen_diario(db: State<'_, DbConn>, fecha: String) -> Result<ResumenCierre, String> {
    let repo = CierreSqlite::new(db.inner().clone());
    CierreService::new(repo)
        .resumen_diario(&fecha)
        .map_err(|e| e.to_string())
}

/// Consolida y persiste el cierre de caja del día + VACUUM (R10).
#[tauri::command]
pub fn cerrar_caja(db: State<'_, DbConn>, fecha: String) -> Result<CierrePersistido, String> {
    let repo = CierreSqlite::new(db.inner().clone());
    CierreService::new(repo)
        .cerrar_caja(&fecha)
        .map_err(|e| e.to_string())
}

/// Lista los cierres guardados, más reciente primero.
#[tauri::command]
pub fn listar_cierres(db: State<'_, DbConn>) -> Result<Vec<CierrePersistido>, String> {
    let repo = CierreSqlite::new(db.inner().clone());
    CierreService::new(repo)
        .listar_cierres()
        .map_err(|e| e.to_string())
}

/// Exporta los cierres del rango `[desde, hasta]` a CSV.
#[tauri::command]
pub fn exportar_cierres_csv(
    db: State<'_, DbConn>,
    desde: String,
    hasta: String,
) -> Result<String, String> {
    let repo = CierreSqlite::new(db.inner().clone());
    CierreService::new(repo)
        .exportar_csv(&desde, &hasta)
        .map_err(|e| e.to_string())
}
