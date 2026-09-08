//! Algoritmo de matching (±$1 / ±10 min) y estados OK / MONTO DIFIERE /
//! SIN COINCIDENCIA.
//!
//! Este módulo contiene la lógica pura de conciliación, reutilizable tanto
//! por el matching automático (SQL) como por la vista de la UI.

use crate::config::defaults;
use serde::{Deserialize, Serialize};

/// Estado de un registro de conciliación, tal como lo muestra la UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EstadoConciliacion {
    /// Monto y pago vinculado coinciden (diferencia <= 1 centavo, R8).
    Ok,
    /// Vinculado pero el monto difiere (> 1 centavo, R8).
    MontoDifiere,
    /// Sin pago de MP vinculado.
    SinCoincidencia,
}

/// Candidato de matcheo: una venta MP y un pago MP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidato {
    pub monto_venta: i64,
    pub monto_pago: i64,
    /// Diferencia de tiempo en segundos (positiva o negativa).
    pub delta_segundos: i64,
}

/// Regla de negocio para el estado de conciliación (R7 + R8).
///
/// - Si no hay pago vinculado → `SinCoincidencia`.
/// - Si hay pago con diferencia > 1 centavo → `MontoDifiere`.
/// - Si los montos coinciden (<= 1 centavo) → `Ok`.
pub fn estado_conciliacion(
    hay_pago_vinculado: bool,
    monto_venta: i64,
    monto_pago: Option<i64>,
) -> EstadoConciliacion {
    if !hay_pago_vinculado {
        return EstadoConciliacion::SinCoincidencia;
    }
    match monto_pago {
        Some(mp) if (monto_venta - mp).abs() <= 1 => EstadoConciliacion::Ok,
        Some(_) => EstadoConciliacion::MontoDifiere,
        None => EstadoConciliacion::SinCoincidencia,
    }
}

/// Regla R7: un candidato matchea si el monto coincide con tolerancia de
/// ±100 centavos Y el tiempo difiere en a lo sumo ±600 segundos (10 min).
pub fn es_candidato_valido(candidato: &Candidato) -> bool {
    let monto_ok = (candidato.monto_venta - candidato.monto_pago).abs()
        <= defaults::MATCHING_MONTO_TOLERANCIA;
    let tiempo_ok = candidato.delta_segundos.abs() <= defaults::MATCHING_VENTANA_SEGUNDOS;
    monto_ok && tiempo_ok
}

/// Deviation de un par no matcheado: cuánto se desvía del ±$1 permitido.
/// Útil para mostrar al operador "le faltan/faltaron $X" en la UI.
pub fn desviacion_monto(venta: i64, pago: i64) -> i64 {
    pago - venta
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn candidato(venta: i64, pago: i64, delta_seg: i64) -> Candidato {
        Candidato {
            monto_venta: venta,
            monto_pago: pago,
            delta_segundos: delta_seg,
        }
    }

    #[test]
    fn match_exacto() {
        assert!(es_candidato_valido(&candidato(1500, 1500, 120)));
    }

    #[test]
    fn match_dentro_tolerancia_monto() {
        // ±$1 = ±100 centavos
        assert!(es_candidato_valido(&candidato(1500, 1400, 0)));
        assert!(es_candidato_valido(&candidato(1500, 1600, 0)));
    }

    #[test]
    fn match_fuera_tolerancia_monto() {
        assert!(!es_candidato_valido(&candidato(1500, 1399, 0)));
        assert!(!es_candidato_valido(&candidato(1500, 1601, 0)));
    }

    #[test]
    fn match_dentro_ventana_tiempo() {
        // ±10 min = ±600 seg
        assert!(es_candidato_valido(&candidato(1500, 1500, 600)));
        assert!(es_candidato_valido(&candidato(1500, 1500, -600)));
    }

    #[test]
    fn match_fuera_ventana_tiempo() {
        assert!(!es_candidato_valido(&candidato(1500, 1500, 601)));
        assert!(!es_candidato_valido(&candidato(1500, 1500, -601)));
    }

    #[test]
    fn requiere_ambas_condiciones() {
        // Mismo monto pero fuera de tiempo
        assert!(!es_candidato_valido(&candidato(1500, 1500, 700)));
        // En tiempo pero monto muy distinto
        assert!(!es_candidato_valido(&candidato(1500, 2000, 0)));
    }

    #[test]
    fn estado_sin_pago() {
        assert_eq!(
            estado_conciliacion(false, 1500, None),
            EstadoConciliacion::SinCoincidencia
        );
    }

    #[test]
    fn estado_ok() {
        assert_eq!(
            estado_conciliacion(true, 1500, Some(1500)),
            EstadoConciliacion::Ok
        );
        // 1 centavo de diferencia es OK (R8)
        assert_eq!(
            estado_conciliacion(true, 1500, Some(1501)),
            EstadoConciliacion::Ok
        );
    }

    #[test]
    fn estado_monto_difiere() {
        assert_eq!(
            estado_conciliacion(true, 1500, Some(1502)),
            EstadoConciliacion::MontoDifiere
        );
        assert_eq!(
            estado_conciliacion(true, 1500, Some(1200)),
            EstadoConciliacion::MontoDifiere
        );
    }

    #[test]
    fn desviacion_monto_muestra_diferencia() {
        assert_eq!(desviacion_monto(1500, 1600), 100);
        assert_eq!(desviacion_monto(1500, 1400), -100);
    }
}