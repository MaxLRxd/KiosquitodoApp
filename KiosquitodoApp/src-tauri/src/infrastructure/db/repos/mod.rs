//! Implementaciones concretas de repositorios sobre SQLite.
//!
//! Cada submódulo contiene el adaptador de un recurso; se re-exportan los
//! adaptadores al nivel del módulo para que la capa de commands (`api`)
//! los importe como `repos::ProductoSqlite` (equivalente a los beans de
//! Spring Boot).

pub mod cierre_sqlite;
pub mod pago_sqlite;
pub mod producto_sqlite;
pub mod venta_sqlite;

pub use cierre_sqlite::CierreSqlite;
pub use pago_sqlite::PagoSqlite;
pub use producto_sqlite::ProductoSqlite;
pub use venta_sqlite::VentaSqlite;
