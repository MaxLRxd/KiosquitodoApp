//! Casos de uso de POS: registrar venta (validaciÃ³n de stock y medios mixtos)
//! y devoluciones.
//!
//! El servicio valida la coherencia del pedido (dominio) y que exista stock
//! antes de pedirle la transacciÃ³n atÃ³mica al `VentaRepo`. La garantÃ­a final
//! de consistencia (R9) la impone el repo dentro de su transacciÃ³n.

use crate::domain::venta::{calcular_total, validar_venta, NuevaVenta, Venta, VentaItem};
use crate::errors::{AppError, AppResult};
use crate::ports::producto_repo::ProductoRepo;
use crate::ports::venta_repo::VentaRepo;

/// Casos de uso de ventas. Requiere tanto el repo de ventas como el de
/// productos (las ventas descuentan stock, R9).
pub struct VentaService<V: VentaRepo, P: ProductoRepo> {
    venta_repo: V,
    producto_repo: P,
}

impl<V: VentaRepo, P: ProductoRepo> VentaService<V, P> {
    pub fn new(venta_repo: V, producto_repo: P) -> Self {
        Self {
            venta_repo,
            producto_repo,
        }
    }

    /// Registra una venta: valida reglas del dominio, verifica stock y delega
    /// la transacciÃ³n atÃ³mica (cabecera + Ã­tems + descuento de stock).
    pub fn registrar_venta(&self, venta: &NuevaVenta) -> AppResult<Venta> {
        validar_venta(
            &venta.items,
            venta.medio_pago,
            venta.monto_efectivo,
            venta.monto_mp,
        )
        .map_err(AppError::from)?;

        self.verificar_stock(&venta.items)?;
        self.venta_repo.crear_venta_transaccional(venta)
    }

    /// Obtiene una venta completa (con Ã­tems) por id.
    pub fn obtener_venta(&self, id: i64) -> AppResult<Option<Venta>> {
        self.venta_repo.obtener_venta(id)
    }

    /// Lista las ventas de un dÃ­a (`YYYY-MM-DD`), mÃ¡s recientes primero.
    pub fn listar_ventas_dia(&self, fecha: &str) -> AppResult<Vec<Venta>> {
        self.venta_repo.listar_ventas_dia(fecha)
    }

    /// Obtiene los Ã­tems de una venta.
    pub fn obtener_items_venta(&self, venta_id: i64) -> AppResult<Vec<VentaItem>> {
        self.venta_repo.obtener_items_venta(venta_id)
    }

    /// Registra una devoluciÃ³n sobre una venta. Recupera el stock de los
    /// Ã­tems devueltos (transaccional en el repo).
    pub fn registrar_devolucion(
        &self,
        venta_id: i64,
        items: &[VentaItem],
        motivo: Option<&str>,
    ) -> AppResult<()> {
        if !matches!(self.venta_repo.obtener_venta(venta_id)?, Some(_)) {
            return Err(AppError::negocio("La venta no existe"));
        }
        if items.is_empty() {
            return Err(AppError::negocio(
                "La devoluciÃ³n debe tener al menos un Ã­tem",
            ));
        }
        if items.iter().any(|i| i.cantidad <= 0) {
            return Err(AppError::negocio(
                "La cantidad devuelta de cada Ã­tem debe ser mayor a cero",
            ));
        }
        self.venta_repo.crear_devolucion(venta_id, items, motivo)
    }

    /// Verifica que exista stock suficiente para cada Ã­tem. Es una validaciÃ³n
    /// temprana (UX), la transacciÃ³n del repo sigue siendo la barrera final.
    fn verificar_stock(&self, items: &[VentaItem]) -> AppResult<()> {
        for item in items {
            let producto = self
                .producto_repo
                .buscar_por_id(item.producto_id)?
                .ok_or_else(|| AppError::negocio("Uno de los productos no existe"))?;
            if producto.stock < item.cantidad {
                return Err(AppError::negocio(format!(
                    "Stock insuficiente para {} (disponible: {}, pedido: {})",
                    producto.nombre, producto.stock, item.cantidad
                )));
            }
        }
        Ok(())
    }

    /// Recalcula el total desde los Ã­tems (consistencia entre DTO y dominio).
    pub fn total_venta(&self, items: &[VentaItem]) -> i64 {
        calcular_total(items)
    }
}

// â”€â”€ Tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::producto::{NuevoProducto, Producto};
    use crate::domain::venta::MedioPago;
    use crate::ports::producto_repo::ProductoRepo;
    use std::sync::{Arc, Mutex};

    /// Mock del repo de productos (mÃ­nimo: id, stock y bÃºsqueda por id).
    struct MockProductos {
        productos: Mutex<Vec<Producto>>,
        siguiente_id: Mutex<i64>,
    }

    impl MockProductos {
        fn nuevo() -> Self {
            Self {
                productos: Mutex::new(Vec::new()),
                siguiente_id: Mutex::new(1),
            }
        }

        fn agregar(&self, nombre: &str, stock: i64) -> i64 {
            let mut sig = self.siguiente_id.lock().unwrap();
            let id = *sig;
            *sig += 1;
            let p = Producto {
                id,
                nombre: nombre.to_string(),
                barcode: Some("7790000000011".to_string()),
                precio_venta: 1000,
                precio_costo: Some(500),
                stock,
                stock_minimo: 0,
                activo: true,
                categoria_id: None,
            };
            self.productos.lock().unwrap().push(p);
            id
        }

        fn descontar(&self, id: i64, cantidad: i64) {
            if let Some(p) = self.productos.lock().unwrap().iter_mut().find(|p| p.id == id) {
                p.stock -= cantidad;
            }
        }
    }

    impl ProductoRepo for Arc<MockProductos> {
        fn buscar_por_barcode(&self, barcode: &str) -> AppResult<Option<Producto>> {
            Ok(self
                .productos
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.activo && p.barcode.as_deref() == Some(barcode))
                .cloned())
        }

        fn buscar_por_id(&self, id: i64) -> AppResult<Option<Producto>> {
            Ok(self
                .productos
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.activo && p.id == id)
                .cloned())
        }

        fn listar_productos(
            &self,
            _filtro: &str,
            _categoria_id: Option<i64>,
            _limite: i64,
            _offset: i64,
        ) -> AppResult<Vec<Producto>> {
            Ok(self.productos.lock().unwrap().clone())
        }

        fn crear_producto(&self, _nuevo: &NuevoProducto) -> AppResult<Producto> {
            unreachable!("no se usa en estos tests")
        }

        fn actualizar_producto(&self, _producto: &Producto) -> AppResult<()> {
            unreachable!("no se usa en estos tests")
        }

        fn eliminar_producto(&self, _id: i64) -> AppResult<()> {
            unreachable!("no se usa en estos tests")
        }

        fn ajustar_stock(
            &self,
            _id: i64,
            _stock_nuevo: i64,
            _motivo: &str,
            _operador: Option<&str>,
        ) -> AppResult<()> {
            unreachable!("no se usa en estos tests")
        }

        fn costos_de_productos(&self, _ids: &[i64]) -> AppResult<Vec<(i64, i64)>> {
            Ok(Vec::new())
        }
    }

    /// Mock del repo de ventas: guarda ventas en memoria y descuenta stock
    /// del mock de productos (misma garantÃ­a R9 que el repo real).
    struct MockVentas {
        ventas: Mutex<Vec<Venta>>,
        siguiente_id: Mutex<i64>,
        productos: Option<Arc<MockProductos>>,
    }

    impl MockVentas {
        fn con_productos(productos: Arc<MockProductos>) -> Self {
            Self {
                ventas: Mutex::new(Vec::new()),
                siguiente_id: Mutex::new(1),
                productos: Some(productos),
            }
        }
    }

    impl VentaRepo for MockVentas {
        fn crear_venta_transaccional(&self, venta: &NuevaVenta) -> AppResult<Venta> {
            let mut sig = self.siguiente_id.lock().unwrap();
            let id = *sig;
            *sig += 1;
            let v = Venta {
                id,
                items: venta.items.clone(),
                total: calcular_total(&venta.items),
                medio_pago: venta.medio_pago,
                monto_efectivo: venta.monto_efectivo,
                monto_mp: venta.monto_mp,
                descripcion_mp: venta.descripcion_mp.clone(),
                hora: "2026-09-10T10:00:00".to_string(),
            };
            if let Some(p) = self.productos.as_ref() {
                for item in &venta.items {
                    p.descontar(item.producto_id, item.cantidad);
                }
            }
            self.ventas.lock().unwrap().push(v.clone());
            Ok(v)
        }

        fn obtener_venta(&self, id: i64) -> AppResult<Option<Venta>> {
            Ok(self.ventas.lock().unwrap().iter().find(|v| v.id == id).cloned())
        }

        fn listar_ventas_dia(&self, _fecha: &str) -> AppResult<Vec<Venta>> {
            Ok(self.ventas.lock().unwrap().clone())
        }

        fn obtener_items_venta(&self, venta_id: i64) -> AppResult<Vec<VentaItem>> {
            Ok(self
                .ventas
                .lock()
                .unwrap()
                .iter()
                .find(|v| v.id == venta_id)
                .map(|v| v.items.clone())
                .unwrap_or_default())
        }

        fn crear_devolucion(
            &self,
            _venta_id: i64,
            _items: &[VentaItem],
            _motivo: Option<&str>,
        ) -> AppResult<()> {
            Ok(())
        }
    }

    fn item(producto_id: i64, cantidad: i64, precio: i64) -> VentaItem {
        VentaItem {
            producto_id,
            cantidad,
            precio_unitario: precio,
        }
    }

    #[test]
    fn registrar_venta_efectivo_valida_y_persiste() {
        let productos = Arc::new(MockProductos::nuevo());
        let pid = productos.agregar("Agua 500ml", 30);
        let ventas = MockVentas::con_productos(productos.clone());
        let svc = VentaService::new(ventas, productos.clone());

        let venta = svc
            .registrar_venta(&NuevaVenta {
                items: vec![item(pid, 2, 1000)],
                medio_pago: MedioPago::Efectivo,
                monto_efectivo: 2000,
                monto_mp: 0,
                descripcion_mp: None,
            })
            .unwrap();

        assert_eq!(venta.total, 2000);
        assert_eq!(svc.obtener_items_venta(venta.id).unwrap().len(), 1);
    }

    #[test]
    fn registrar_venta_rechaza_medio_incoherente() {
        let productos = Arc::new(MockProductos::nuevo());
        let pid = productos.agregar("Agua 500ml", 30);
        let ventas = MockVentas::con_productos(productos.clone());
        let svc = VentaService::new(ventas, productos.clone());

        let err = svc
            .registrar_venta(&NuevaVenta {
                items: vec![item(pid, 1, 1000)],
                medio_pago: MedioPago::MercadoPago,
                monto_efectivo: 500,
                monto_mp: 500,
                descripcion_mp: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Negocio(_)));
    }

    #[test]
    fn registrar_venta_rechaza_stock_insuficiente() {
        let productos = Arc::new(MockProductos::nuevo());
        let pid = productos.agregar("Alfajor Jorgito", 1);
        let ventas = MockVentas::con_productos(productos.clone());
        let svc = VentaService::new(ventas, productos.clone());

        let err = svc
            .registrar_venta(&NuevaVenta {
                items: vec![item(pid, 5, 1500)],
                medio_pago: MedioPago::Efectivo,
                monto_efectivo: 7500,
                monto_mp: 0,
                descripcion_mp: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Negocio(m) if m.contains("Stock insuficiente")));
    }

    #[test]
    fn registrar_venta_rechaza_producto_inexistente() {
        let productos = Arc::new(MockProductos::nuevo());
        let ventas = MockVentas::con_productos(productos.clone());
        let svc = VentaService::new(ventas, productos.clone());

        let err = svc
            .registrar_venta(&NuevaVenta {
                items: vec![item(999, 1, 1000)],
                medio_pago: MedioPago::Efectivo,
                monto_efectivo: 1000,
                monto_mp: 0,
                descripcion_mp: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Negocio(m) if m.contains("no existe")));
    }

    #[test]
    fn registrar_venta_mp_y_mixto() {
        let productos = Arc::new(MockProductos::nuevo());
        let pid = productos.agregar("Sprite", 10);
        let ventas = MockVentas::con_productos(productos.clone());
        let svc = VentaService::new(ventas, productos.clone());

        // MP puro
        svc.registrar_venta(&NuevaVenta {
            items: vec![item(pid, 1, 2000)],
            medio_pago: MedioPago::MercadoPago,
            monto_efectivo: 0,
            monto_mp: 2000,
            descripcion_mp: Some("Compra kiosco".to_string()),
        })
        .unwrap();

        // Mixto
        let venta_mixta = svc
            .registrar_venta(&NuevaVenta {
                items: vec![item(pid, 1, 2000)],
                medio_pago: MedioPago::Mixto,
                monto_efectivo: 500,
                monto_mp: 1500,
                descripcion_mp: None,
            })
            .unwrap();
        assert_eq!(venta_mixta.medio_pago, MedioPago::Mixto);
        assert_eq!(productos.buscar_por_id(pid).unwrap().unwrap().stock, 8);
    }

    #[test]
    fn registrar_devolucion_rechaza_venta_inexistente() {
        let productos = Arc::new(MockProductos::nuevo());
        let ventas = MockVentas::con_productos(productos.clone());
        let svc = VentaService::new(ventas, productos.clone());

        let err = svc
            .registrar_devolucion(999, &[item(1, 1, 1000)], Some("error de cobro"))
            .unwrap_err();
        assert!(matches!(err, AppError::Negocio(m) if m.contains("no existe")));
    }

    #[test]
    fn total_venta_recalcula_desde_items() {
        let productos = Arc::new(MockProductos::nuevo());
        let ventas = MockVentas::con_productos(productos.clone());
        let svc = VentaService::new(ventas, productos.clone());

        assert_eq!(svc.total_venta(&[item(1, 2, 500), item(2, 1, 1000)]), 2000);
    }
}