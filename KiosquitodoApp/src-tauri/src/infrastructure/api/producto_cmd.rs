//! Comandos Tauri del mÃ³dulo inventario (productos y stock).
//!
//! Cada command es un envoltorio fino: resuelve el repo SQLite desde el
//! `DbConn` compartido, construye el servicio y delega. Los errores se
//! serializan como `String` legible para el frontend.

use crate::application::producto_service::ProductoService;
use crate::domain::producto::{NuevoProducto, Producto};
use crate::errors::{AppError, AppResult};
use crate::infrastructure::db::DbConn;
use crate::infrastructure::db::repos::ProductoSqlite;
use crate::ports::producto_repo::ProductoRepo;
use tauri::State;

/// Crear un producto en el inventario (valida reglas, R9).
#[tauri::command]
pub fn crear_producto(
    db: State<'_, DbConn>,
    nuevo: NuevoProducto,
) -> Result<Producto, String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .crear_producto(&nuevo)
        .map_err(|e| e.to_string())
}

/// Lista productos con filtro de texto, categorÃ­a y paginaciÃ³n.
#[tauri::command]
pub fn listar_productos(
    db: State<'_, DbConn>,
    filtro: String,
    categoria_id: Option<i64>,
    limite: i64,
    offset: i64,
) -> Result<Vec<Producto>, String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .listar_productos(&filtro, categoria_id, limite, offset)
        .map_err(|e| e.to_string())
}

/// Busca un producto por cÃ³digo de barras (para el POS).
#[tauri::command]
pub fn buscar_producto_por_barcode(
    db: State<'_, DbConn>,
    barcode: String,
) -> Result<Option<Producto>, String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .buscar_por_barcode(&barcode)
        .map_err(|e| e.to_string())
}

/// Actualiza los campos editables de un producto.
#[tauri::command]
pub fn actualizar_producto(
    db: State<'_, DbConn>,
    producto: Producto,
) -> Result<(), String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .actualizar_producto(&producto)
        .map_err(|e| e.to_string())
}

/// Baja lÃ³gica de un producto.
#[tauri::command]
pub fn eliminar_producto(db: State<'_, DbConn>, id: i64) -> Result<(), String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .eliminar_producto(id)
        .map_err(|e| e.to_string())
}

/// Ajuste manual de stock registrando el movimiento.
#[tauri::command]
pub fn ajustar_stock(
    db: State<'_, DbConn>,
    id: i64,
    stock_nuevo: i64,
    motivo: String,
    operador: Option<String>,
) -> Result<(), String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .ajustar_stock(id, stock_nuevo, &motivo, operador.as_deref())
        .map_err(|e| e.to_string())
}
