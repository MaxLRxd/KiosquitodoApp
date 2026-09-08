//! Trait del repositorio de pagos MP importados.

use crate::domain::pago::PagoMp;
use crate::errors::AppResult;

/// Contrato de persistencia de los pagos importados de la API de MP.
pub trait PagoRepo: Send + Sync {
    /// Inserta un pago de forma idempotente (`INSERT OR IGNORE` sobre
    /// `mp_id` único, R5). Devuelve `true` si fue insertado, `false` si ya
    /// existía.
    fn upsert_pago(&self, pago: &PagoMp) -> AppResult<bool>;

    /// Lista los pagos importados en un rango de fechas (`YYYY-MM-DDTHH:MM:SS`).
    fn pagos_en_rango(&self, desde: &str, hasta: &str) -> AppResult<Vec<PagoMp>>;

    /// Busca un pago por su `mp_id` natural.
    fn obtener_pago_por_mp_id(&self, mp_id: &str) -> AppResult<Option<PagoMp>>;

    /// Vincula un pago importado a una venta (conciliación manual), R7.
    fn vincular_pago_a_venta(&self, mp_id: &str, venta_id: i64) -> AppResult<()>;

    /// Matching automático (R7): vincula los pagos huérfanos con ventas MP de
    /// monto ±100 centavos dentro de ±10 minutos. Devuelve cuántos se vincularon.
    fn aplicar_matching_automatico(&self) -> AppResult<usize>;

    /// Verifica si un pago ya existe (para decidir si mostrar "nuevo").
    fn existe_pago(&self, mp_id: &str) -> AppResult<bool>;
}