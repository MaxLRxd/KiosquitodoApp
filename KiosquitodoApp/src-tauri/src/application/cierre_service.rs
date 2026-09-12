//! Casos de uso de cierre de caja: resumen diario, cierre persistido + VACUUM
//! y exportación CSV del histórico.

use crate::domain::cierre::ResumenCierre;
use crate::errors::{AppError, AppResult};
use crate::ports::cierre_repo::{CierrePersistido, CierreRepo};

/// Casos de uso de cierre de caja. `CierreRepo` ya consolida el resumen del
/// día por SQL (totales, margen y delta MP); el servicio coordina la
/// persistencia y la compactación posterior.
pub struct CierreService<C: CierreRepo> {
    repo: C,
}

impl<C: CierreRepo> CierreService<C> {
    pub fn new(repo: C) -> Self {
        Self { repo }
    }

    /// Resumen del día (sin persistir). Útil para previsualizar el cierre.
    pub fn resumen_diario(&self, fecha: &str) -> AppResult<ResumenCierre> {
        validar_fecha(fecha)?;
        self.repo.resumen_dia(fecha)
    }

    /// Cierra la caja: consolida el resumen, lo persiste y ejecuta VACUUM
    /// para compactar el archivo de la BD (liberación de espacio del WAL).
    pub fn cerrar_caja(&self, fecha: &str) -> AppResult<CierrePersistido> {
        validar_fecha(fecha)?;
        let resumen = self.repo.resumen_dia(fecha)?;
        let id = self.repo.guardar_cierre(fecha, &resumen)?;
        self.repo.vacuar()?;
        Ok(CierrePersistido {
            id,
            fecha: fecha.to_string(),
            resumen,
        })
    }

    /// Historial de cierres, más reciente primero.
    pub fn listar_cierres(&self) -> AppResult<Vec<CierrePersistido>> {
        self.repo.obtener_cierres()
    }

    /// Exporta los cierres del rango `[desde, hasta]` a CSV (montos en
    /// centavos, primera fila = encabezado). Si los bordes están vacíos, no
    /// filtra por ese lado.
    pub fn exportar_csv(&self, desde: &str, hasta: &str) -> AppResult<String> {
        let cierres = self.repo.obtener_cierres()?;
        let mut buffer = String::from(
            "fecha,total_efectivo,total_mp,total_ventas,cantidad_ventas,margen_bruto,delta_mp\n",
        );

        for c in cierres
            .into_iter()
            .filter(|c| dentro_de_rango(&c.fecha, desde, hasta))
        {
            let r = &c.resumen;
            buffer.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                c.fecha,
                r.total_efectivo,
                r.total_mp,
                r.total_ventas,
                r.cantidad_ventas,
                r.margen_bruto,
                r.delta_mp,
            ));
        }
        Ok(buffer)
    }
}

/// Acepta fechas con formato estricto `YYYY-MM-DD`.
fn validar_fecha(fecha: &str) -> AppResult<()> {
    let f = fecha.trim();
    let valida = f.len() == 10
        && f.as_bytes()[4] == b'-'
        && f.as_bytes()[7] == b'-'
        && f[0..4].chars().all(|c| c.is_ascii_digit())
        && f[5..7].chars().all(|c| c.is_ascii_digit())
        && f[8..10].chars().all(|c| c.is_ascii_digit());
    if !valida {
        return Err(AppError::negocio(
            "La fecha debe tener formato YYYY-MM-DD",
        ));
    }
    Ok(())
}

fn dentro_de_rango(fecha: &str, desde: &str, hasta: &str) -> bool {
    let desde_ok = desde.trim().is_empty() || fecha >= desde.trim();
    let hasta_ok = hasta.trim().is_empty() || fecha <= hasta.trim();
    desde_ok && hasta_ok
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct EstadoMock {
        cierres: Vec<CierrePersistido>,
        vacuar_llamado: bool,
    }

    /// Mock con estado interno observable vía `Arc` compartido.
    #[derive(Clone)]
    struct MockCierre {
        estado: Arc<Mutex<EstadoMock>>,
    }

    impl MockCierre {
        fn nuevo(datos: Vec<(String, ResumenCierre)>) -> Self {
            let cierres = datos
                .into_iter()
                .enumerate()
                .map(|(i, (fecha, resumen))| CierrePersistido {
                    id: i as i64 + 1,
                    fecha,
                    resumen,
                })
                .collect();
            Self {
                estado: Arc::new(Mutex::new(EstadoMock {
                    cierres,
                    vacuar_llamado: false,
                })),
            }
        }
    }

    impl CierreRepo for MockCierre {
        fn resumen_dia(&self, _fecha: &str) -> AppResult<ResumenCierre> {
            Ok(ResumenCierre::nuevo(1000, 2000, 3000, 5, 800, 0))
        }

        fn guardar_cierre(&self, fecha: &str, resumen: &ResumenCierre) -> AppResult<i64> {
            let mut estado = self.estado.lock().unwrap();
            let id = estado.cierres.len() as i64 + 1;
            estado.cierres.push(CierrePersistido {
                id,
                fecha: fecha.to_string(),
                resumen: resumen.clone(),
            });
            Ok(id)
        }

        fn obtener_cierres(&self) -> AppResult<Vec<CierrePersistido>> {
            Ok(self.estado.lock().unwrap().cierres.clone())
        }

        fn vacuar(&self) -> AppResult<()> {
            self.estado.lock().unwrap().vacuar_llamado = true;
            Ok(())
        }
    }

    fn svc(mock: MockCierre) -> CierreService<MockCierre> {
        CierreService::new(mock)
    }

    fn resumen(efectivo: i64) -> ResumenCierre {
        ResumenCierre::nuevo(efectivo, 0, efectivo, 1, efectivo / 2, 0)
    }

    #[test]
    fn resumen_diario_devuelve_del_repo() {
        let s = svc(MockCierre::nuevo(vec![]));
        let r = s.resumen_diario("2026-09-10").unwrap();
        assert_eq!(r.total_efectivo, 1000);
    }

    #[test]
    fn resumen_diario_valida_formato_de_fecha() {
        let s = svc(MockCierre::nuevo(vec![]));
        assert!(s.resumen_diario("10-09-2026").is_err());
        assert!(s.resumen_diario("").is_err());
    }

    #[test]
    fn cerrar_caja_persiste_resumen_y_vacua() {
        let mock = MockCierre::nuevo(vec![]);
        let s = svc(mock.clone());

        let cierre = s.cerrar_caja("2026-09-10").unwrap();
        assert_eq!(cierre.fecha, "2026-09-10");
        assert_eq!(s.listar_cierres().unwrap().len(), 1);
        assert!(mock.estado.lock().unwrap().vacuar_llamado);
    }

    #[test]
    fn exportar_csv_incluye_encabezado_y_datos() {
        let mock = MockCierre::nuevo(vec![
            ("2026-09-09".to_string(), resumen(1000)),
            ("2026-09-10".to_string(), resumen(2000)),
        ]);
        let s = svc(mock);

        let csv = s.exportar_csv("", "").unwrap();
        let lineas: Vec<&str> = csv.trim().split('\n').collect();
        assert_eq!(lineas.len(), 3);
        assert!(lineas[0].starts_with("fecha,total_efectivo"));
        assert!(lineas[1].starts_with("2026-09-09,1000"));
        assert!(lineas[2].starts_with("2026-09-10,2000"));
    }

    #[test]
    fn exportar_csv_filtra_por_rango() {
        let mock = MockCierre::nuevo(vec![
            ("2026-09-08".to_string(), resumen(500)),
            ("2026-09-09".to_string(), resumen(1000)),
            ("2026-09-10".to_string(), resumen(2000)),
        ]);
        let s = svc(mock);

        let csv = s.exportar_csv("2026-09-09", "2026-09-09").unwrap();
        let lineas: Vec<&str> = csv.trim().split('\n').collect();
        assert_eq!(lineas.len(), 2); // header + 1 fila
        assert!(lineas[1].starts_with("2026-09-09"));
    }
}