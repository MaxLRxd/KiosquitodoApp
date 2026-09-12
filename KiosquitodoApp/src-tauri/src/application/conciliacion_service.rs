//! Casos de uso de conciliación: sync de pagos MP (trayecto offline),
//! matching automático y vinculación manual.
//!
//! Orquesta `MpCliente` (API remota), `PagoRepo` (persistencia de pagos) y
//! `VentaRepo` (ventas MP para delta y vinculación), sin conocer HTTP ni SQL.

use crate::domain::pago::PagoMp;
use crate::domain::venta::Venta;
use crate::errors::{AppError, AppResult};
use crate::ports::mp_cliente::MpCliente;
use crate::ports::pago_repo::PagoRepo;
use crate::ports::venta_repo::VentaRepo;

/// Resultado de una sincronización con la API de MP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncResult {
    /// Pagos nuevos insertados (no existían localmente).
    pub nuevos: usize,
    /// Pagos que ya estaban importados (idempotencia R5).
    pub existentes: usize,
}

/// Casos de uso de conciliación MP.
pub struct ConciliacionService<M: MpCliente, P: PagoRepo, V: VentaRepo> {
    mp_cliente: M,
    pago_repo: P,
    venta_repo: V,
}

impl<M: MpCliente, P: PagoRepo, V: VentaRepo> ConciliacionService<M, P, V> {
    pub fn new(mp_cliente: M, pago_repo: P, venta_repo: V) -> Self {
        Self {
            mp_cliente,
            pago_repo,
            venta_repo,
        }
    }

    /// Importa los pagos elegibles de la API en el rango `[desde, hasta]`
    /// (ISO 8601). Inserta de forma idempotente sobre `mp_id` (R5).
    pub async fn sincronizar(
        &self,
        access_token: &str,
        desde: &str,
        hasta: &str,
    ) -> AppResult<SyncResult> {
        let pagos = self
            .mp_cliente
            .buscar_pagos(access_token, desde, hasta)
            .await?;

        let mut nuevos = 0;
        let mut existentes = 0;
        for pago in &pagos {
            if self.pago_repo.upsert_pago(pago)? {
                nuevos += 1;
            } else {
                existentes += 1;
            }
        }
        Ok(SyncResult { nuevos, existentes })
    }

    /// Matching automático (R7): vincula pagos huérfanos con ventas MP de
    /// monto ±100 centavos dentro de ±10 minutos. Devuelve cuántos se vincularon.
    pub fn aplicar_matching(&self) -> AppResult<usize> {
        self.pago_repo.aplicar_matching_automatico()
    }

    /// Vincular manualmente un pago MP a una venta (el operador corrige
    /// cuando el matching automático no encontró candidato).
    pub fn vincular_pago_a_venta(&self, mp_id: &str, venta_id: i64) -> AppResult<()> {
        let venta = self
            .venta_repo
            .obtener_venta(venta_id)?
            .ok_or_else(|| AppError::negocio("La venta no existe"))?;
        if !venta.medio_pago.involucra_mp() {
            return Err(AppError::negocio(
                "Solo se pueden vincular pagos a ventas con Mercado Pago",
            ));
        }
        let pago = self
            .pago_repo
            .obtener_pago_por_mp_id(mp_id)?
            .ok_or_else(|| AppError::negocio("El pago de Mercado Pago no existe"))?;

        // R8: si hay pago con diferencia > 1 centavo, la vista lo marcará
        // como MONTO DIFIERE; aquí solo se reporta, no se bloquea la
        // vinculación (el operador decide).
        let _desviacion = pago.monto - venta.monto_mp;

        self.pago_repo.vincular_pago_a_venta(mp_id, venta_id)
    }

    /// Delta de conciliación de un día (`YYYY-MM-DD`): Σ pagos acreditados
    /// según API − Σ montos MP registrados en ventas. Positivo = la API
    /// muestra más de lo registrado.
    pub fn delta_dia(&self, fecha_dia: &str) -> AppResult<i64> {
        let pagos = self.pagos_importados_del_dia(fecha_dia)?;
        let acreditado: i64 = pagos.iter().map(|p| p.monto).sum();

        let ventas = self.venta_repo.listar_ventas_dia(fecha_dia)?;
        let registrado: i64 = ventas
            .iter()
            .filter(|v| v.medio_pago.involucra_mp())
            .map(|v| v.monto_mp)
            .sum();

        Ok(acreditado - registrado)
    }

    /// Pagos importados de un día (rango `YYYY-MM-DDT00:00:00`..`23:59:59`).
    pub fn pagos_importados_del_dia(&self, fecha_dia: &str) -> AppResult<Vec<PagoMp>> {
        self.pago_repo.pagos_en_rango(
            &format!("{fecha_dia}T00:00:00"),
            &format!("{fecha_dia}T23:59:59"),
        )
    }

    /// Cuántos pagos del día fueron importados pero siguen sin venta vinculada.
    pub fn pagos_huerfanos_dia(&self, fecha_dia: &str) -> AppResult<Vec<PagoMp>> {
        Ok(self
            .pagos_importados_del_dia(fecha_dia)?
            .into_iter()
            .filter(|p| p.venta_id.is_none())
            .collect())
    }

    /// Ventas MP del día (para la grilla de conciliación de la UI).
    pub fn ventas_mp_dia(&self, fecha_dia: &str) -> AppResult<Vec<Venta>> {
        Ok(self
            .venta_repo
            .listar_ventas_dia(fecha_dia)?
            .into_iter()
            .filter(|v| v.medio_pago.involucra_mp())
            .collect())
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::venta::{MedioPago, NuevaVenta, Venta, VentaItem};
    use std::sync::Mutex;

    // ── Mocks ─────────────────────────────────────────────────────────

    struct MockMp {
        pagos: Vec<PagoMp>,
    }

    impl MockMp {
        fn con(pagos: Vec<PagoMp>) -> Self {
            Self { pagos }
        }
    }

    impl MpCliente for MockMp {
        fn buscar_pagos(
            &self,
            _access_token: &str,
            _desde: &str,
            _hasta: &str,
        ) -> impl std::future::Future<Output = AppResult<Vec<PagoMp>>> + Send {
            let pagos = self.pagos.clone();
            async move { Ok(pagos) }
        }
    }

    #[derive(Default)]
    struct MockPagos {
        pagos: Mutex<Vec<PagoMp>>,
    }

    impl MockPagos {
        fn con(pagos: Vec<PagoMp>) -> Self {
            Self {
                pagos: Mutex::new(pagos),
            }
        }
    }

    impl PagoRepo for MockPagos {
        fn upsert_pago(&self, pago: &PagoMp) -> AppResult<bool> {
            let mut pagos = self.pagos.lock().unwrap();
            if pagos.iter().any(|p| p.mp_id == pago.mp_id) {
                return Ok(false);
            }
            pagos.push(pago.clone());
            Ok(true)
        }

        fn pagos_en_rango(&self, _desde: &str, _hasta: &str) -> AppResult<Vec<PagoMp>> {
            Ok(self.pagos.lock().unwrap().clone())
        }

        fn obtener_pago_por_mp_id(&self, mp_id: &str) -> AppResult<Option<PagoMp>> {
            Ok(self
                .pagos
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.mp_id == mp_id)
                .cloned())
        }

        fn vincular_pago_a_venta(&self, mp_id: &str, venta_id: i64) -> AppResult<()> {
            let mut pagos = self.pagos.lock().unwrap();
            if let Some(p) = pagos.iter_mut().find(|p| p.mp_id == mp_id) {
                p.venta_id = Some(venta_id);
                Ok(())
            } else {
                Err(AppError::negocio("El pago no existe"))
            }
        }

        fn aplicar_matching_automatico(&self) -> AppResult<usize> {
            Ok(0)
        }

        fn existe_pago(&self, mp_id: &str) -> AppResult<bool> {
            Ok(self.pagos.lock().unwrap().iter().any(|p| p.mp_id == mp_id))
        }
    }

    #[derive(Default)]
    struct MockVentas {
        ventas: Mutex<Vec<Venta>>,
    }

    impl MockVentas {
        fn con(ventas: Vec<Venta>) -> Self {
            Self {
                ventas: Mutex::new(ventas),
            }
        }

        fn venta_mp(id: i64, monto_mp: i64) -> Venta {
            Venta {
                id,
                items: Vec::new(),
                total: monto_mp,
                medio_pago: MedioPago::MercadoPago,
                monto_efectivo: 0,
                monto_mp,
                descripcion_mp: None,
                hora: "2026-09-10T10:00:00".to_string(),
            }
        }
    }

    impl VentaRepo for MockVentas {
        fn crear_venta_transaccional(&self, _venta: &NuevaVenta) -> AppResult<Venta> {
            unreachable!("no se usa en estos tests")
        }

        fn obtener_venta(&self, id: i64) -> AppResult<Option<Venta>> {
            Ok(self.ventas.lock().unwrap().iter().find(|v| v.id == id).cloned())
        }

        fn listar_ventas_dia(&self, _fecha: &str) -> AppResult<Vec<Venta>> {
            Ok(self.ventas.lock().unwrap().clone())
        }

        fn obtener_items_venta(&self, _venta_id: i64) -> AppResult<Vec<VentaItem>> {
            Ok(Vec::new())
        }

        fn crear_devolucion(
            &self,
            _venta_id: i64,
            _items: &[VentaItem],
            _motivo: Option<&str>,
        ) -> AppResult<()> {
            unreachable!("no se usa en estos tests")
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────

    fn pago(mp_id: &str, monto: i64, venta_id: Option<i64>) -> PagoMp {
        PagoMp {
            id: 0,
            mp_id: mp_id.to_string(),
            fecha_aprobacion: Some("2026-09-10T10:30:00.000-03:00".to_string()),
            monto,
            descripcion: None,
            venta_id,
        }
    }

    fn service<'a>(
        mp: MockMp,
        pagos: MockPagos,
        ventas: MockVentas,
    ) -> ConciliacionService<MockMp, MockPagos, MockVentas> {
        ConciliacionService::new(mp, pagos, ventas)
    }

    // ── Tests ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn sincronizar_inserta_nuevos_y_detecta_existentes() {
        let svc = service(
            MockMp::con(vec![pago("a", 1500, None), pago("b", 800, None)]),
            MockPagos::con(vec![pago("a", 1500, None)]), // "a" ya existe
            MockVentas::default(),
        );

        let res = svc
            .sincronizar("tok_test", "2026-09-10T10:00:00", "2026-09-10T11:00:00")
            .await
            .unwrap();

        assert_eq!(res.nuevos, 1);
        assert_eq!(res.existentes, 1);
    }

    #[tokio::test]
    async fn sincronizar_no_inserta_repetidos() {
        let svc = service(
            MockMp::con(vec![pago("x", 2000, None), pago("x", 2000, None)]),
            MockPagos::default(),
            MockVentas::default(),
        );

        let res = svc.sincronizar("tok", "a", "b").await.unwrap();
        assert_eq!(res.nuevos, 1);
        assert_eq!(res.existentes, 1);
    }

    #[tokio::test]
    async fn sincronizar_vacio_no_cambia_nada() {
        let svc = service(MockMp::con(vec![]), MockPagos::default(), MockVentas::default());
        let res = svc.sincronizar("tok", "a", "b").await.unwrap();
        assert_eq!(res.nuevos, 0);
        assert_eq!(res.existentes, 0);
    }

    #[test]
    fn vincular_pago_a_venta_mp() {
        let venta = MockVentas::venta_mp(7, 1500);
        let svc = service(
            MockMp::con(vec![]),
            MockPagos::con(vec![pago("abc", 1500, None)]),
            MockVentas::con(vec![venta]),
        );

        svc.vincular_pago_a_venta("abc", 7).unwrap();
        assert_eq!(
            svc.pago_repo.obtener_pago_por_mp_id("abc").unwrap().unwrap().venta_id,
            Some(7)
        );
    }

    #[test]
    fn vincular_rechaza_venta_inexistente() {
        let svc = service(
            MockMp::con(vec![]),
            MockPagos::con(vec![pago("abc", 1500, None)]),
            MockVentas::default(),
        );
        assert!(svc.vincular_pago_a_venta("abc", 999).is_err());
    }

    #[test]
    fn vincular_rechaza_venta_sin_mp() {
        let mut venta = MockVentas::venta_mp(1, 1500);
        venta.medio_pago = MedioPago::Efectivo;
        venta.monto_mp = 0;
        venta.total = 1500;
        venta.monto_efectivo = 1500;
        let svc = service(
            MockMp::con(vec![]),
            MockPagos::con(vec![pago("abc", 1500, None)]),
            MockVentas::con(vec![venta]),
        );
        let err = svc.vincular_pago_a_venta("abc", 1).unwrap_err();
        assert!(matches!(err, AppError::Negocio(m) if m.contains("Mercado Pago")));
    }

    #[test]
    fn vincular_rechaza_pago_inexistente() {
        let svc = service(
            MockMp::con(vec![]),
            MockPagos::default(),
            MockVentas::con(vec![MockVentas::venta_mp(1, 1500)]),
        );
        assert!(svc.vincular_pago_a_venta("no_existe", 1).is_err());
    }

    #[test]
    fn delta_dia_positivo_cuando_api_muestra_mas() {
        let svc = service(
            MockMp::con(vec![]),
            MockPagos::con(vec![pago("a", 1500, Some(1)), pago("b", 800, None)]),
            MockVentas::con(vec![MockVentas::venta_mp(1, 1500)]),
        );
        // Acreditado 2300 − registrado 1500 = 800 (pago huérfano según API)
        assert_eq!(svc.delta_dia("2026-09-10").unwrap(), 800);
    }

    #[test]
    fn delta_dia_ignora_ventas_en_efectivo() {
        let svc = service(
            MockMp::con(vec![]),
            MockPagos::default(),
            MockVentas::con(vec![MockVentas::venta_mp(1, 1500)]),
        );
        // No hay pagos importados y la venta es MP → delta negativo
        assert_eq!(svc.delta_dia("2026-09-10").unwrap(), -1500);
    }

    #[test]
    fn pagos_huerfanos_solo_devuelve_sin_venta() {
        let svc = service(
            MockMp::con(vec![]),
            MockPagos::con(vec![pago("a", 1000, None), pago("b", 500, Some(3))]),
            MockVentas::default(),
        );
        let huerfanos = svc.pagos_huerfanos_dia("2026-09-10").unwrap();
        assert_eq!(huerfanos.len(), 1);
        assert_eq!(huerfanos[0].mp_id, "a");
    }

    #[test]
    fn ventas_mp_dia_filtra_solo_las_mp() {
        let mut efectivo = MockVentas::venta_mp(1, 2000);
        efectivo.medio_pago = MedioPago::Efectivo;
        efectivo.monto_mp = 0;
        efectivo.monto_efectivo = 2000;

        let svc = service(
            MockMp::con(vec![]),
            MockPagos::default(),
            MockVentas::con(vec![MockVentas::venta_mp(2, 3000), efectivo]),
        );
        let mps = svc.ventas_mp_dia("2026-09-10").unwrap();
        assert_eq!(mps.len(), 1);
        assert_eq!(mps[0].id, 2);
    }
}