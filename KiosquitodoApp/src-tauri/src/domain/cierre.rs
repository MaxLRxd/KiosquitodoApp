//! Cierre de caja: totales del día, margen bruto y delta de conciliación MP.
//!
//! Módulo **puro**: calcula el resumen diario a partir de ventas y pagos
//! ya traídos de la persistencia, sin conocer SQL.

use serde::{Deserialize, Serialize};

/// Resumen del día para el cierre de caja (Módulo 4 del SPEC).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumenCierre {
    /// Suma de montos en efectivo del día (centavos).
    pub total_efectivo: i64,
    /// Suma de montos con MP del día (centavos).
    pub total_mp: i64,
    /// Suma de todos los montos del día (centavos).
    pub total_ventas: i64,
    /// Cantidad de transacciones del día.
    pub cantidad_ventas: i64,
    /// Margen bruto estimado (precio_venta − precio_costo).
    pub margen_bruto: i64,
    /// Delta de conciliación: Σ mp_pagos.monto − Σ ventas.monto_mp.
    /// Positivo = la API muestra más de lo registrado.
    pub delta_mp: i64,
}

impl ResumenCierre {
    pub fn nuevo(
        total_efectivo: i64,
        total_mp: i64,
        total_ventas: i64,
        cantidad_ventas: i64,
        margen_bruto: i64,
        delta_mp: i64,
    ) -> Self {
        Self {
            total_efectivo,
            total_mp,
            total_ventas,
            cantidad_ventas,
            margen_bruto,
            delta_mp,
        }
    }
}

/// Aporte de una venta individual al resumen del día.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AporteVenta {
    pub monto_efectivo: i64,
    pub monto_mp: i64,
    pub margen_bruto: i64,
    pub involucra_mp: bool,
}

/// Aporte de un pago MP importado al resumen del día.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AportePagoMp {
    pub monto: i64,
}

/// Agrega los aportes de ventas y pagos a un resumen parcial.
/// El delta MP se calcula comparando lo registrado `vs` lo acreditado según API.
pub fn calcular_resumen(
    ventas: &[AporteVenta],
    pagos_mp: &[AportePagoMp],
) -> ResumenCierre {
    let mut efectivo = 0i64;
    let mut mp = 0i64;
    let mut total = 0i64;
    let mut cantidad = 0i64;
    let mut margen = 0i64;

    for v in ventas {
        efectivo += v.monto_efectivo;
        mp += v.monto_mp;
        total += v.monto_efectivo + v.monto_mp;
        cantidad += 1;
        margen += v.margen_bruto;
    }

    // Delta = Σ montos acreditados según API − Σ montos MP registrados.
    // Solo las ventas que involucran MP se restan del lado registrado.
    let monto_mp_acreditado: i64 = pagos_mp.iter().map(|p| p.monto).sum();
    let monto_mp_registrado: i64 = ventas
        .iter()
        .filter(|v| v.involucra_mp)
        .map(|v| v.monto_mp)
        .sum();
    let delta_mp = monto_mp_acreditado - monto_mp_registrado;

    ResumenCierre::nuevo(efectivo, mp, total, cantidad, margen, delta_mp)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn venta(efectivo: i64, mp: i64, margen: i64) -> AporteVenta {
        AporteVenta {
            monto_efectivo: efectivo,
            monto_mp: mp,
            margen_bruto: margen,
            involucra_mp: mp > 0,
        }
    }

    #[test]
    fn resumen_con_ventas_mixtas_y_efectivo() {
        let ventas = vec![venta(1000, 0, 400), venta(500, 1500, 700)];
        let resumen = calcular_resumen(&ventas, &[]);

        assert_eq!(resumen.total_efectivo, 1500);
        assert_eq!(resumen.total_mp, 1500);
        assert_eq!(resumen.total_ventas, 3000);
        assert_eq!(resumen.cantidad_ventas, 2);
        assert_eq!(resumen.margen_bruto, 1100);
    }

    #[test]
    fn delta_mp_cero_cuando_cuadra() {
        let ventas = vec![venta(0, 1500, 500)];
        let pagos = vec![AportePagoMp { monto: 1500 }];
        let resumen = calcular_resumen(&ventas, &pagos);
        assert_eq!(resumen.delta_mp, 0);
    }

    #[test]
    fn delta_mp_positivo_cuando_api_muestra_mas() {
        let ventas = vec![venta(0, 1500, 500)];
        let pagos = vec![
            AportePagoMp { monto: 1500 },
            AportePagoMp { monto: 800 }, // pago huérfano según API
        ];
        let resumen = calcular_resumen(&ventas, &pagos);
        assert_eq!(resumen.delta_mp, 800);
    }

    #[test]
    fn delta_mp_negativo_cuando_falta_acreditar() {
        let ventas = vec![venta(0, 1500, 500)];
        let pagos = vec![AportePagoMp { monto: 1200 }];
        let resumen = calcular_resumen(&ventas, &pagos);
        assert_eq!(resumen.delta_mp, -300);
    }

    #[test]
    fn resumen_vacia() {
        let resumen = calcular_resumen(&[], &[]);
        assert_eq!(resumen.total_ventas, 0);
        assert_eq!(resumen.cantidad_ventas, 0);
        assert_eq!(resumen.delta_mp, 0);
    }

    #[test]
    fn venta_efectivo_no_afecta_delta() {
        let ventas = vec![venta(2000, 0, 800)];
        let pagos = vec![AportePagoMp { monto: 2000 }];
        // El pago de la API de 2000 no debe contrarestar la venta en efectivo
        let resumen = calcular_resumen(&ventas, &pagos);
        assert_eq!(resumen.delta_mp, 2000);
    }
}