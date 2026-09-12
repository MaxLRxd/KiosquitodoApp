//! Commands Tauri del módulo de ventas (POS local, R9).

use crate::application::venta_service::VentaService;
use crate::domain::venta::{NuevaVenta, Venta, VentaItem};
use crate::infrastructure::db::{DbConn, repos::{VentaSqlite, ProductoSqlite}};
use crate::ports::venta_repo::VentaRepo;
use crate::ports::producto_repo::ProductoRepo;
use tauri::State;

/// Registra una venta descontando stock y validando el medio de pago (R9).
#[tauri::command]
pub fn registrar_venta(db: State<'_, DbConn>, nueva: NuevaVenta) -> Result<Venta, String> {
    let venta_repo = VentaSqlite::new(db.inner().clone());
    let producto_repo = ProductoSqlite::new(db.inner().clone());
    VentaService::new(venta_repo, producto_repo)
        .registrar_venta(&nueva)
        .map_err(|e| e.to_string())
}

/// Obtiene una venta completa (con ítems) por id.
#[tauri::command]
pub fn obtener_venta(db: State<'_, DbConn>, id: i64) -> Result<Option<Venta>, String> {
    let venta_repo = VentaSqlite::new(db.inner().clone());
    let producto_repo = ProductoSqlite::new(db.inner().clone());
    VentaService::new(venta_repo, producto_repo)
        .obtener_venta(id)
        .map_err(|e| e.to_string())
}

/// Lista las ventas de un día (`YYYY-MM-DD`).
#[tauri::command]
pub fn listar_ventas_dia(
    db: State<'_, DbConn>,
    fecha: String,
) -> Result<Vec<Venta>, String> {
    let venta_repo = VentaSqlite::new(db.inner().clone());
    let producto_repo = ProductoSqlite::new(db.inner().clone());
    VentaService::new(venta_repo, producto_repo)
        .listar_ventas_dia(&fecha)
        .map_err(|e| e.to_string())
}

/// Registra una devolución y repone el stock de los ítems devueltos.
#[tauri::command]
pub fn registrar_devolucion(
    db: State<'_, DbConn>,
    venta_id: i64,
    items: Vec<VentaItem>,
    motivo: Option<String>,
) -> Result<(), String> {
    let venta_repo = VentaSqlite::new(db.inner().clone());
    let producto_repo = ProductoSqlite::new(db.inner().clone());
    VentaService::new(venta_repo, producto_repo)
        .registrar_devolucion(venta_id, &items, motivo.as_deref())
        .map_err(|e| e.to_string())
}
