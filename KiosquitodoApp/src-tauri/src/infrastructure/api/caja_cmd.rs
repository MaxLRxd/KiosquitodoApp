//! Commands Tauri del módulo de caja: resumen diario, cierre y export.

use crate::application::cierre_service::CierreService;
use crate::domain::cierre::ResumenCierre;
use crate::ports::cierre_repo::CierrePersistido;
use crate::infrastructure::db::DbConn;
use crate::infrastructure::db::repos::CierreSqlite;
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
    let cierre = CierreService::new(repo)
        .cerrar_caja(&fecha)
        .map_err(|e| e.to_string())?;

    log::info!(
        "Cierre {} registrado: {} ventas, total {} centavos",
        cierre.fecha,
        cierre.resumen.cantidad_ventas,
        cierre.resumen.total_ventas
    );
    Ok(cierre)
}

/// Indica si la fecha ya fue cerrada (para habilitar la reapertura en la UI).
#[tauri::command]
pub fn caja_esta_cerrada(db: State<'_, DbConn>, fecha: String) -> Result<bool, String> {
    let repo = CierreSqlite::new(db.inner().clone());
    CierreService::new(repo)
        .esta_cerrado(&fecha)
        .map_err(|e| e.to_string())
}

/// Reabre el día borrando su cierre persistido (idempotente).
#[tauri::command]
pub fn reabrir_caja(db: State<'_, DbConn>, fecha: String) -> Result<(), String> {
    let repo = CierreSqlite::new(db.inner().clone());
    CierreService::new(repo)
        .reabrir_caja(&fecha)
        .map_err(|e| e.to_string())?;
    log::info!("Caja reabierta para la fecha {fecha}");
    Ok(())
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
