//! Capa de entrada IPC: commands Tauri (aprox. a controllers de Spring Boot).
//!
//! Cada command es un envoltorio fino: construye el repo SQLite concreto
//! desde el `DbConn` del estado, instancia el service y devuelve el DTO.
//!
//! **Nota de alcance**: la integraciÃ³n con Mercado Pago (sync de pagos,
//! conciliaciÃ³n online y keyring de token) queda DIFERIDA por decisiÃ³n del
//! equipo; aquÃ­ solo hay operaciones locales de producto, venta y cierre.

pub mod caja_cmd;
pub mod producto_cmd;
pub mod config_cmd;
pub mod venta_cmd;
