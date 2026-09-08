//! Trait del repositorio de ventas, ítems y devoluciones.

use crate::domain::venta::{NuevaVenta, Venta, VentaItem};
use crate::errors::AppResult;

/// Contrato de persistencia del agregado `Venta` y sus devoluciones.
pub trait VentaRepo: Send + Sync {
    /// Crea la venta en una **única transacción**: inserta la cabecera,
    /// los ítems y descuenta el stock de cada producto (R9).
    /// Si algo falla, se revierte todo.
    fn crear_venta_transaccional(&self, venta: &NuevaVenta) -> AppResult<Venta>;

    /// Obtiene una venta completa (con ítems) por su id.
    fn obtener_venta(&self, id: i64) -> AppResult<Option<Venta>>;

    /// Lista las ventas de una fecha (`YYYY-MM-DD`), más recientes primero.
    fn listar_ventas_dia(&self, fecha: &str) -> AppResult<Vec<Venta>>;

    /// Obtiene los ítems de una venta.
    fn obtener_items_venta(&self, venta_id: i64) -> AppResult<Vec<VentaItem>>;

    /// Registra una devolución: incrementa stock de los ítems devueltos y
    /// guarda cabecera + ítems de la devolución. Transaccional.
    fn crear_devolucion(
        &self,
        venta_id: i64,
        items: &[VentaItem],
        motivo: Option<&str>,
    ) -> AppResult<()>;
}