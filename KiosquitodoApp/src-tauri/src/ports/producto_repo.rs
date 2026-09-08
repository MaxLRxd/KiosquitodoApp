//! Trait del repositorio de productos y categorías.

use crate::domain::producto::{NuevoProducto, Producto};
use crate::errors::AppResult;

/// Contrato de persistencia para el inventario.
///
/// Las implementaciones concretas (SQLite, mocks de tests) viven en
/// infraestructura. La capa de aplicación depende solo de este trait.
pub trait ProductoRepo: Send + Sync {
    /// Busca un producto activo por código de barras exacto.
    fn buscar_por_barcode(&self, barcode: &str) -> AppResult<Option<Producto>>;

    /// Busca un producto por id.
    fn buscar_por_id(&self, id: i64) -> AppResult<Option<Producto>>;

    /// Lista productos paginados. `filtro` busca por nombre parcial o barcode;
    /// `categoria_id` puede filtrar por categoría (None = todas).
    fn listar_productos(
        &self,
        filtro: &str,
        categoria_id: Option<i64>,
        limite: i64,
        offset: i64,
    ) -> AppResult<Vec<Producto>>;

    /// Crea un producto y devuelve el persistido (con id asignado).
    fn crear_producto(&self, nuevo: &NuevoProducto) -> AppResult<Producto>;

    /// Actualiza los campos editables de un producto existente.
    fn actualizar_producto(&self, producto: &Producto) -> AppResult<()>;

    /// Elimina un producto (baja lógica: `activo = 0`).
    fn eliminar_producto(&self, id: i64) -> AppResult<()>;

    /// Ajusta el stock manualmente y registra el movimiento (recuento, rotura,
    /// vencimiento, error de carga, otro).
    fn ajustar_stock(
        &self,
        id: i64,
        stock_nuevo: i64,
        motivo: &str,
        operador: Option<&str>,
    ) -> AppResult<()>;

    /// Devuelve los `(producto_id, precio_costo)` de un conjunto de productos
    /// (para el cálculo de margen bruto).
    fn costos_de_productos(&self, ids: &[i64]) -> AppResult<Vec<(i64, i64)>>;
}