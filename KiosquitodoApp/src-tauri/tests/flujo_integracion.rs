//! Smoke test de integración: el mismo camino que ejecuta la app en
//! producción (SQLite en archivo con PRAGMAs + migraciones, repos reales,
//! services reales): categoría → producto → venta → devolución → resumen →
//! cierre → CSV → listar devoluciones.

use kiosco_app_lib::application::cierre_service::CierreService;
use kiosco_app_lib::application::producto_service::ProductoService;
use kiosco_app_lib::application::venta_service::VentaService;
use kiosco_app_lib::domain::producto::NuevoProducto;
use kiosco_app_lib::domain::venta::{MedioPago, NuevaVenta, VentaItem};
use kiosco_app_lib::infrastructure::db::repos::{
    CierreSqlite, ProductoSqlite, VentaSqlite,
};
use kiosco_app_lib::infrastructure::db::DbConn;
use kiosco_app_lib::ports::producto_repo::CategoriaRepo;
use kiosco_app_lib::ports::venta_repo::VentaRepo;

fn bd_temporal() -> (DbConn, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("kiosco_e2e_{}.db", std::process::id()));
    let db = DbConn::abrir(&path).expect("abrir BD en archivo");
    (db, path)
}

fn producto(nombre: &str, barcode: &str, precio: i64, costo: i64, stock: i64) -> NuevoProducto {
    NuevoProducto {
        nombre: nombre.to_string(),
        barcode: Some(barcode.to_string()),
        precio_venta: precio,
        precio_costo: Some(costo),
        stock_inicial: stock,
        stock_minimo: 2,
        categoria_id: Some(1),
    }
}

#[test]
fn flujo_completo_en_bd_real() {
    let (db, path) = bd_temporal();

    // ── Categoría ─────────────────────────────────────────────────────
    let producto_repo = ProductoSqlite::new(db.clone());
    let categoria = producto_repo
        .crear_categoria("Bebidas", Some("#3b82f6"))
        .expect("crear categoría");
    assert!(categoria.id > 0);
    assert_eq!(
        producto_repo
            .listar_categorias()
            .unwrap()
            .iter()
            .map(|c| c.nombre.as_str())
            .collect::<Vec<_>>(),
        vec!["Bebidas"]
    );

    // ── Productos ─────────────────────────────────────────────────────
    let p_svc = ProductoService::new(ProductoSqlite::new(db.clone()));
    let coca = p_svc
        .crear_producto(&producto("Coca-Cola 1.5L", "7790000000016", 3500, 2450, 10))
        .expect("crear producto");
    let agua = p_svc
        .crear_producto(&producto("Agua 500ml", "7790000000023", 1500, 600, 20))
        .expect("crear producto");
    assert_eq!(p_svc.buscar_por_barcode("7790000000016").unwrap().unwrap().nombre, "Coca-Cola 1.5L");

    // ── Venta en efectivo ─────────────────────────────────────────────
    let v_svc = VentaService::new(VentaSqlite::new(db.clone()), ProductoSqlite::new(db.clone()));
    let items = vec![
        VentaItem {
            producto_id: coca.id,
            cantidad: 2,
            precio_unitario: 3500,
        },
        VentaItem {
            producto_id: agua.id,
            cantidad: 1,
            precio_unitario: 1500,
        },
    ];
    let total = v_svc.total_venta(&items);
    assert_eq!(total, 8500);
    let venta = v_svc
        .registrar_venta(&NuevaVenta {
            items: items.clone(),
            medio_pago: MedioPago::Efectivo,
            monto_efectivo: total,
            monto_mp: 0,
            descripcion_mp: None,
        })
        .expect("registrar venta");
    assert!(venta.id > 0);

    // El descuento de stock se aplicó (R9).
    assert_eq!(p_svc.buscar_por_id(coca.id).unwrap().unwrap().stock, 8);
    assert_eq!(p_svc.buscar_por_id(agua.id).unwrap().unwrap().stock, 19);

    // ── Devolución parcial ────────────────────────────────────────────
    v_svc
        .registrar_devolucion(
            venta.id,
            &[VentaItem {
                producto_id: coca.id,
                cantidad: 1,
                precio_unitario: 3500,
            }],
            Some("cliente se arrepintió"),
        )
        .expect("registrar devolución");
    assert_eq!(p_svc.buscar_por_id(coca.id).unwrap().unwrap().stock, 9);

    let devoluciones = VentaSqlite::new(db.clone())
        .listar_devoluciones()
        .expect("listar devoluciones");
    assert_eq!(devoluciones.len(), 1);
    assert_eq!(devoluciones[0].items.len(), 1);
    assert_eq!(devoluciones[0].motivo.as_deref(), Some("cliente se arrepintió"));

    // ── Cierre de caja ────────────────────────────────────────────────
    let c_svc = CierreService::new(CierreSqlite::new(db.clone()));
    let resumen = c_svc
        .resumen_diario(&venta.hora[..10])
        .expect("resumen diario");
    assert_eq!(resumen.cantidad_ventas, 1);
    assert_eq!(resumen.total_ventas, 8500);
    assert_eq!(resumen.total_efectivo, 8500);

    let cierre = c_svc
        .cerrar_caja(&venta.hora[..10])
        .expect("cerrar caja");
    assert!(cierre.id > 0);
    assert_eq!(c_svc.listar_cierres().unwrap().len(), 1);

    // ── CSV ───────────────────────────────────────────────────────────
    let csv = c_svc
        .exportar_csv(&venta.hora[..10], &venta.hora[..10])
        .expect("exportar csv");
    assert!(csv.contains("cantidad_ventas"));
    assert!(csv.contains("1"));

    // Limpieza
    drop(db);
    let _ = std::fs::remove_file(&path);
}