//! Traits (puertos) que desacoplan la capa de aplicación de la infraestructura.

pub mod cierre_repo;
pub mod credencial_repo;
pub mod mp_cliente;
pub mod pago_repo;
pub mod producto_repo;
pub mod venta_repo;

pub use cierre_repo::{CierrePersistido, CierreRepo};
pub use credencial_repo::CredencialRepo;
pub use mp_cliente::MpCliente;
pub use pago_repo::PagoRepo;
pub use producto_repo::ProductoRepo;
pub use venta_repo::VentaRepo;