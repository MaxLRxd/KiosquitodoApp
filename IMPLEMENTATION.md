# IMPLEMENTATION.md — Guía de implementación


## Cómo navegar el código

> **Nota de estado (12/09/2026)**: este documento quedó desactualizado (declaraba
> stubs en módulos ya implementados). Estado real y detalles en
> `Documentación/informe_estado_kiosquitodoapp.md`. La Fase 2 NO está cerrada:
> el backend P1–P14 está implementado, pero el árbol **no compila** (4 errores
> en `config_cmd.rs`), `lib.rs` no está wireado y Mercado Pago está diferido.

| Documento SPEC | Código relacionado |
|---|---|
| SPEC §1 Requerimientos | `KiosquitodoApp/src/routes/` (módulos POS, inventario, conciliación, cierre, configuración, devoluciones) |
| SPEC §2 Stack y arquitectura | `KiosquitodoApp/src-tauri/src/` (capas `domain`, `ports`, `application`, `infrastructure`) |
| SPEC §3 Hardware + MP | `KiosquitodoApp/src/lib/utils/scanner.ts` (escáner, implementado) y `infrastructure/mercadopago` (API de MP, **pendiente**: solo el trait `ports/mp_cliente.rs`, sin cliente reqwest) |

## Convenciones de implementación vigentes

- **Montos en centavos**: `i64` en Rust / `INTEGER` en SQLite / `number` en TS. Conversión a decimal solo en presentación (`lib/utils/moneda.ts`). La API de MP entrega decimales → `(monto * 100.0).round() as i64` a la entrada.
- **Nombrado**: camelCase en Rust para `fn`/`var` (estándar) pero elementos JSON serde en `snake_case`; TS usa `snake_case` en DTOs compartidos. Entidades de dominio y tablas de BD en español.
- **Arquitectura**: capas con dependencias unidireccionales `domain → ports → application → infrastructure`. `infrastructure/api` son comandos Tauri delgados.
- **Errores**: toda la cadena retorna `AppResult<T>` (`errors::AppError`). Los comandos convierten a `Result<T, String>` solo en la frontera IPC.
- **Persistencia**: `rusqlite` bundled, WAL activado, `foreign_keys` ON, `busy_timeout`, migraciones versionadas (`schema_version`).

## Avance por módulo (Fase 2) — estado real al 12/09/2026

| Módulo | Contenido | Avance |
|---|---|---|
| `errors/` | `AppError`, `AppResult<T>`, `From` | ✅ Implementado |
| `config/` | Rutas data/backups/logs + defaults | ✅ Implementado |
| `domain/*` | Producto, Categoria, Venta, VentaItem, Pago, Conciliacion, Cierre | ✅ Implementado (con tests) |
| `ports/*` | Traits `producto_repo`, `venta_repo`, `pago_repo`, `cierre_repo`, `mp_cliente`, `credencial_repo` | ⚠️ Traits listos; **falta `config_repo`** que `config_cmd.rs:9` importa (E0432). `existe_pago` está sin uso en producción |
| `infrastructure/db` | Conexión, PRAGMAs, `migrations.rs`, `repos/*_sqlite.rs` | ✅ Implementado (migraciones v10). **Pendientes**: bug de zona horaria en matching (`pago_sqlite.rs:93`), N+1 en `listar_ventas_dia`, `actualizar_producto` ignora stock |
| `infrastructure/api` | Comandos Tauri (producto/venta/caja/config) | ❌ Escritos pero **NO compilan** (4 errores en `config_cmd.rs`); `lib.rs` sin `invoke_handler` ni `DbConn` |
| `infrastructure/mercadopago` | Cliente reqwest paginado | ⏸ Pendiente (diferido). `mod.rs` es stub; `mp_cliente.rs` sin adaptador |
| `infrastructure/system` | Logging, backup, keyring, autostart | ⚠️ Implementado pero **sin cablear** (nadie lo invoca); plugins no registrados |
| Frontend `src/lib/utils/scanner.ts` | Discriminación escáner vs teclado | ✅ Implementado |
| Frontend `src/routes/*` | Páginas de módulos | ⏸ Placeholders "En construcción"; sin tests (`*.test.ts`/`*.spec.ts` = 0) |

## Checklist de la Fase 2 (P1–P17)

Se completa a medida que se ejecuta el plan de `Documentación/fase-2.md`:

- [x] P1 `errors/` + `config/`
- [x] P2 `domain/producto.rs`
- [x] P3 `domain/venta.rs`
- [x] P4 `domain/pago.rs` + `conciliacion.rs` + `cierre.rs`
- [x] P5 `ports/*`
- [x] P6 `infrastructure/db` (conexión + migraciones)
- [x] P7 repos `producto_sqlite.rs`
- [x] P8 repos `venta_sqlite.rs`
- [x] P9 repos `pago_sqlite.rs`
- [x] P10 repos `cierre_sqlite.rs`
- [x] P11 `application/producto_service.rs`
- [x] P12 `application/venta_service.rs`
- [x] P13 `application/conciliacion_service.rs`
- [x] P14 `application/cierre_service.rs`
- [ ] P15 `system/` + `mercadopago/client.rs` — system parcial (sin cablear); **client.rs inexistente**
- [ ] P16 `infrastructure/api/*_cmd.rs` + `lib.rs` — commands escritos, **no compilan** (4 errores), sin wiring
- [ ] P17 Integración final (tauri dev, cargo test, npm check)

**Pendientes de correctitud detectados en auditoría (12/09)**: bug de zona
horaria del matching, N+1 en `listar_ventas_dia`, `actualizar_producto` sin
stock, validación de producto duplicada, mojibake en strings de usuario,
matching sin unicidad por venta, `existe_pago` muerto, `csp: null`,
mercadopago/client.rs inexistente.

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
| 12/09/2026 | Actualización tras auditoría del commit `54b7ed1`: módulos P1–P14 marcados como implementados; estado real de P15/P16 (system sin cablear, api no compila, lib.rs sin wiring, MP diferido); hallazgos de correctitud agregados (zona horaria, N+1, stock, mojibake, CSP, etc.). Detalle en `Documentación/informe_estado_kiosquitodoapp.md` |