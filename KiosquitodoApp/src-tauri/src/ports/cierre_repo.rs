//! Trait del repositorio de cierres de caja y movimientos de stock.

use crate::domain::cierre::ResumenCierre;
use crate::errors::AppResult;

/// Cierre persistido en la tabla `cierres_caja`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CierrePersistido {
    pub id: i64,
    pub fecha: String,
    pub resumen: ResumenCierre,
}

/// Contrato de persistencia de cierres de caja.
pub trait CierreRepo: Send + Sync {
    /// Calcula el resumen diario (totales, margen, delta MP) para una fecha
    /// `YYYY-MM-DD` consultando ventas, ítems y pagos.
    fn resumen_dia(&self, fecha: &str) -> AppResult<ResumenCierre>;

    /// Persiste un cierre. Devuelve el id asignado.
    fn guardar_cierre(&self, fecha: &str, resumen: &ResumenCierre) -> AppResult<i64>;

    /// Lista los cierres anteriores, más reciente primero.
    fn obtener_cierres(&self) -> AppResult<Vec<CierrePersistido>>;
}