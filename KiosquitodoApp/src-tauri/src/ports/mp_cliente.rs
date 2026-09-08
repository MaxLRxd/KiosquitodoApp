//! Trait del cliente HTTP de la API de Mercado Pago.
//!
//! Es la única abstracción que conoce "algo remoto": la implementación real
//! usa reqwest; los tests pueden usar un mock local.

use crate::domain::pago::PagoMp;
use crate::errors::AppResult;

/// Contrato del cliente de la API de MP.
///
/// `desde`/`hasta` son timestamps ISO 8601 (ej: `NOW-1DAYS` o
/// `2026-09-07T10:30:00.000-03:00`). El cliente filtra `approved` +
/// `accredited`, pagina automáticamente y convierte a centavos.
pub trait MpCliente: Send + Sync {
    /// Trae todos los pagos elegibles en el rango, paginando internamente.
    fn buscar_pagos(
        &self,
        access_token: &str,
        desde: &str,
        hasta: &str,
    ) -> impl std::future::Future<Output = AppResult<Vec<PagoMp>>> + Send;
}