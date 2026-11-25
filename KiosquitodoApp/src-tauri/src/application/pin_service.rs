//! Servicio de PIN: persiste solo `hash(salt || pin)`, nunca el PIN en claro.
//!
//! El valor guardado en la clave `pin_hash_salt` tiene el formato
//! `<salt_hex>:<hash_hex>`. El salt se pide al sistema operativo con
//! `getrandom` (16 bytes) y se guarda junto al hash para poder recalcularlo.
//! Así, dos kioscos con el mismo PIN producen valores distintos.

use crate::errors::{AppError, AppResult};
use crate::ports::config_repo::ConfigRepo;
use getrandom::getrandom;
use sha2::{Digest, Sha256};

const CLAVE_PIN: &str = "pin_hash_salt";
const LONGITUD_SALT: usize = 16;
const LONGITUD_HASH: usize = 32;

/// Casos de uso del PIN sobre un repositorio clave/valor.
pub struct PinService<R: ConfigRepo> {
    repo: R,
}

impl<R: ConfigRepo> PinService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Devuelve `true` si ya hay un PIN configurado. Nunca expone el hash.
    pub fn configurado(&self) -> AppResult<bool> {
        Ok(self.repo.obtener(CLAVE_PIN)?.is_some())
    }

    /// Valida, genera un salt nuevo y guarda el hash. Sin PIN en claro.
    pub fn establecer(&self, pin: &str) -> AppResult<()> {
        validar_pin(pin)?;

        let mut salt = [0u8; LONGITUD_SALT];
        getrandom(&mut salt)
            .map_err(|e| AppError::negocio(format!("No se pudo generar el salt aleatorio: {e}")))?;

        let hash = calcular_hash(&salt, pin);
        let guardado = format!("{}:{}", a_hex(&salt), a_hex(&hash));
        self.repo.guardar(CLAVE_PIN, &guardado)
    }

    /// Recalcula el hash con el salt guardado y compara. Sin PIN => `false`.
    pub fn verificar(&self, pin: &str) -> AppResult<bool> {
        let Some(guardado) = self.repo.obtener(CLAVE_PIN)? else {
            return Ok(false);
        };
        if !es_pin_valido(pin) {
            return Ok(false);
        }

        let (salt_hex, hash_hex) = guardado
            .split_once(':')
            .ok_or_else(|| AppError::negocio("Valor de PIN corrupto (falta el separador)"))?;
        let salt = de_hex(salt_hex)
            .ok_or_else(|| AppError::negocio("Valor de PIN corrupto (salt inválido)"))?;
        let hash_guardado = de_hex(hash_hex)
            .ok_or_else(|| AppError::negocio("Valor de PIN corrupto (hash inválido)"))?;

        if salt.len() != LONGITUD_SALT || hash_guardado.len() != LONGITUD_HASH {
            return Err(AppError::negocio(
                "Valor de PIN corrupto (longitud inválida)",
            ));
        }

        let hash_calculado = calcular_hash(&salt, pin);
        Ok(&hash_calculado[..] == hash_guardado.as_slice())
    }

    /// Elimina el PIN. Útil para desactivar el bloqueo sin quedar afuera.
    pub fn eliminar(&self) -> AppResult<()> {
        self.repo.eliminar(CLAVE_PIN)
    }
}

/// SHA-256 sobre `salt || pin`. Extraído para poder contrastarlo con vectores
/// conocidos en los tests (con salt vacío, `abc` da el vector oficial).
fn calcular_hash(salt: &[u8], pin: &str) -> [u8; LONGITUD_HASH] {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(pin.as_bytes());
    let resultado = hasher.finalize();

    let mut hash = [0u8; LONGITUD_HASH];
    hash.copy_from_slice(&resultado);
    hash
}

fn validar_pin(pin: &str) -> AppResult<()> {
    if es_pin_valido(pin) {
        Ok(())
    } else {
        Err(AppError::negocio("El PIN debe tener entre 4 y 6 dígitos"))
    }
}

fn es_pin_valido(pin: &str) -> bool {
    (4..=6).contains(&pin.len()) && pin.bytes().all(|b| b.is_ascii_digit())
}

fn a_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn de_hex(texto: &str) -> Option<Vec<u8>> {
    if texto.len() % 2 != 0 {
        return None;
    }
    texto
        .as_bytes()
        .chunks_exact(2)
        .map(|par| Some((valor_hex(par[0])? << 4) | valor_hex(par[1])?))
        .collect()
}

fn valor_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RepoMemoria {
        valores: Mutex<HashMap<String, String>>,
    }

    impl ConfigRepo for RepoMemoria {
        fn obtener(&self, clave: &str) -> AppResult<Option<String>> {
            Ok(self.valores.lock().unwrap().get(clave).cloned())
        }

        fn guardar(&self, clave: &str, valor: &str) -> AppResult<()> {
            self.valores
                .lock()
                .unwrap()
                .insert(clave.to_string(), valor.to_string());
            Ok(())
        }

        fn eliminar(&self, clave: &str) -> AppResult<()> {
            self.valores.lock().unwrap().remove(clave);
            Ok(())
        }
    }

    fn servicio() -> PinService<RepoMemoria> {
        PinService::new(RepoMemoria::default())
    }

    #[test]
    fn hash_coincide_con_vector_sha256_conocido() {
        // SHA-256("abc") publicado por NIST.
        assert_eq!(
            a_hex(&calcular_hash(b"", "abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn arranca_sin_pin_y_no_verifica() {
        let svc = servicio();
        assert!(!svc.configurado().unwrap());
        assert!(!svc.verificar("1234").unwrap());
    }

    #[test]
    fn establecer_y_verificar() {
        let svc = servicio();
        svc.establecer("1234").unwrap();
        assert!(svc.configurado().unwrap());
        assert!(svc.verificar("1234").unwrap());
        assert!(!svc.verificar("9999").unwrap());
    }

    #[test]
    fn guarda_salt_y_hash_pero_no_el_pin() {
        let svc = servicio();
        svc.establecer("1234").unwrap();
        let guardado = svc.repo.obtener(CLAVE_PIN).unwrap().unwrap();
        let partes: Vec<&str> = guardado.split(':').collect();
        assert_eq!(partes.len(), 2);
        assert_eq!(partes[0].len(), LONGITUD_SALT * 2);
        assert_eq!(partes[1].len(), LONGITUD_HASH * 2);
        assert_ne!(guardado, "1234");
    }

    #[test]
    fn cada_establecimiento_usa_un_salt_nuevo() {
        let svc = servicio();
        svc.establecer("1234").unwrap();
        let primero = svc.repo.obtener(CLAVE_PIN).unwrap().unwrap();
        svc.establecer("1234").unwrap();
        let segundo = svc.repo.obtener(CLAVE_PIN).unwrap().unwrap();
        assert_ne!(primero, segundo);
        assert!(svc.verificar("1234").unwrap());
    }

    #[test]
    fn valida_largo_y_digitos() {
        let svc = servicio();
        assert!(svc.establecer("123").is_err());
        assert!(svc.establecer("1234567").is_err());
        assert!(svc.establecer("12ab").is_err());
        assert!(svc.establecer("").is_err());
    }

    #[test]
    fn eliminar_desactiva_el_pin() {
        let svc = servicio();
        svc.establecer("1234").unwrap();
        svc.eliminar().unwrap();
        svc.eliminar().unwrap();
        assert!(!svc.configurado().unwrap());
        assert!(!svc.verificar("1234").unwrap());
    }

    #[test]
    fn valor_corrupto_se_reporta() {
        let svc = servicio();
        svc.repo.guardar(CLAVE_PIN, "no-es-hex").unwrap();
        assert!(svc.verificar("1234").is_err());
    }
}
