//! Casos de uso de inventario: CRUD, bÃºsqueda, paginaciÃ³n y ajuste de stock.
//!
//! El servicio orquesta la lÃ³gica pura del dominio (validaciones) y delega la
//! persistencia al `ProductoRepo` (inversiÃ³n de dependencias, capa hexagonal).

use crate::domain::producto::{
    validar_barcode, validar_costo, validar_nuevo_producto, validar_precio, validar_stock,
    validar_stock_minimo, NuevoProducto, Producto,
};
use crate::errors::{AppError, AppResult};
use crate::ports::producto_repo::ProductoRepo;

/// Casos de uso de inventario. GenÃ©rico sobre el trait para poder inyectar
/// el repo SQLite en producciÃ³n y un mock en los tests.
pub struct ProductoService<R: ProductoRepo> {
    repo: R,
}

impl<R: ProductoRepo> ProductoService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Crea un producto validando las reglas de negocio antes de persistir.
    pub fn crear_producto(&self, nuevo: &NuevoProducto) -> AppResult<Producto> {
        validar_nuevo_producto(nuevo).map_err(AppError::from)?;
        self.repo.crear_producto(nuevo)
    }

    /// Busca por cÃ³digo de barras exacto. `None` si no existe o estÃ¡ inactivo.
    pub fn buscar_por_barcode(&self, barcode: &str) -> AppResult<Option<Producto>> {
        self.repo.buscar_por_barcode(barcode)
    }

    /// Busca por id. `None` si no existe o estÃ¡ inactivo.
    pub fn buscar_por_id(&self, id: i64) -> AppResult<Option<Producto>> {
        self.repo.buscar_por_id(id)
    }

    /// Lista productos paginados y filtrados. `pagina` y `tam_pagina` son
    /// 1-based; se ordenan por nombre ascendente.
    pub fn listar_productos(
        &self,
        filtro: &str,
        categoria_id: Option<i64>,
        pagina: i64,
        tam_pagina: i64,
    ) -> AppResult<Vec<Producto>> {
        if pagina < 1 {
            return Err(AppError::negocio(
                "El nÃºmero de pÃ¡gina debe ser mayor o igual a uno",
            ));
        }
        if tam_pagina < 1 {
            return Err(AppError::negocio(
                "El tamaÃ±o de pÃ¡gina debe ser mayor a cero",
            ));
        }
        let offset = (pagina - 1) * tam_pagina;
        self.repo
            .listar_productos(filtro, categoria_id, tam_pagina, offset)
    }

    /// Actualiza campos editables (nombre, precios, stock, categorÃ­a).
    /// El id y la baja lÃ³gica no se tocan.
    pub fn actualizar_producto(&self, producto: &Producto) -> AppResult<()> {
        validar_producto_actualizado(producto)?;
        self.repo.actualizar_producto(producto)
    }

    /// Baja lÃ³gica: inactiva el producto para no perder historial de ventas.
    pub fn eliminar_producto(&self, id: i64) -> AppResult<()> {
        self.repo.eliminar_producto(id)
    }

    /// Ajuste manual de stock. Valida que el producto exista y que el nuevo
    /// stock no sea negativo; el repo registra el movimiento de inventario.
    pub fn ajustar_stock(
        &self,
        id: i64,
        stock_nuevo: i64,
        motivo: &str,
        operador: Option<&str>,
    ) -> AppResult<()> {
        if stock_nuevo < 0 {
            return Err(AppError::negocio("El nuevo stock no puede ser negativo"));
        }
        if motivo.trim().is_empty() {
            return Err(AppError::negocio(
                "Debe indicar un motivo para el ajuste de stock",
            ));
        }
        match self.repo.buscar_por_id(id)? {
            Some(_) => self.repo.ajustar_stock(id, stock_nuevo, motivo, operador),
            None => Err(AppError::negocio("El producto no existe")),
        }
    }
}

/// Valida un `Producto` existente (mismas reglas que `validar_nuevo_producto`).
fn validar_producto_actualizado(p: &Producto) -> AppResult<()> {
    if p.nombre.trim().is_empty() {
        return Err(AppError::negocio("El nombre del producto no puede estar vacÃ­o"));
    }
    if !validar_barcode(p.barcode.as_deref().unwrap_or("")) {
        return Err(AppError::negocio("El cÃ³digo de barras debe tener 8, 12 o 13 dÃ­gitos"));
    }
    if !validar_precio(p.precio_venta) {
        return Err(AppError::negocio("El precio de venta debe ser mayor a cero (centavos)"));
    }
    if !validar_costo(p.precio_costo) {
        return Err(AppError::negocio("El precio de costo no puede ser negativo (centavos)"));
    }
    if !validar_stock(p.stock) {
        return Err(AppError::negocio("El stock no puede ser negativo"));
    }
    if !validar_stock_minimo(p.stock_minimo) {
        return Err(AppError::negocio("El stock mÃ­nimo no puede ser negativo"));
    }
    Ok(())
}

// â”€â”€ Tests â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mock del repo en memoria que replica las invariantes mÃ­nimas de
    /// `producto_sqlite` (id autoincremental, barcode Ãºnico, baja lÃ³gica).
    struct MockProductoRepo {
        productos: Mutex<Vec<Producto>>,
        siguiente_id: Mutex<i64>,
    }

    impl MockProductoRepo {
        fn nuevo() -> Self {
            Self {
                productos: Mutex::new(Vec::new()),
                siguiente_id: Mutex::new(1),
            }
        }

        fn nuevo_del_service() -> ProductoService<MockProductoRepo> {
            ProductoService::new(Self::nuevo())
        }

        fn insertar_directo(&self, nombre: &str, barcode: &str, precio: i64, stock: i64) -> Producto {
            let mut sig = self.siguiente_id.lock().unwrap();
            let p = Producto {
                id: *sig,
                nombre: nombre.to_string(),
                barcode: Some(barcode.to_string()),
                precio_venta: precio,
                precio_costo: Some(precio / 2),
                stock,
                stock_minimo: 0,
                activo: true,
                categoria_id: None,
            };
            *sig += 1;
            self.productos.lock().unwrap().push(p.clone());
            p
        }
    }

    impl ProductoRepo for MockProductoRepo {
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
            filtro: &str,
            categoria_id: Option<i64>,
            limite: i64,
            offset: i64,
        ) -> AppResult<Vec<Producto>> {
            let f = filtro.trim().to_lowercase();
            let mut prods: Vec<Producto> = self
                .productos
                .lock()
                .unwrap()
                .iter()
                .filter(|p| {
                    p.activo
                        && (f.is_empty()
                            || p.nombre.to_lowercase().contains(&f)
                            || p.barcode.as_deref().unwrap_or("").contains(&f))
                        && categoria_id.map_or(true, |c| p.categoria_id == Some(c))
                })
                .cloned()
                .collect();
            prods.sort_by(|a, b| a.nombre.cmp(&b.nombre));
            Ok(prods
                .into_iter()
                .skip(offset as usize)
                .take(limite as usize)
                .collect())
        }

        fn crear_producto(&self, nuevo: &NuevoProducto) -> AppResult<Producto> {
            let dup = self
                .productos
                .lock()
                .unwrap()
                .iter()
                .any(|p| p.barcode == nuevo.barcode);
            if dup {
                return Err(AppError::negocio(
                    "Ya existe un producto con ese cÃ³digo de barras",
                ));
            }
            let mut sig = self.siguiente_id.lock().unwrap();
            let p = Producto {
                id: *sig,
                nombre: nuevo.nombre.trim().to_string(),
                barcode: nuevo.barcode.clone(),
                precio_venta: nuevo.precio_venta,
                precio_costo: nuevo.precio_costo,
                stock: nuevo.stock_inicial,
                stock_minimo: nuevo.stock_minimo,
                activo: true,
                categoria_id: nuevo.categoria_id,
            };
            *sig += 1;
            self.productos.lock().unwrap().push(p.clone());
            Ok(p)
        }

        fn actualizar_producto(&self, producto: &Producto) -> AppResult<()> {
            let mut prods = self.productos.lock().unwrap();
            if let Some(p) = prods.iter_mut().find(|p| p.id == producto.id) {
                *p = producto.clone();
                Ok(())
            } else {
                Err(AppError::negocio("El producto no existe"))
            }
        }

        fn eliminar_producto(&self, id: i64) -> AppResult<()> {
            let mut prods = self.productos.lock().unwrap();
            if let Some(p) = prods.iter_mut().find(|p| p.id == id) {
                p.activo = false;
                Ok(())
            } else {
                Err(AppError::negocio("El producto no existe"))
            }
        }

        fn ajustar_stock(
            &self,
            id: i64,
            stock_nuevo: i64,
            _motivo: &str,
            _operador: Option<&str>,
        ) -> AppResult<()> {
            let mut prods = self.productos.lock().unwrap();
            if let Some(p) = prods.iter_mut().find(|p| p.id == id) {
                p.stock = stock_nuevo;
                Ok(())
            } else {
                Err(AppError::negocio("El producto no existe"))
            }
        }

        fn costos_de_productos(&self, ids: &[i64]) -> AppResult<Vec<(i64, i64)>> {
            let prods = self.productos.lock().unwrap();
            Ok(ids
                .iter()
                .filter_map(|id| {
                    prods
                        .iter()
                        .find(|p| p.id == *id)
                        .map(|p| (*id, p.precio_costo.unwrap_or(0)))
                })
                .collect())
        }
    }

    fn nuevo_valido() -> NuevoProducto {
        NuevoProducto {
            nombre: "Coca-Cola 1.5L".to_string(),
            barcode: Some("7790001234567".to_string()),
            precio_venta: 3500,
            precio_costo: Some(2450),
            stock_inicial: 12,
            stock_minimo: 5,
            categoria_id: None,
        }
    }

    #[test]
    fn crear_valida_y_persiste() {
        let svc = MockProductoRepo::nuevo_del_service();
        let p = svc.crear_producto(&nuevo_valido()).unwrap();
        assert!(p.id >= 1);
        assert_eq!(p.nombre, "Coca-Cola 1.5L");
    }

    #[test]
    fn crear_rechaza_datos_invalidos() {
        let svc = MockProductoRepo::nuevo_del_service();
        let mut invalido = nuevo_valido();
        invalido.precio_venta = 0;
        let err = svc.crear_producto(&invalido).unwrap_err();
        assert!(matches!(err, AppError::Negocio(_)));
    }

    #[test]
    fn crear_rechaza_barcode_duplicado() {
        let svc = MockProductoRepo::nuevo_del_service();
        svc.crear_producto(&nuevo_valido()).unwrap();
        let err = svc.crear_producto(&nuevo_valido()).unwrap_err();
        assert!(matches!(err, AppError::Negocio(ref m) if m.contains("cÃ³digo de barras")));
    }

    #[test]
    fn buscar_por_barcode_devuelve_producto() {
        let repo = MockProductoRepo::nuevo();
        repo.insertar_directo("Agua 500ml", "7790000000011", 1200, 30);
        let svc = ProductoService::new(repo);

        assert!(svc.buscar_por_barcode("7790000000011").unwrap().is_some());
        assert!(svc.buscar_por_barcode("999").unwrap().is_none());
    }

    #[test]
    fn buscar_por_id_devuelve_none_si_inexistente() {
        let svc = MockProductoRepo::nuevo_del_service();
        assert!(svc.buscar_por_id(99).unwrap().is_none());
    }

    #[test]
    fn listar_filtra_y_pagina() {
        let repo = MockProductoRepo::nuevo();
        repo.insertar_directo("Coca-Cola", "1111111111111", 3500, 10);
        repo.insertar_directo("Sprite", "2222222222222", 3000, 10);
        repo.insertar_directo("Agua", "3333333333333", 1200, 10);
        let svc = ProductoService::new(repo);

        let todas = svc.listar_productos("", None, 1, 10).unwrap();
        assert_eq!(todas.len(), 3);

        let filtradas = svc.listar_productos("co", None, 1, 10).unwrap();
        assert_eq!(filtradas.len(), 1);
        assert_eq!(filtradas[0].nombre, "Coca-Cola");

        let pagina2 = svc.listar_productos("", None, 3, 1).unwrap();
        assert_eq!(pagina2.len(), 1);
        assert_eq!(pagina2[0].nombre, "Sprite");
    }

    #[test]
    fn listar_valida_parametros_de_paginacion() {
        let svc = MockProductoRepo::nuevo_del_service();
        assert!(svc.listar_productos("", None, 0, 10).is_err());
        assert!(svc.listar_productos("", None, 1, 0).is_err());
    }

    #[test]
    fn actualizar_aplica_cambios() {
        let repo = MockProductoRepo::nuevo();
        let creado = repo.insertar_directo("Alfajor 1", "5555555555555", 1500, 20);
        let svc = ProductoService::new(repo);

        let mut editado = creado.clone();
        editado.precio_venta = 1800;
        svc.actualizar_producto(&editado).unwrap();

        let p = svc.buscar_por_id(creado.id).unwrap().unwrap();
        assert_eq!(p.precio_venta, 1800);
    }

    #[test]
    fn actualizar_rechaza_reglas_de_negocio() {
        let repo = MockProductoRepo::nuevo();
        let creado = repo.insertar_directo("Alfajor 1", "5566666666666", 1500, 20);
        let svc = ProductoService::new(repo);

        let mut invalido = creado.clone();
        invalido.precio_venta = -1;
        let err = svc.actualizar_producto(&invalido).unwrap_err();
        assert!(matches!(err, AppError::Negocio(_)));
    }

    #[test]
    fn eliminar_es_baja_logica() {
        let repo = MockProductoRepo::nuevo();
        let creado = repo.insertar_directo("Alfajor 2", "5577777777777", 1500, 20);
        let svc = ProductoService::new(repo);

        svc.eliminar_producto(creado.id).unwrap();
        assert!(svc.buscar_por_id(creado.id).unwrap().is_none());
    }

    #[test]
    fn ajustar_stock_valida_y_aplica() {
        let repo = MockProductoRepo::nuevo();
        let creado = repo.insertar_directo("Alfajor 3", "5588888888888", 1500, 20);
        let svc = ProductoService::new(repo);

        svc.ajustar_stock(creado.id, 45, "recuento", Some("ana"))
            .unwrap();
        assert_eq!(svc.buscar_por_id(creado.id).unwrap().unwrap().stock, 45);

        assert!(svc.ajustar_stock(creado.id, -1, "recuento", None).is_err());
        assert!(svc.ajustar_stock(creado.id, 10, "  ", None).is_err());
        assert!(svc.ajustar_stock(999, 10, "recuento", None).is_err());
    }
}