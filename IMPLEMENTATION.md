# IMPLEMENTATION.md — Guía de implementación

## Rol de este documento

Complementa el **SPEC** (`SPEC.md`): mientras el SPEC define **qué** se construye (reglas irrompibles R1–R12 y detalle por dominio en §1–§3), este documento registra **cómo está implementado realmente**: estructura del código, decisiones tomadas en la implementación y avance por módulo. Es la guía viva que se actualiza al ritmo de la Fase 2.

## Cómo navegar el código

| Documento SPEC | Código relacionado |
|---|---|
| SPEC §1 Requerimientos | `KiosquitodoApp/src/routes/` (módulos POS, inventario, conciliación, cierre, configuración, devoluciones) |
| SPEC §2 Stack y arquitectura | `KiosquitodoApp/src-tauri/src/` (capas `domain`, `ports`, `application`, `infrastructure`) |
| SPEC §3 Hardware + MP | `KiosquitodoApp/src/lib/utils/scanner.ts` (escáner) y `infrastructure/mercadopago` (API de MP) |

## Convenciones de implementación vigentes

- **Montos en centavos**: `i64` en Rust / `INTEGER` en SQLite / `number` en TS. Conversión a decimal solo en presentación (`lib/utils/moneda.ts`). La API de MP entrega decimales → `(monto * 100.0).round() as i64` a la entrada.
- **Nombrado**: camelCase en Rust para `fn`/`var` (estándar) pero elementos JSON serde en `snake_case`; TS usa `snake_case` en DTOs compartidos. Entidades de dominio y tablas de BD en español.
- **Arquitectura**: capas con dependencias unidireccionales `domain → ports → application → infrastructure`. `infrastructure/api` son comandos Tauri delgados.
- **Errores**: toda la cadena retorna `AppResult<T>` (`errors::AppError`). Los comandos convierten a `Result<T, String>` solo en la frontera IPC.
- **Persistencia**: `rusqlite` bundled, WAL activado, `foreign_keys` ON, `busy_timeout`, migraciones versionadas (`schema_version`).

## Avance por módulo (Fase 2)

| Módulo | Contenido | Avance |
|---|---|---|
| `errors/` | `AppError`, `AppResult<T>`, `From` | Stub |
| `config/` | Rutas data/backups/logs + defaults | Stub |
| `domain/*` | Producto, Categoria, Venta, VentaItem, Pago, Conciliacion, Cierre | Stub |
| `ports/*` | Traits `producto_repo`, `venta_repo`, `pago_repo`, `cierre_repo`, `mp_cliente`, `credencial_repo` | Stub |
| `infrastructure/db` | Conexión, PRAGMAs, `migrations.rs`, `repos/*_sqlite.rs` | Stub |
| `infrastructure/api` | Comandos Tauri | Stub |
| `infrastructure/mercadopago` | Cliente reqwest paginado | Stub |
| `infrastructure/system` | Logging, backup, keyring, autostart | Stub |
| Frontend `src/lib/utils/scanner.ts` | Discriminación escáner vs teclado | Implementado |

## Checklist de la Fase 2 (P1–P17)

Se completa a medida que se ejecuta el plan de `Documentación/fase-2.md`:

- [ ] P1 `errors/` + `config/`
- [ ] P2 `domain/producto.rs`
- [ ] P3 `domain/venta.rs`
- [ ] P4 `domain/pago.rs` + `conciliacion.rs` + `cierre.rs`
- [ ] P5 `ports/*`
- [ ] P6 `infrastructure/db` (conexión + migraciones)
- [ ] P7 repos `producto_sqlite.rs`
- [ ] P8 repos `venta_sqlite.rs`
- [ ] P9 repos `pago_sqlite.rs`
- [ ] P10 repos `cierre_sqlite.rs`
- [ ] P11 `application/producto_service.rs`
- [ ] P12 `application/venta_service.rs`
- [ ] P13 `application/conciliacion_service.rs`
- [ ] P14 `application/cierre_service.rs`
- [ ] P15 `system/` + `mercadopago/client.rs`
- [ ] P16 `infrastructure/api/*_cmd.rs` + `lib.rs`
- [ ] P17 Integración final (tauri dev, cargo test, npm check)

## Verificaciones de regresión

| Comando | Alcance |
|---|---|
| `cargo check` | Compilación backend |
| `cargo test` | Tests unitarios (BD en memoria) |
| `npm run check` | Tipos y lints del frontend |
| `npm run build` | Bundle SPA |

## Registro de cambios

| Fecha | Cambio |
|---|---|
| Septiembre 2026 | Creación del documento; backend íntegramente en stubs (Fases P1–P17 pendientes) |