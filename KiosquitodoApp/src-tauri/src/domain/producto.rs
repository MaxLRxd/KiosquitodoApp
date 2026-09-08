//! Entidad `Producto` y `Categoria`: validación de barcode, precios en centavos
//! y stock mínimo.
//!
//! Este módulo es **puro**: no conoce SQL, I/O ni Tauri. Solo expone tipos y
//! funciones de validación reutilizables por la capa de aplicación.

use serde::{Deserialize, Serialize};

/// Producto del inventario. Todos los montos en centavos (`i64`).
///
/// Alineado con la tabla `productos` del esquema SQLite y con el DTO
/// `Producto` del frontend (`src/lib/types/index.ts`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Producto {
    pub id: i64,
    pub nombre: String,
    pub barcode: Option<String>,
    pub precio_venta: i64,
    pub precio_costo: Option<i64>,
    pub stock: i64,
    pub stock_minimo: i64,
    pub activo: bool,
    pub categoria_id: Option<i64>,
}

/// Categoría de productos (Bebidas, Alfajores, etc.). Tabla `categorias`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Categoria {
    pub id: i64,
    pub nombre: String,
    pub color: Option<String>,
}

/// Datos para crear o actualizar un producto (sin id ni timestamps).
/// La capa de aplicación los valida antes de persistir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NuevoProducto {
    pub nombre: String,
    pub barcode: Option<String>,
    pub precio_venta: i64,
    pub precio_costo: Option<i64>,
    pub stock_inicial: i64,
    pub stock_minimo: i64,
    pub categoria_id: Option<i64>,
}

/// Verifica que un código de barras sea EAN-8 (8 dígitos), UPC-A (12)
/// o EAN-13 (13 dígitos), compuesto solo por dígitos.
pub fn validar_barcode(barcode: &str) -> bool {
    let limpio = barcode.trim();
    match limpio.len() {
        8 | 12 | 13 => limpio.chars().all(|c| c.is_ascii_digit()),
        _ => false,
    }
}

/// Reglas de precio: debe ser un monto positivo en centavos.
pub fn validar_precio(precio: i64) -> bool {
    precio > 0
}

/// Reglas de costo: puede no existir, pero si existe debe ser no negativo
/// (el costo puede ser $0, por ejemplo mercadería canjeada).
pub fn validar_costo(costo: Option<i64>) -> bool {
    match costo {
        None => true,
        Some(c) => c >= 0,
    }
}

/// Reglas de stock: el stock actual no puede ser negativo.
pub fn validar_stock(stock: i64) -> bool {
    stock >= 0
}

/// Reglas combinadas de stock mínimo: no puede ser negativo.
pub fn validar_stock_minimo(stock_minimo: i64) -> bool {
    stock_minimo >= 0
}

/// Estado de stock para la alerta visual del POS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoStock {
    /// stock > stock_minimo: sin alerta
    Ok,
    /// stock == 0: sin unidades disponibles
    Agotado,
    /// 0 < stock <= stock_minimo: por debajo del mínimo configurado
    Bajo,
}

/// Determina el estado de alerta de un producto.
pub fn estado_stock(producto: &Producto) -> EstadoStock {
    if producto.stock == 0 {
        EstadoStock::Agotado
    } else if producto.stock <= producto.stock_minimo {
        EstadoStock::Bajo
    } else {
        EstadoStock::Ok
    }
}

/// Valida un `NuevoProducto` completo. Retorna un mensaje con la primera
/// regla violada, o `Ok(())` si todas se cumplen.
pub fn validar_nuevo_producto(nuevo: &NuevoProducto) -> Result<(), &'static str> {
    if nuevo.nombre.trim().is_empty() {
        return Err("El nombre del producto no puede estar vacío");
    }
    if !validar_barcode(nuevo.barcode.as_deref().unwrap_or("")) {
        return Err("El código de barras debe tener 8, 12 o 13 dígitos");
    }
    if !validar_precio(nuevo.precio_venta) {
        return Err("El precio de venta debe ser mayor a cero (centavos)");
    }
    if !validar_costo(nuevo.precio_costo) {
        return Err("El precio de costo no puede ser negativo (centavos)");
    }
    if !validar_stock(nuevo.stock_inicial) {
        return Err("El stock inicial no puede ser negativo");
    }
    if !validar_stock_minimo(nuevo.stock_minimo) {
        return Err("El stock mínimo no puede ser negativo");
    }
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn producto_base() -> Producto {
        Producto {
            id: 1,
            nombre: "Coca-Cola 1.5L".to_string(),
            barcode: Some("7790001234567".to_string()),
            precio_venta: 3500,
            precio_costo: Some(2450),
            stock: 12,
            stock_minimo: 5,
            activo: true,
            categoria_id: Some(1),
        }
    }

    #[test]
    fn barcode_ean13_valido() {
        assert!(validar_barcode("7790001234567"));
    }

    #[test]
    fn barcode_ean8_valido() {
        assert!(validar_barcode("12345678"));
    }

    #[test]
    fn barcode_upc_valido() {
        assert!(validar_barcode("012345678905"));
    }

    #[test]
    fn barcode_con_letras_invalido() {
        assert!(!validar_barcode("77900AB12345"));
    }

    #[test]
    fn barcode_longitud_invalida() {
        assert!(!validar_barcode("12345"));
        assert!(!validar_barcode("1234567890123456"));
    }

    #[test]
    fn precio_centavos_positivo() {
        assert!(validar_precio(1));
        assert!(!validar_precio(0));
        assert!(!validar_precio(-100));
    }

    #[test]
    fn costo_opcional_no_negativo() {
        assert!(validar_costo(None));
        assert!(validar_costo(Some(0)));
        assert!(validar_costo(Some(2450)));
        assert!(!validar_costo(Some(-1)));
    }

    #[test]
    fn stock_no_negativo() {
        assert!(validar_stock(0));
        assert!(validar_stock(10));
        assert!(!validar_stock(-5));
    }

    #[test]
    fn estado_stock_valores() {
        let mut p = producto_base();
        p.stock = 6;
        assert_eq!(estado_stock(&p), EstadoStock::Ok);

        p.stock = 5; // igual al mínimo → Bajo
        assert_eq!(estado_stock(&p), EstadoStock::Bajo);

        p.stock = 1;
        assert_eq!(estado_stock(&p), EstadoStock::Bajo);

        p.stock = 0;
        assert_eq!(estado_stock(&p), EstadoStock::Agotado);
    }

    #[test]
    fn nuevo_producto_valido() {
        let nuevo = NuevoProducto {
            nombre: "Alfajor Jorgito".to_string(),
            barcode: Some("7790001234567".to_string()),
            precio_venta: 1500,
            precio_costo: Some(900),
            stock_inicial: 20,
            stock_minimo: config_default(),
            categoria_id: None,
        };
        assert!(validar_nuevo_producto(&nuevo).is_ok());
    }

    #[test]
    fn nuevo_producto_nombre_vacio() {
        let mut nuevo = nuevo_valido();
        nuevo.nombre = "   ".to_string();
        assert_eq!(
            validar_nuevo_producto(&nuevo),
            Err("El nombre del producto no puede estar vacío")
        );
    }

    #[test]
    fn nuevo_producto_precio_cero() {
        let mut nuevo = nuevo_valido();
        nuevo.precio_venta = 0;
        assert_eq!(
            validar_nuevo_producto(&nuevo),
            Err("El precio de venta debe ser mayor a cero (centavos)")
        );
    }

    fn nuevo_valido() -> NuevoProducto {
        NuevoProducto {
            nombre: "Agua 500ml".to_string(),
            barcode: Some("7790001234567".to_string()),
            precio_venta: 1200,
            precio_costo: Some(600),
            stock_inicial: 30,
            stock_minimo: 10,
            categoria_id: None,
        }
    }

    fn config_default() -> i64 {
        5
    }
}