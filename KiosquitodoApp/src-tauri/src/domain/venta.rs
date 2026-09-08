//! Agregado `Venta`: ítems, medios de pago (efectivo/MP/mixto), totales en
//! centavos y devoluciones.
//!
//! Módulo **puro**: no conoce SQL ni I/O. El total se calcula siempre a partir
//! de los ítems para que el estado no pueda desincronizarse.

use serde::{Deserialize, Serialize};

/// Medio de pago de una venta. Serializa en `snake_case` para el IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MedioPago {
    Efectivo,
    MercadoPago,
    Mixto,
}

impl MedioPago {
    /// ¿La venta involucra Mercado Pago? (para sync + conciliación)
    pub fn involucra_mp(self) -> bool {
        matches!(self, Self::MercadoPago | Self::Mixto)
    }

    /// ¿La venta involucra efectivo?
    pub fn involucra_efectivo(self) -> bool {
        matches!(self, Self::Efectivo | Self::Mixto)
    }
}

/// Ítem de una venta. `precio_unitario` en centavos (capturado al momento
/// de la venta, inmutable ante cambios posteriores de precio del producto).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VentaItem {
    pub producto_id: i64,
    pub cantidad: i64,
    pub precio_unitario: i64,
}

impl VentaItem {
    /// Subtotal del ítem en centavos (satura en i64::MAX implícitamente;
    /// cantidades y precios reales nunca se acercan al límite).
    pub fn subtotal(&self) -> i64 {
        self.cantidad.saturating_mul(self.precio_unitario)
    }
}

/// Venta completa (agregado raíz). `total` se deriva de los ítems y se
/// persiste como denormalización para queries rápidas de conciliación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Venta {
    pub id: i64,
    pub items: Vec<VentaItem>,
    pub total: i64,
    pub medio_pago: MedioPago,
    pub monto_efectivo: i64,
    pub monto_mp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descripcion_mp: Option<String>,
    pub hora: String,
}

/// Comando de entrada para crear una venta (sin id ni hora, los asigna el repo).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NuevaVenta {
    pub items: Vec<VentaItem>,
    pub medio_pago: MedioPago,
    pub monto_efectivo: i64,
    pub monto_mp: i64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descripcion_mp: Option<String>,
}

/// Calcula el total de una venta en centavos: Σ cantidad × precio_unitario.
pub fn calcular_total(items: &[VentaItem]) -> i64 {
    items.iter().map(VentaItem::subtotal).sum()
}

/// Valida la coherencia de una venta antes de persistirla.
///
/// Reglas:
/// 1. Debe tener al menos un ítem.
/// 2. Ningún ítem puede tener cantidad menor o igual a cero.
/// 3. El precio unitario debe ser positivo (o coherente en devoluciones).
/// 4. Los montos parciales deben coincidir con el medio de pago y cuadrar el total.
pub fn validar_venta(
    items: &[VentaItem],
    medio_pago: MedioPago,
    monto_efectivo: i64,
    monto_mp: i64,
) -> Result<(), &'static str> {
    if items.is_empty() {
        return Err("La venta debe tener al menos un ítem");
    }
    if items.iter().any(|i| i.cantidad <= 0) {
        return Err("La cantidad de cada ítem debe ser mayor a cero");
    }
    if items.iter().any(|i| i.precio_unitario < 0) {
        return Err("El precio unitario no puede ser negativo");
    }

    let total = calcular_total(items);

    match medio_pago {
        MedioPago::Efectivo => {
            if monto_mp != 0 {
                return Err("Una venta en efectivo no puede tener monto de Mercado Pago");
            }
            if monto_efectivo != total {
                return Err("El monto en efectivo debe coincidir con el total de la venta");
            }
        }
        MedioPago::MercadoPago => {
            if monto_efectivo != 0 {
                return Err("Una venta con Mercado Pago no puede tener monto en efectivo");
            }
            if monto_mp != total {
                return Err("El monto de Mercado Pago debe coincidir con el total de la venta");
            }
        }
        MedioPago::Mixto => {
            if monto_efectivo <= 0 || monto_mp <= 0 {
                return Err("En una venta mixta ambos montos deben ser mayores a cero");
            }
            if monto_efectivo + monto_mp != total {
                return Err("Efectivo + Mercado Pago debe coincidir con el total de la venta");
            }
        }
    }

    Ok(())
}

/// Calcula el margen bruto de una venta (Σ (precio_venta − precio_costo) × cantidad).
/// `costos` es un mapa `producto_id → precio_costo`; los productos sin costo
/// registrado aportan 0 al margen.
pub fn calcular_margen(items: &[VentaItem], costos: &[(i64, i64)]) -> i64 {
    let costo_por_producto: std::collections::HashMap<i64, i64> =
        costos.iter().copied().collect();

    items
        .iter()
        .map(|i| {
            let costo_unitario = costo_por_producto.get(&i.producto_id).copied().unwrap_or(0);
            (i.precio_unitario - costo_unitario).saturating_mul(i.cantidad)
        })
        .sum()
}

/// Regla de conciliación (SPEC R8): dos montos no difieren si la diferencia
/// absoluta es <= 1 centavo (ruido de redondeo tolerado).
pub fn montos_coinciden(a: i64, b: i64) -> bool {
    (a - b).abs() <= 1
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn item(producto_id: i64, cantidad: i64, precio_unitario: i64) -> VentaItem {
        VentaItem {
            producto_id,
            cantidad,
            precio_unitario,
        }
    }

    #[test]
    fn subtotal_calculado() {
        assert_eq!(item(1, 3, 500).subtotal(), 1500);
    }

    #[test]
    fn total_suma_items() {
        let items = vec![item(1, 1, 1000), item(2, 2, 500), item(3, 4, 250)];
        assert_eq!(calcular_total(&items), 3000);
    }

    #[test]
    fn medio_pago_flags() {
        assert!(MedioPago::MercadoPago.involucra_mp());
        assert!(MedioPago::Mixto.involucra_mp());
        assert!(!MedioPago::Efectivo.involucra_mp());
        assert!(MedioPago::Efectivo.involucra_efectivo());
        assert!(MedioPago::Mixto.involucra_efectivo());
        assert!(!MedioPago::MercadoPago.involucra_efectivo());
    }

    #[test]
    fn venta_efectivo_valida() {
        let items = vec![item(1, 2, 500)];
        assert!(validar_venta(&items, MedioPago::Efectivo, 1000, 0).is_ok());
    }

    #[test]
    fn venta_efectivo_monto_incorrecto() {
        let items = vec![item(1, 2, 500)];
        assert_eq!(
            validar_venta(&items, MedioPago::Efectivo, 900, 0),
            Err("El monto en efectivo debe coincidir con el total de la venta")
        );
    }

    #[test]
    fn venta_mp_valida() {
        let items = vec![item(1, 1, 1500)];
        assert!(validar_venta(&items, MedioPago::MercadoPago, 0, 1500).is_ok());
    }

    #[test]
    fn venta_mp_con_efectivo_invalido() {
        let items = vec![item(1, 1, 1500)];
        assert_eq!(
            validar_venta(&items, MedioPago::MercadoPago, 100, 1400),
            Err("Una venta con Mercado Pago no puede tener monto en efectivo")
        );
    }

    #[test]
    fn venta_mixta_valida() {
        let items = vec![item(1, 1, 2000)];
        assert!(validar_venta(&items, MedioPago::Mixto, 500, 1500).is_ok());
    }

    #[test]
    fn venta_mixta_no_cuadra() {
        let items = vec![item(1, 1, 2000)];
        assert_eq!(
            validar_venta(&items, MedioPago::Mixto, 500, 1200),
            Err("Efectivo + Mercado Pago debe coincidir con el total de la venta")
        );
    }

    #[test]
    fn venta_sin_items_invalida() {
        assert_eq!(
            validar_venta(&[], MedioPago::Efectivo, 0, 0),
            Err("La venta debe tener al menos un ítem")
        );
    }

    #[test]
    fn venta_cantidad_cero_invalida() {
        let items = vec![item(1, 0, 500)];
        assert!(validar_venta(&items, MedioPago::Efectivo, 0, 0).is_err());
    }

    #[test]
    fn venta_mixta_montos_tienen_que_ser_positivos() {
        let items = vec![item(1, 1, 1000)];
        // medio mixto pero un monto en cero → inválido
        assert_eq!(
            validar_venta(&items, MedioPago::Mixto, 0, 1000),
            Err("En una venta mixta ambos montos deben ser mayores a cero")
        );
    }

    #[test]
    fn margen_con_costos() {
        let items = vec![item(1, 2, 500), item(2, 1, 1000)];
        let costos = vec![(1, 300), (2, 700)];
        // (500-300)*2 + (1000-700)*1 = 400 + 300 = 700
        assert_eq!(calcular_margen(&items, &costos), 700);
    }

    #[test]
    fn margen_sin_costo_aporta_cero() {
        let items = vec![item(1, 3, 500)];
        // Producto 1 no tiene costo → margen = (500-0)*3
        assert_eq!(calcular_margen(&items, &[]), 1500);
    }

    #[test]
    fn montos_iguales_por_regla_r8() {
        assert!(montos_coinciden(1500, 1500));
        assert!(montos_coinciden(1501, 1500)); // 1 centavo de diferencia: OK
        assert!(!montos_coinciden(1502, 1500)); // 2 centavos: difiere
    }
}