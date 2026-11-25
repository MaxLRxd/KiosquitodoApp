//! Comandos Tauri del módulo inventario (productos y stock).
//!
//! Cada command es un envoltorio fino: resuelve el repo SQLite desde el
//! `DbConn` compartido, construye el servicio y delega. Los errores se
//! serializan como `String` legible para el frontend.

use crate::application::producto_service::ProductoService;
use crate::domain::producto::{Categoria, NuevoProducto, Producto};
use crate::infrastructure::db::DbConn;
use crate::infrastructure::db::repos::ProductoSqlite;
use crate::ports::producto_repo::CategoriaRepo;
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

/// Lista productos con filtro de texto, categoría y paginación.
#[tauri::command(rename_all = "snake_case")]
pub fn listar_productos(
    db: State<'_, DbConn>,
    filtro: String,
    categoria_id: Option<i64>,
    limite: i64,
    offset: i64,
) -> Result<Vec<Producto>, String> {
    // El frontend pide "limite/offset" (0-based); el service espera
    // "pagina/tam_pagina" (1-based).
    let (pagina, tam_pagina) = traducir_paginacion(offset, limite);
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .listar_productos(&filtro, categoria_id, pagina, tam_pagina)
        .map_err(|e| e.to_string())
}

/// Convierte `(offset, limite)` con base 0 a `(pagina, tam_pagina)` con base 1.
fn traducir_paginacion(offset: i64, limite: i64) -> (i64, i64) {
    let tam_pagina = limite.max(1);
    let pagina = if offset > 0 { offset / tam_pagina + 1 } else { 1 };
    (pagina, tam_pagina)
}

/// Busca un producto por código de barras (para el POS).
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

/// Baja lógica de un producto.
#[tauri::command]
pub fn eliminar_producto(db: State<'_, DbConn>, id: i64) -> Result<(), String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    ProductoService::new(repo)
        .eliminar_producto(id)
        .map_err(|e| e.to_string())
}

/// Ajuste manual de stock registrando el movimiento.
#[tauri::command(rename_all = "snake_case")]
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

/// Lista todas las categorías, ordenadas por nombre.
#[tauri::command]
pub fn listar_categorias(db: State<'_, DbConn>) -> Result<Vec<Categoria>, String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    repo.listar_categorias().map_err(|e| e.to_string())
}

/// Crea una categoría (nombre único).
#[tauri::command]
pub fn crear_categoria(
    db: State<'_, DbConn>,
    nueva: CategoriaNueva,
) -> Result<Categoria, String> {
    let repo = ProductoSqlite::new(db.inner().clone());
    repo.crear_categoria(&nueva.nombre, nueva.color.as_deref())
        .map_err(|e| e.to_string())
}

/// Comando de entrada para crear una categoría.
#[derive(serde::Deserialize)]
pub struct CategoriaNueva {
    pub nombre: String,
    pub color: Option<String>,
}
