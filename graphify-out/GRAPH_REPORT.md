# Graph Report - KiosquitodoApp  (2026-09-12)

## Corpus Check
- 15 files · ~79,518 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 969 nodes · 2003 edges · 61 communities (40 shown, 4 thin omitted)
- Extraction: 97% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 50 edges (avg confidence: 0.86)
- Token cost: 1,024 input · 3,072 output

## Community Hubs (Navigation)
- Conciliation Matching Service
- Common App Types & Errors
- Venta Pricing & Margins
- Product Service Tests
- Cierre de Caja Service
- Venta Service Mocks
- Cierre SQLite Persistence
- Product SQLite Persistence
- Keyring Credentials Store
- Frontend Package Metadata
- Docs: Tauri Commands & Pages
- Tauri App Configuration
- Docs: Architecture & Rationale
- Frontend Dev Dependencies
- Config Module Resolution
- Conciliation Domain Logic
- Product Repository Ports
- Docs: Scanner Input Design
- Docs: MercadoPago Reconciliation
- Docs: Infrastructure Conventions
- Toast & App State
- TypeScript Compiler Config
- Payment Conversion Logic
- SQLite Migrations
- Frontend Scripts
- Backup Management
- Learning Skill Docs
- Docs: Scanner Hardware Integration
- System Infrastructure Modules
- SvelteKit App Types
- Frontend Domain Types
- Project Docs Spec & README
- Autostart Module
- Scanner Listener
- Svelte Config
- App Icons & Favicon
- Logging Init
- Sound Effects
- Tauri Runtime Deps
- API Client
- Date Formatting
- SSR Layout
- MercadoPago Module
- System Module

## God Nodes (most connected - your core abstractions)
1. `DbConn` - 40 edges
2. `PagoMp` - 28 edges
3. `VentaItem` - 25 edges
4. `ProductoSqlite` - 22 edges
5. `ejecutarMock()` - 21 edges
6. `Query: analiza el estado actual del proyecto y determina que falta por hacer` - 20 edges
7. `ejecutar()` - 19 edges
8. `ResumenCierre` - 18 edges
9. `service()` - 17 edges
10. `ProductoRepo` - 17 edges

## Surprising Connections (you probably didn't know these)
- `infrastructure/api — comandos Tauri (producto/venta/caja/config)` --conceptually_related_to--> `Comando Rust sync_mercadopago (import + matching)`  [INFERRED]
  IMPLEMENTATION.md → SPEC.md
- `src/routes/* — páginas de módulos (placeholders sin tests)` --conceptually_related_to--> `Módulo 3 — Conciliación Mercado Pago`  [INFERRED]
  IMPLEMENTATION.md → SPEC.md
- `app.html — shell HTML de SvelteKit` --conceptually_related_to--> `src/routes/* — páginas de módulos (placeholders sin tests)`  [INFERRED]
  KiosquitodoApp/src/app.html → IMPLEMENTATION.md
- `Rationale: polling /v1/payments/search (sin servidor para webhooks)` --rationale_for--> `Sync con API Mercado Pago por polling a /v1/payments/search`  [EXTRACTED]
  README.md → SPEC.md
- `Rationale: escáner HID nativo + timer JS (sin driver)` --rationale_for--> `Escáner USB HID (emulación de teclado, EAN-13)`  [EXTRACTED]
  README.md → SPEC.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Flujo de escaneo de código de barras: escáner HID → teclado virtual → lógica de discriminación → Tauri IPC → Rust → SQLite → respuesta a la UI** — diagramas_flujo_escaner_barras_escanner_usb, diagramas_flujo_escaner_barras_windows_hid, diagramas_flujo_escaner_barras_webview2_svelte_ui, diagramas_flujo_escaner_barras_logica_discriminacion, diagramas_flujo_escaner_barras_tauri_ipc_buscar_producto, diagramas_flujo_escaner_barras_handler_rust, diagramas_flujo_escaner_barras_sqlite [EXTRACTED 1.00]
- **Capas del stack técnico del kiosco: operador/escáner → UI Svelte → lógica Rust Tauri → SQLite + sistema de archivos → hardware** — diagramas_stack_tecnico_kiosco_operador_escanner, diagramas_stack_tecnico_kiosco_ui_svelte_tailwind, diagramas_stack_tecnico_kiosco_rust_backend, diagramas_stack_tecnico_kiosco_sqlite_rusqlite, diagramas_stack_tecnico_kiosco_sistema_archivos, diagramas_stack_tecnico_kiosco_hardware [EXTRACTED 1.00]
- **Tauri Application Icon Set** — kiosquitodoapp_src_tauri_icons_128x128, kiosquitodoapp_src_tauri_icons_128x128_2x, kiosquitodoapp_src_tauri_icons_32x32 [EXTRACTED 1.00]
- **Decisión de diseño: distinguir escaneo de escáner de tipeo manual mediante buffer + timer 80–100 ms + Enter** — diagramas_flujo_escaner_barras_logica_discriminacion, diagramas_flujo_escaner_barras_campo_texto_normal, diagramas_flujo_escaner_barras_rationale_timer_discriminacion [INFERRED 0.85]
- **Flujo de sincronización, conciliación y cierre de caja de Mercado Pago** — spec_sync_mp_polling, spec_comando_sync_mercadopago, spec_matching_automatico, spec_modulo_conciliacion_mp, spec_modulo_cierre_caja, spec_tabla_mp_pagos, spec_tabla_ventas [INFERRED 0.85]
- **Flujo de escaneo de código de barras (HID → listener → buscar_producto)** — spec_escaner_hid, spec_crearlistenerescaner, spec_comando_buscar_producto, spec_tabla_productos, spec_modulo_pos [INFERRED 0.85]
- **Stack fijo: Tauri + backend Rust + Svelte + SQLite** — spec_stack_arquitectura, spec_tauri_v2, spec_backend_rust, spec_frontend_svelte, spec_sqlite_rusqlite [INFERRED 0.95]
- **Pendientes — que falta por hacer** — config_repo_config_repo, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_map_err, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_warnings, lib_lib, pago_sqlite_pago_sqlite, mercadopago_client_client, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_n_plus_1, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_stock, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_mojibake, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_unicidad_matching, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_existe_pago_muerto, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_csp_null [INFERRED 0.75]
- **Estado del proyecto — frentes abiertos** — graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_backend_p1_p14, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_frontend_sin_tests, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_mercadopago_diferido, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_tests_backend, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_src_routes_placeholders [INFERRED 0.75]
- **Pipeline de escaneo de código de barras** — diagramas_flujo_escaner_barras_escanner_usb, diagramas_flujo_escaner_barras_windows_hid, diagramas_flujo_escaner_barras_webview2_svelte_ui, diagramas_flujo_escaner_barras_logica_discriminacion, diagramas_flujo_escaner_barras_tauri_ipc, diagramas_flujo_escaner_barras_handler_rust, diagramas_flujo_escaner_barras_sqlite [INFERRED 0.85]
- **Capas del stack tecnológico del sistema de kiosco** — diagramas_stack_tecnico_kiosco_operador_scanner, diagramas_stack_tecnico_kiosco_ui, diagramas_stack_tecnico_kiosco_business_logic, diagramas_stack_tecnico_kiosco_sqlite, diagramas_stack_tecnico_kiosco_filesystem, diagramas_stack_tecnico_kiosco_hardware [EXTRACTED 1.00]

## Communities (61 total, 4 thin omitted)

### Community 0 - "Conciliation Matching Service"
Cohesion: 0.05
Nodes (63): Display, Formatter, Into, AppError, display_db_error(), display_negocio(), From, Result (+55 more)

### Community 1 - "Common App Types & Errors"
Cohesion: 0.07
Nodes (45): ConciliacionService, ConciliacionService<M, P, V>, delta_dia_ignora_ventas_en_efectivo(), delta_dia_positivo_cuando_api_muestra_mas(), MockMp, MockPagos, MockVentas, pago() (+37 more)

### Community 2 - "Venta Pricing & Margins"
Cohesion: 0.06
Nodes (34): Arc<MockProductos>, item(), MockProductos, MockVentas, registrar_devolucion_rechaza_venta_inexistente(), registrar_venta_efectivo_valida_y_persiste(), registrar_venta_mp_y_mixto(), registrar_venta_rechaza_medio_incoherente() (+26 more)

### Community 3 - "Product Service Tests"
Cohesion: 0.07
Nodes (43): actualizar_aplica_cambios(), actualizar_rechaza_reglas_de_negocio(), ajustar_stock_valida_y_aplica(), buscar_por_barcode_devuelve_producto(), buscar_por_id_devuelve_none_si_inexistente(), crear_rechaza_barcode_duplicado(), crear_rechaza_datos_invalidos(), crear_valida_y_persiste() (+35 more)

### Community 4 - "Cierre de Caja Service"
Cohesion: 0.08
Nodes (41): C, cerrar_caja_persiste_resumen_y_vacua(), CierreService, CierreService<C>, dentro_de_rango(), EstadoMock, exportar_csv_filtra_por_rango(), exportar_csv_incluye_encabezado_y_datos() (+33 more)

### Community 5 - "Venta Service Mocks"
Cohesion: 0.17
Nodes (22): crear_producto(), crear_venta_con_stock_insuficiente_rollback(), crear_venta_descuenta_stock_en_transaccion(), devolucion_recupera_stock(), devolucion_registra_movimiento(), fila_a_venta(), listar_ventas_dia(), medio_pago_a_db() (+14 more)

### Community 6 - "Cierre SQLite Persistence"
Cohesion: 0.16
Nodes (21): crear_producto_para_venta(), crear_venta_mp(), existe_pago(), fila_a_pago(), matching_automatico_une_por_monto_y_tiempo(), matching_no_vincula_monto_muy_distinto(), obtener_por_mp_id(), pago() (+13 more)

### Community 7 - "Product SQLite Persistence"
Cohesion: 0.18
Nodes (20): ajustar_stock_registra_movimiento(), barcode_unico_rechaza_duplicado(), buscar_barcode_inexistente_devuelve_none(), buscar_por_id(), COLUMNAS_PRODUCTO, costos_de_productos_devuelve_pares(), crear_y_buscar_por_barcode(), eliminar_baja_logica() (+12 more)

### Community 8 - "Keyring Credentials Store"
Cohesion: 0.12
Nodes (21): crearCierre(), ejecutarMock(), horaLocalIso(), mockAjustarStock(), mockBuscarProductoPorBarcode(), mockCerrarCaja(), mockCrearCategoria(), mockEliminarProducto() (+13 more)

### Community 9 - "Frontend Package Metadata"
Cohesion: 0.11
Nodes (8): appState, cerrar(), Notificacion, notificar(), TipoNotificacion, TipoSonido, Modulo, MODULOS

### Community 10 - "Docs: Tauri Commands & Pages"
Cohesion: 0.17
Nodes (20): crearCategoria(), crearProducto(), crearTienda(), mockActualizarProducto(), mockCrearProducto(), reiniciarTiendaMock(), TiendaMock, validarBarcode() (+12 more)

### Community 11 - "Tauri App Configuration"
Cohesion: 0.12
Nodes (18): AppConfig, BACKUP_RETENCION_DIAS, dirs_home(), home_dir_resuelto(), HTTP_TIMEOUT_SECS, LOG_RETENCION_DIAS, MATCHING_MONTO_TOLERANCIA, MATCHING_VENTANA_SEGUNDOS (+10 more)

### Community 12 - "Docs: Architecture & Rationale"
Cohesion: 0.16
Nodes (21): config_cmd.rs (P16, no compila — 4 errores), config_repo.rs (pendiente), Backend P1-P14 implementado, Content-Security-Policy null, Query: analiza el estado actual del proyecto y determina que falta por hacer, existe_pago: código muerto, Frontend sin tests, map_err pendiente x3 (+13 more)

### Community 13 - "Frontend Dev Dependencies"
Cohesion: 0.15
Nodes (21): Campo de texto normal (input manual), Escáner USB (HID · emula teclado), Handler Rust (SELECT barcode), Lógica de discriminación (JS · buffer + timer 80–100 ms), Decisión de diseño: si llega Enter → es escaneo; si no → es teclado manual (buffer + timer 80–100 ms), SQLite · responde en <1 ms, Tauri IPC · invoke buscar_producto, Tauri IPC · invoke("buscar_producto") · barcode string → comando Rust (+13 more)

### Community 14 - "Config Module Resolution"
Cohesion: 0.13
Nodes (12): Entry, CUENTA, KeyringCredenciales, AppResult, Option, Self, SERVICIO, CredencialRepo (+4 more)

### Community 15 - "Conciliation Domain Logic"
Cohesion: 0.14
Nodes (20): config/ — rutas data/backups/logs y defaults, domain/ — Producto, Venta, Pago, Conciliacion, Cierre, errors/ — AppError y AppResult<T>, app.html — shell HTML de SvelteKit, Decisiones clave del stack (Tauri, Rust, Svelte, SQLite, HID, polling MP), Rationale: escáner HID nativo + timer JS (sin driver), Rationale: polling /v1/payments/search (sin servidor para webhooks), Rationale: Tauri sin Chromium → 50–100 MB RAM (+12 more)

### Community 16 - "Product Repository Ports"
Cohesion: 0.16
Nodes (19): infrastructure/api — comandos Tauri (producto/venta/caja/config), src/routes/* — páginas de módulos (placeholders sin tests), El problema y la solución (inventario en papel → app local), Ajuste manual de stock (recuento, rotura, vencimiento), Atajos de teclado (F1–F6, Escape, Enter, Ctrl+E/M), Comando Rust buscar_producto (Tauri invoke), Módulo 4 — Cierre de Caja, Módulo 6 — Devoluciones / Notas de Crédito (+11 more)

### Community 17 - "Docs: Scanner Input Design"
Cohesion: 0.11
Nodes (18): description, name, type, version, autoprefixer, jsdom, postcss, svelte (+10 more)

### Community 18 - "Docs: MercadoPago Reconciliation"
Cohesion: 0.20
Nodes (18): actualizarProducto(), ajustarStock(), buscarProductoPorBarcode(), cerrarCaja(), crearCategoria(), crearProducto(), ejecutar(), eliminarProducto() (+10 more)

### Community 19 - "Docs: Infrastructure Conventions"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 20 - "Toast & App State"
Cohesion: 0.12
Nodes (17): devDependencies, autoprefixer, jsdom, postcss, svelte, svelte-check, @sveltejs/adapter-static, @sveltejs/kit (+9 more)

### Community 21 - "TypeScript Compiler Config"
Cohesion: 0.13
Nodes (5): Candidato, es_candidato_valido(), estado_conciliacion(), EstadoConciliacion, Option

### Community 22 - "Payment Conversion Logic"
Cohesion: 0.27
Nodes (7): ProductoRepo, AppResult, Option, Producto, Send, Sync, Vec

### Community 23 - "SQLite Migrations"
Cohesion: 0.17
Nodes (9): carrito, crearCarrito(), ItemCarrito, producto(), Producto, CallbackEscaneo, crearListenerEscaneo(), TECLAS_CONTROL (+1 more)

### Community 24 - "Frontend Scripts"
Cohesion: 0.19
Nodes (14): Convención: montos en centavos (i64 / INTEGER / number), Precisión monetaria: montos en centavos (i64 / INTEGER), Comando Rust sync_mercadopago (import + matching), Estados de conciliación: OK / SIN COINCIDENCIA EN MP / MONTO DIFIERE, Estrategias de conciliación (A manual asistida, B matching auto, C QR), Indicador de conectividad (deducido del último sync), Matching automático: monto ±100 centavos en ventana ±10 min, Módulo 3 — Conciliación Mercado Pago (+6 more)

### Community 25 - "Backup Management"
Cohesion: 0.21
Nodes (13): Convención de nombrado (camelCase / snake_case DTOs / español), infrastructure/db — conexión, PRAGMAs, migrations.rs, repos_sqlite, infrastructure/mercadopago — cliente reqwest (diferido), Migraciones versionadas de BD (v10), ports/ — traits producto_repo, venta_repo, pago_repo, cierre_repo, mp_cliente, Sync MP idempotente (INSERT OR IGNORE sobre mp_id, approved+accredited), Credenciales MP cifradas en keyring (Windows Credential Store), Migraciones de esquema versionadas (MIGRATIONS v1–v10) (+5 more)

### Community 26 - "Learning Skill Docs"
Cohesion: 0.27
Nodes (11): AportePagoMp, AporteVenta, calcular_resumen(), delta_mp_cero_cuando_cuadra(), delta_mp_negativo_cuando_falta_acreditar(), delta_mp_positivo_cuando_api_muestra_mas(), resumen_con_ventas_mixtas_y_efectivo(), resumen_vacia() (+3 more)

### Community 27 - "Docs: Scanner Hardware Integration"
Cohesion: 0.28
Nodes (7): AppResult, Option, Send, Sync, Vec, Venta, VentaRepo

### Community 28 - "System Infrastructure Modules"
Cohesion: 0.15
Nodes (12): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, skipLibCheck (+4 more)

### Community 29 - "SvelteKit App Types"
Cohesion: 0.30
Nodes (8): a_centavos(), descripcion_y_fecha_opcionales(), desde_api(), desde_api_elegible(), es_elegible(), MpPagoApi, pago_api(), Option

### Community 30 - "Frontend Domain Types"
Cohesion: 0.35
Nodes (11): bd_en_memoria(), bd_reciente_solo_aplica_pendientes(), ejecutar_migraciones(), migraciones_son_idempotentes(), migrar_bd_vacia_crea_esquema_completo(), MIGRATIONS, productos_tiene_categoria_y_restricciones(), Connection (+3 more)

### Community 31 - "Project Docs Spec & README"
Cohesion: 0.18
Nodes (11): scripts, build, check, check:watch, dev, preview, tauri, tauri:build (+3 more)

### Community 32 - "Autostart Module"
Cohesion: 0.29
Nodes (10): IMPLEMENTATION.md — Guía de implementación, src/lib/utils/scanner.ts — discriminación escáner vs teclado, README.md — Sistema de Gestión de Kiosco (Documentación Técnica), Escáneres compatibles (Prosoft S2100, Titanika, Nictom — USB HID), Metodología SPEC-driven con IA, crearListenerEscaner (buffer temporizado 100 ms, scanner.ts), Escáner USB HID (emulación de teclado, EAN-13), SPEC.md — Especificación del sistema (fuente de verdad) (+2 more)

### Community 33 - "Scanner Listener"
Cohesion: 0.46
Nodes (8): SKILL.md — Aprendizaje activo con IA (skill de opencode), Calibración según nivel del usuario (junior vs experto), Diagnóstico rápido: modo 'nuevo' vs modo 'rutina', Modo 'es nuevo para mí', Modo 'esto es rutina', Proteger el esfuerzo de generar (aprender haciendo), Revisión crítica, no aceptación pasiva, Tutor socrático, no oráculo

### Community 34 - "Svelte Config"
Cohesion: 0.33
Nodes (7): infrastructure/system — logging, backup, keyring, autostart, Backup automático diario con wal_checkpoint(TRUNCATE), Sistema de logging rotativo (log + flexi_logger), Módulo 5 — Configuración, Rationale: PRAGMA wal_checkpoint antes de copiar el .db, Rationale: restore exige reinicio (SQLite no permite reemplazo con conexión abierta), Restore de backups (copia + reinicio de app)

### Community 35 - "App Icons & Favicon"
Cohesion: 0.29
Nodes (6): App, Error, Locals, PageData, PageState, Platform

### Community 36 - "Logging Init"
Cohesion: 0.60
Nodes (5): autostart_habilitado(), deshabilitar_autostart(), habilitar_autostart(), AppHandle, AppResult

### Community 37 - "Sound Effects"
Cohesion: 0.40
Nodes (3): config, @sveltejs/adapter-static, @sveltejs/vite-plugin-svelte

### Community 38 - "Tauri Runtime Deps"
Cohesion: 0.67
Nodes (4): Tauri App Icon 128x128, Tauri App Icon 128x128@2x (HiDPI), Tauri App Icon 32x32, Frontend Favicon

### Community 39 - "API Client"
Cohesion: 0.83
Nodes (3): init_logging(), AppResult, Path

## Ambiguous Edges - Review These
- `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` → `Sistema de archivos`  [AMBIGUOUS]
  diagramas/stack_tecnico_kiosco.svg · relation: conceptually_related_to
- `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` → `SQLite (vía rusqlite)`  [AMBIGUOUS]
  diagramas/stack_tecnico_kiosco.svg · relation: conceptually_related_to
- `Tauri App Icon 128x128` → `Frontend Favicon`  [AMBIGUOUS]
  KiosquitodoApp/src-tauri/icons/128x128.png · relation: conceptually_related_to

## Knowledge Gaps
- **123 isolated node(s):** `Notificacion`, `TipoNotificacion`, `Modulo`, `Error`, `Locals` (+118 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 283 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` and `Sistema de archivos`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` and `SQLite (vía rusqlite)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Tauri App Icon 128x128` and `Frontend Favicon`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `String` connect `Conciliation Matching Service` to `Common App Types & Errors`, `Venta Pricing & Margins`, `Product Service Tests`, `Cierre de Caja Service`, `Tauri App Configuration`, `Config Module Resolution`, `SvelteKit App Types`, `Frontend Domain Types`?**
  _High betweenness centrality (0.140) - this node is a cross-community bridge._
- **Why does `DbConn` connect `Conciliation Matching Service` to `Cierre de Caja Service`, `Venta Service Mocks`, `Cierre SQLite Persistence`, `Product SQLite Persistence`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Why does `PagoMp` connect `Common App Types & Errors` to `Conciliation Matching Service`, `SvelteKit App Types`, `Cierre SQLite Persistence`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **What connects `Notificacion`, `TipoNotificacion`, `Modulo` to the rest of the system?**
  _123 weakly-connected nodes found - possible documentation gaps or missing edges._