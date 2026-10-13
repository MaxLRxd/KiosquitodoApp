//! Pago importado de Mercado Pago: elegibilidad (approved + accredited) y
//! dedupe por `mp_id`.
//!
//! Módulo **puro**: la conversión de decimales (API) a centavos y el filtro de
//! elegibilidad ocurren aquí, a la entrada del sistema.

use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Pago importado de la API de Mercado Pago. `monto` siempre en centavos.
///
/// Alineado con la tabla `mp_pagos` y el DTO `MpPago` del frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagoMp {
    /// Id local de la tabla `mp_pagos` (0 para pagos recién importados).
    #[serde(default)]
    pub id: i64,
    /// ID único del pago en MP (clave de idempotencia, R5).
    pub mp_id: String,
    /// Timestamp de aprobación **normalizado a hora local naive**
    /// (`YYYY-MM-DDTHH:MM:SS`, opcional si la API no lo manda).
    ///
    /// Se convierte desde el ISO 8601 con offset que devuelve MP (ver
    /// [`normalizar_fecha_local`]) para que coincida con `ventas.hora`.
    pub fecha_aprobacion: Option<String>,
    /// Monto en centavos (`(transaction_amount * 100.0).round() as i64`, R6).
    pub monto: i64,
    /// Descripción que ingresó el pagador.
    pub descripcion: Option<String>,
    /// Venta vinculada (null durante la importación inicial).
    pub venta_id: Option<i64>,
}

/// Respuesta cruda de la API de MP para un pago individual.
/// Solo los campos necesarios para la importación.
#[derive(Debug, Clone, Deserialize)]
pub struct MpPagoApi {
    pub id: u64,
    #[serde(default)]
    pub date_approved: Option<String>,
    pub transaction_amount: f64,
    #[serde(default)]
    pub description: Option<String>,
    pub status: String,
    pub status_detail: String,
}

/// Regla R5: solo son importables los pagos `approved` **y** `accredited`.
/// `status = approved` no garantiza que el dinero esté acreditado.
pub fn es_elegible(status: &str, status_detail: &str) -> bool {
    status == "approved" && status_detail == "accredited"
}

/// Regla R6: convierte un monto decimal (f64 de la API) a centavos con
/// redondeo para absorber ruido de punto flotante (1500.0000001 → 1500).
pub fn a_centavos(transaction_amount: f64) -> i64 {
    (transaction_amount * 100.0).round() as i64
}

/// Normaliza el timestamp de aprobación de MP (ISO 8601 con offset, ej.
/// `2026-09-07T10:30:00.000-03:00`) al formato **local naive** que usa la
/// columna `ventas.hora` (`YYYY-MM-DDTHH:MM:SS`).
///
/// Es crucial para el matching de conciliación (R7): sin normalizar, SQLite
/// interpreta el texto con offset como UTC y la hora naive como local, y
/// `strftime('%s', ...)` los compara desplazados por el offset de la zona
/// (ej. −03:00 en Argentina). Se conserva el separador `T` para no romper las
/// comparaciones de rango (`pagos_en_rango`).
///
/// Idempotente: una fecha ya normalizada se devuelve igual. Si el texto no se
/// puede parsear, se devuelve sin cambios.
pub fn normalizar_fecha_local(fecha: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(fecha) {
        return dt
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(fecha, "%Y-%m-%dT%H:%M:%S%.f") {
        return naive.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    fecha.to_string()
}

/// Convierte y filtra una respuesta de la API en un `PagoMp` elegible.
/// Devuelve `None` si el pago no cumple `approved` + `accredited`.
pub fn desde_api(pago: MpPagoApi) -> Option<PagoMp> {
    if !es_elegible(&pago.status, &pago.status_detail) {
        return None;
    }
    Some(PagoMp {
        id: 0,
        mp_id: pago.id.to_string(),
        fecha_aprobacion: pago.date_approved.as_deref().map(normalizar_fecha_local),
        monto: a_centavos(pago.transaction_amount),
        descripcion: pago.description,
        venta_id: None,
    })
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pago_api(id: u64, monto: f64, status: &str, status_detail: &str) -> MpPagoApi {
        MpPagoApi {
            id,
            date_approved: Some("2026-09-07T10:30:00.000-03:00".to_string()),
            transaction_amount: monto,
            description: Some("Compra kiosco".to_string()),
            status: status.to_string(),
            status_detail: status_detail.to_string(),
        }
    }

    #[test]
    fn elegible_solo_approved_y_accredited() {
        assert!(es_elegible("approved", "accredited"));
        assert!(!es_elegible("approved", "pending"));
        assert!(!es_elegible("pending", "accredited"));
        assert!(!es_elegible("rejected", "accredited"));
    }

    #[test]
    fn conversion_a_centavos_redondea() {
        assert_eq!(a_centavos(1500.0), 150000);
        assert_eq!(a_centavos(1500.0000001), 150000);
        assert_eq!(a_centavos(1.99), 199);
        assert_eq!(a_centavos(0.005), 1); // 0.5 centavos → redondea a 1
    }

    #[test]
    fn desde_api_elegible() {
        let p = desde_api(pago_api(42, 15.00, "approved", "accredited")).unwrap();
        assert_eq!(p.mp_id, "42");
        assert_eq!(p.monto, 1500);
        assert_eq!(p.venta_id, None);
    }

    #[test]
    fn desde_api_rechaza_no_acreditado() {
        assert!(desde_api(pago_api(43, 15.00, "approved", "pending")).is_none());
        assert!(desde_api(pago_api(44, 15.00, "rejected", "accredited")).is_none());
    }

    #[test]
    fn descripcion_y_fecha_opcionales() {
        let mut api = pago_api(45, 10.0, "approved", "accredited");
        api.description = None;
        api.date_approved = None;
        let p = desde_api(api).unwrap();
        assert_eq!(p.descripcion, None);
        assert_eq!(p.fecha_aprobacion, None);
    }

    #[test]
    fn normalizar_convierte_offset_a_local_naive() {
        // El mismo instante con offsets distintos debe dar el mismo texto local.
        let con_offset = normalizar_fecha_local("2026-09-07T10:30:00.000-03:00");
        let en_utc = normalizar_fecha_local("2026-09-07T13:30:00.000+00:00");
        assert_eq!(con_offset, en_utc);
        // Formato `YYYY-MM-DDTHH:MM:SS` (19 chars, sin offset ni milisegundos).
        assert_eq!(con_offset.len(), 19);
        assert!(NaiveDateTime::parse_from_str(&con_offset, "%Y-%m-%dT%H:%M:%S").is_ok());
    }

    #[test]
    fn normalizar_es_idempotente_y_tolera_texto_invalido() {
        assert_eq!(
            normalizar_fecha_local("2026-09-07T10:30:00"),
            "2026-09-07T10:30:00"
        );
        assert_eq!(normalizar_fecha_local("no-es-fecha"), "no-es-fecha");
    }
}