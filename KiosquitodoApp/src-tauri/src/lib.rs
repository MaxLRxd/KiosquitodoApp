pub mod application;
pub mod config;
pub mod domain;
pub mod errors;
pub mod infrastructure;
pub mod ports;

use application::config_service::ConfigService;
use infrastructure::api::{caja_cmd, config_cmd, producto_cmd, venta_cmd};
use infrastructure::db::repos::ConfigSqlite;
use infrastructure::db::DbConn;
use infrastructure::system::backup;
use std::path::Path;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // Registrado sin endpoint ni clave pública: no se llama a `check()`,
        // así que el arranque no depende de un servidor de actualizaciones.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let cfg = config::AppConfig::resolver()
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

            infrastructure::system::logging::init_logging(&cfg.log_dir)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

            let db = DbConn::abrir(&cfg.db_path)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

            // Usa la carpeta guardada en preferencias (M4); si no se puede
            // leer, cae a la default y avisa, sin impedir el arranque.
            let backup_dir = ConfigService::new(
                ConfigSqlite::new(db.clone()),
                cfg.backup_dir.to_string_lossy().into_owned(),
            )
            .obtener()
            .map(|configuracion| configuracion.backup_dir)
            .unwrap_or_else(|e| {
                log::warn!("No se pudo leer la carpeta de backups: {e}");
                cfg.backup_dir.to_string_lossy().into_owned()
            });

            if let Err(e) = backup::backup_diario_si_necesario(&db, Path::new(&backup_dir)) {
                log::warn!("No se pudo crear el backup diario: {e}");
            }

            app.manage(db);

            log::info!("App iniciada. BD: {}", cfg.db_path.display());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            producto_cmd::crear_producto,
            producto_cmd::listar_productos,
            producto_cmd::buscar_producto_por_barcode,
            producto_cmd::actualizar_producto,
            producto_cmd::eliminar_producto,
            producto_cmd::ajustar_stock,
            producto_cmd::listar_categorias,
            producto_cmd::crear_categoria,
            venta_cmd::registrar_venta,
            venta_cmd::obtener_venta,
            venta_cmd::listar_ventas_dia,
            venta_cmd::registrar_devolucion,
            venta_cmd::listar_devoluciones,
            caja_cmd::resumen_diario,
            caja_cmd::cerrar_caja,
            caja_cmd::listar_cierres,
            caja_cmd::exportar_cierres_csv,
            config_cmd::obtener_rutas,
            config_cmd::obtener_configuracion,
            config_cmd::guardar_configuracion,
            config_cmd::pin_configurado,
            config_cmd::establecer_pin,
            config_cmd::verificar_pin,
            config_cmd::eliminar_pin,
            config_cmd::token_mp_configurado,
            config_cmd::guardar_token_mp,
            config_cmd::eliminar_token_mp,
            config_cmd::seleccionar_carpeta_backups,
            config_cmd::autostart_estado,
            config_cmd::habilitar_autostart,
            config_cmd::deshabilitar_autostart,
            config_cmd::crear_backup_manual,
            config_cmd::verificar_backup_diario,
            config_cmd::limpiar_backups_viejos,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación Tauri");
}
