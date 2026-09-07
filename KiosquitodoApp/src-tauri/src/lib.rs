pub mod application;
pub mod config;
pub mod domain;
pub mod errors;
pub mod infrastructure;
pub mod ports;

pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            // Fase futura: init_logging, backup diario, autostart y bloqueo por PIN.
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación Tauri");
}