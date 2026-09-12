# Graph Report - KiosquitodoApp  (2026-09-12)

## Corpus Check
- 20 files · ~80,574 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1030 nodes · 2088 edges · 73 communities (41 shown, 15 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 44 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Ventas y devoluciones
- Cierre de caja y CSV
- Dominio de pagos MP
- Productos y validación barcode
- Servicio de conciliación
- Tests del servicio de ventas
- Tests del servicio de productos
- Mocks del frontend
- Repositorio SQLite de productos
- Estado global y toasts
- Puertos de productos
- Conexión SQLite y PRAGMAs
- Deudas técnicas del backend
- Flujo del escáner de barcode
- Credenciales en keyring
- Convenciones y estrategia MP
- Metadatos del frontend
- Comandos de productos
- Módulos de la aplicación
- Arquitectura hexagonal
- Cliente API del frontend
- Configuración de Tauri
- AppConfig y directorios
- Autostart y backup
- Dependencias de desarrollo
- Dominio de conciliación
- Tipos compartidos del frontend
- Configuración TypeScript
- Scripts npm
- Skill de aprendizaje activo
- Carrito del frontend
- Hardware escáner USB HID
- Sistema logging y backups
- Tipos de SvelteKit
- Migraciones y repos SQLite
- Autostart en Rust
- Listener del escáner
- Configuración de Svelte
- Plugins Tauri npm
- Iconos de la app
- Logging en Rust
- Layout SSR
- Tipo Future
- Tipo Output
- Colisión Venta
- Tipo Arc
- Colisión Venta
- Tipo AppHandle
- Trait From
- Colisión Venta
- Tipo Connection
- Colisión Venta
- Colisión Venta
- Favicons
- Tipo MedioPago
- Nombre de paquete

## God Nodes (most connected - your core abstractions)
1. `Venta` - 35 edges
2. `VentaItem` - 26 edges
3. `ProductoSqlite` - 26 edges
4. `ejecutarMock()` - 21 edges
5. `Query: analiza el estado actual del proyecto y determina que falta por hacer` - 20 edges
6. `ejecutar()` - 19 edges
7. `VentaSqlite` - 18 edges
8. `service()` - 17 edges
9. `ProductoRepo` - 17 edges
10. `DbConn` - 16 edges

## Surprising Connections (you probably didn't know these)
- `app.html — shell HTML de SvelteKit` --conceptually_related_to--> `src/routes/* — páginas de módulos (placeholders sin tests)`  [INFERRED]
  KiosquitodoApp/src/app.html → IMPLEMENTATION.md
- `Rationale: polling /v1/payments/search (sin servidor para webhooks)` --rationale_for--> `Sync con API Mercado Pago por polling a /v1/payments/search`  [EXTRACTED]
  README.md → SPEC.md
- `infrastructure/api — comandos Tauri (producto/venta/caja/config)` --conceptually_related_to--> `Comando Rust sync_mercadopago (import + matching)`  [INFERRED]
  IMPLEMENTATION.md → SPEC.md
- `src/routes/* — páginas de módulos (placeholders sin tests)` --conceptually_related_to--> `Módulo 3 — Conciliación Mercado Pago`  [INFERRED]
  IMPLEMENTATION.md → SPEC.md
- `Tauri App Icon 128x128` --conceptually_related_to--> `Frontend Favicon`  [AMBIGUOUS]
  KiosquitodoApp/src-tauri/icons/128x128.png → static/favicon.png

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Capas del stack tecnológico del sistema de kiosco** — diagramas_stack_tecnico_kiosco_operador_scanner, diagramas_stack_tecnico_kiosco_ui, diagramas_stack_tecnico_kiosco_business_logic, diagramas_stack_tecnico_kiosco_sqlite, diagramas_stack_tecnico_kiosco_filesystem, diagramas_stack_tecnico_kiosco_hardware [EXTRACTED 1.00]
- **Flujo de escaneo de código de barras: escáner HID → teclado virtual → lógica de discriminación → Tauri IPC → Rust → SQLite → respuesta a la UI** — diagramas_flujo_escaner_barras_escanner_usb, diagramas_flujo_escaner_barras_windows_hid, diagramas_flujo_escaner_barras_webview2_svelte_ui, diagramas_flujo_escaner_barras_logica_discriminacion, diagramas_flujo_escaner_barras_tauri_ipc_buscar_producto, diagramas_flujo_escaner_barras_handler_rust, diagramas_flujo_escaner_barras_sqlite [EXTRACTED 1.00]
- **Capas del stack técnico del kiosco: operador/escáner → UI Svelte → lógica Rust Tauri → SQLite + sistema de archivos → hardware** — diagramas_stack_tecnico_kiosco_operador_escanner, diagramas_stack_tecnico_kiosco_ui_svelte_tailwind, diagramas_stack_tecnico_kiosco_rust_backend, diagramas_stack_tecnico_kiosco_sqlite_rusqlite, diagramas_stack_tecnico_kiosco_sistema_archivos, diagramas_stack_tecnico_kiosco_hardware [EXTRACTED 1.00]
- **Tauri Application Icon Set** — kiosquitodoapp_src_tauri_icons_128x128, kiosquitodoapp_src_tauri_icons_128x128_2x, kiosquitodoapp_src_tauri_icons_32x32 [EXTRACTED 1.00]
- **Estado del proyecto — frentes abiertos** — graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_backend_p1_p14, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_frontend_sin_tests, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_mercadopago_diferido, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_tests_backend, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_src_routes_placeholders [INFERRED 0.75]
- **Pendientes — que falta por hacer** — config_repo_config_repo, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_map_err, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_warnings, lib_lib, pago_sqlite_pago_sqlite, mercadopago_client_client, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_n_plus_1, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_stock, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_mojibake, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_unicidad_matching, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_existe_pago_muerto, graphify_out_memory_query_20260912_205419_ac83037e_analiza_el_estado_actual_del_proyecto_y_determina_csp_null [INFERRED 0.75]
- **Decisión de diseño: distinguir escaneo de escáner de tipeo manual mediante buffer + timer 80–100 ms + Enter** — diagramas_flujo_escaner_barras_logica_discriminacion, diagramas_flujo_escaner_barras_campo_texto_normal, diagramas_flujo_escaner_barras_rationale_timer_discriminacion [INFERRED 0.85]
- **Pipeline de escaneo de código de barras** — diagramas_flujo_escaner_barras_escanner_usb, diagramas_flujo_escaner_barras_windows_hid, diagramas_flujo_escaner_barras_webview2_svelte_ui, diagramas_flujo_escaner_barras_logica_discriminacion, diagramas_flujo_escaner_barras_tauri_ipc, diagramas_flujo_escaner_barras_handler_rust, diagramas_flujo_escaner_barras_sqlite [INFERRED 0.85]
- **Flujo de sincronización, conciliación y cierre de caja de Mercado Pago** — spec_sync_mp_polling, spec_comando_sync_mercadopago, spec_matching_automatico, spec_modulo_conciliacion_mp, spec_modulo_cierre_caja, spec_tabla_mp_pagos, spec_tabla_ventas [INFERRED 0.85]
- **Flujo de escaneo de código de barras (HID → listener → buscar_producto)** — spec_escaner_hid, spec_crearlistenerescaner, spec_comando_buscar_producto, spec_tabla_productos, spec_modulo_pos [INFERRED 0.85]
- **Stack fijo: Tauri + backend Rust + Svelte + SQLite** — spec_stack_arquitectura, spec_tauri_v2, spec_backend_rust, spec_frontend_svelte, spec_sqlite_rusqlite [INFERRED 0.95]

## Communities (73 total, 15 thin omitted)

### Community 0 - "Ventas y devoluciones"
Cohesion: 0.05
Nodes (51): Connection, calcular_margen(), calcular_total(), DevolucionRegistrada, item(), MedioPago, NuevaVenta, Option (+43 more)

### Community 1 - "Cierre de caja y CSV"
Cohesion: 0.05
Nodes (64): C, cerrar_caja_persiste_resumen_y_vacua(), CierreService, CierreService<C>, dentro_de_rango(), EstadoMock, exportar_csv_filtra_por_rango(), exportar_csv_incluye_encabezado_y_datos() (+56 more)

### Community 2 - "Dominio de pagos MP"
Cohesion: 0.06
Nodes (43): a_centavos(), descripcion_y_fecha_opcionales(), desde_api(), desde_api_elegible(), es_elegible(), MpPagoApi, pago_api(), PagoMp (+35 more)

### Community 3 - "Productos y validación barcode"
Cohesion: 0.06
Nodes (41): Display, Formatter, Into, Categoria, config_default(), estado_stock(), estado_stock_valores(), EstadoStock (+33 more)

### Community 4 - "Servicio de conciliación"
Cohesion: 0.10
Nodes (33): Future, ConciliacionService, ConciliacionService<M, P, V>, delta_dia_ignora_ventas_en_efectivo(), delta_dia_positivo_cuando_api_muestra_mas(), MockMp, MockPagos, MockVentas (+25 more)

### Community 5 - "Tests del servicio de ventas"
Cohesion: 0.12
Nodes (23): Arc, Arc<MockProductos>, item(), MockProductos, MockVentas, registrar_devolucion_rechaza_venta_inexistente(), registrar_venta_efectivo_valida_y_persiste(), registrar_venta_mp_y_mixto() (+15 more)

### Community 6 - "Tests del servicio de productos"
Cohesion: 0.14
Nodes (24): actualizar_aplica_cambios(), actualizar_rechaza_reglas_de_negocio(), ajustar_stock_valida_y_aplica(), buscar_por_barcode_devuelve_producto(), buscar_por_id_devuelve_none_si_inexistente(), crear_rechaza_barcode_duplicado(), crear_rechaza_datos_invalidos(), crear_valida_y_persiste() (+16 more)

### Community 7 - "Mocks del frontend"
Cohesion: 0.13
Nodes (29): crearCategoria(), crearCierre(), crearProducto(), crearTienda(), ejecutarMock(), horaLocalIso(), mockActualizarProducto(), mockAjustarStock() (+21 more)

### Community 8 - "Repositorio SQLite de productos"
Cohesion: 0.15
Nodes (23): ajustar_stock_registra_movimiento(), barcode_unico_rechaza_duplicado(), buscar_barcode_inexistente_devuelve_none(), buscar_por_id(), COLUMNAS_PRODUCTO, costos_de_productos_devuelve_pares(), crear_y_buscar_por_barcode(), eliminar_baja_logica() (+15 more)

### Community 9 - "Estado global y toasts"
Cohesion: 0.09
Nodes (9): appState, cerrar(), Notificacion, notificar(), TipoNotificacion, RUTA, TipoSonido, Modulo (+1 more)

### Community 10 - "Puertos de productos"
Cohesion: 0.15
Nodes (16): CategoriaRepo, ProductoRepo, AppResult, Categoria, NuevoProducto, Option, Producto, Send (+8 more)

### Community 11 - "Conexión SQLite y PRAGMAs"
Cohesion: 0.14
Nodes (17): abrir_en_archivo_aplica_wal(), abrir_en_memoria_migra(), checkpoint_wal_no_falla_en_memoria(), DbConn, Arc, Connection, Mutex, Path (+9 more)

### Community 12 - "Deudas técnicas del backend"
Cohesion: 0.13
Nodes (25): config_cmd.rs (P16, no compila — 4 errores), config_repo.rs (pendiente), Backend P1-P14 implementado, Content-Security-Policy null, Query: analiza el estado actual del proyecto y determina que falta por hacer, existe_pago: código muerto, Frontend sin tests, map_err pendiente x3 (+17 more)

### Community 13 - "Flujo del escáner de barcode"
Cohesion: 0.15
Nodes (21): Campo de texto normal (input manual), Escáner USB (HID · emula teclado), Handler Rust (SELECT barcode), Lógica de discriminación (JS · buffer + timer 80–100 ms), Decisión de diseño: si llega Enter → es escaneo; si no → es teclado manual (buffer + timer 80–100 ms), SQLite · responde en <1 ms, Tauri IPC · invoke buscar_producto, Tauri IPC · invoke("buscar_producto") · barcode string → comando Rust (+13 more)

### Community 14 - "Credenciales en keyring"
Cohesion: 0.13
Nodes (12): Entry, CUENTA, KeyringCredenciales, AppResult, Option, Self, SERVICIO, CredencialRepo (+4 more)

### Community 15 - "Convenciones y estrategia MP"
Cohesion: 0.14
Nodes (21): Convención: montos en centavos (i64 / INTEGER / number), Convención de nombrado (camelCase / snake_case DTOs / español), infrastructure/mercadopago — cliente reqwest (diferido), Precisión monetaria: montos en centavos (i64 / INTEGER), Comando Rust sync_mercadopago (import + matching), Estados de conciliación: OK / SIN COINCIDENCIA EN MP / MONTO DIFIERE, Estrategias de conciliación (A manual asistida, B matching auto, C QR), Sync MP idempotente (INSERT OR IGNORE sobre mp_id, approved+accredited) (+13 more)

### Community 16 - "Metadatos del frontend"
Cohesion: 0.10
Nodes (20): description, name, type, version, autoprefixer, jsdom, postcss, svelte (+12 more)

### Community 17 - "Comandos de productos"
Cohesion: 0.32
Nodes (19): actualizar_producto(), ajustar_stock(), buscar_producto_por_barcode(), CategoriaNueva, crear_categoria(), crear_producto(), eliminar_producto(), listar_categorias() (+11 more)

### Community 18 - "Módulos de la aplicación"
Cohesion: 0.16
Nodes (19): infrastructure/api — comandos Tauri (producto/venta/caja/config), src/routes/* — páginas de módulos (placeholders sin tests), El problema y la solución (inventario en papel → app local), Ajuste manual de stock (recuento, rotura, vencimiento), Atajos de teclado (F1–F6, Escape, Enter, Ctrl+E/M), Comando Rust buscar_producto (Tauri invoke), Módulo 4 — Cierre de Caja, Módulo 6 — Devoluciones / Notas de Crédito (+11 more)

### Community 19 - "Arquitectura hexagonal"
Cohesion: 0.15
Nodes (19): config/ — rutas data/backups/logs y defaults, domain/ — Producto, Venta, Pago, Conciliacion, Cierre, errors/ — AppError y AppResult<T>, app.html — shell HTML de SvelteKit, Decisiones clave del stack (Tauri, Rust, Svelte, SQLite, HID, polling MP), Rationale: polling /v1/payments/search (sin servidor para webhooks), Rationale: Tauri sin Chromium → 50–100 MB RAM, Arquitectura hexagonal: domain → ports → application → infrastructure (+11 more)

### Community 20 - "Cliente API del frontend"
Cohesion: 0.20
Nodes (18): actualizarProducto(), ajustarStock(), buscarProductoPorBarcode(), cerrarCaja(), crearCategoria(), crearProducto(), ejecutar(), eliminarProducto() (+10 more)

### Community 21 - "Configuración de Tauri"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 22 - "AppConfig y directorios"
Cohesion: 0.14
Nodes (15): AppConfig, BACKUP_RETENCION_DIAS, dirs_home(), home_dir_resuelto(), HTTP_TIMEOUT_SECS, LOG_RETENCION_DIAS, MATCHING_MONTO_TOLERANCIA, MATCHING_VENTANA_SEGUNDOS (+7 more)

### Community 23 - "Autostart y backup"
Cohesion: 0.24
Nodes (15): AppConfig, AppHandle, From, autostart_estado(), crear_backup_manual(), deshabilitar_autostart(), habilitar_autostart(), limpiar_backups_viejos() (+7 more)

### Community 24 - "Dependencias de desarrollo"
Cohesion: 0.12
Nodes (17): devDependencies, autoprefixer, jsdom, postcss, svelte, svelte-check, @sveltejs/adapter-static, @sveltejs/kit (+9 more)

### Community 25 - "Dominio de conciliación"
Cohesion: 0.13
Nodes (5): Candidato, es_candidato_valido(), estado_conciliacion(), EstadoConciliacion, Option

### Community 26 - "Tipos compartidos del frontend"
Cohesion: 0.21
Nodes (12): TiendaMock, Categoria, CategoriaNueva, CierrePersistido, DevolucionRegistrada, ItemVenta, MedioPago, MpPago (+4 more)

### Community 27 - "Configuración TypeScript"
Cohesion: 0.15
Nodes (12): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, skipLibCheck (+4 more)

### Community 28 - "Scripts npm"
Cohesion: 0.18
Nodes (11): scripts, build, check, check:watch, dev, preview, tauri, tauri:build (+3 more)

### Community 29 - "Skill de aprendizaje activo"
Cohesion: 0.46
Nodes (8): SKILL.md — Aprendizaje activo con IA (skill de opencode), Calibración según nivel del usuario (junior vs experto), Diagnóstico rápido: modo 'nuevo' vs modo 'rutina', Modo 'es nuevo para mí', Modo 'esto es rutina', Proteger el esfuerzo de generar (aprender haciendo), Revisión crítica, no aceptación pasiva, Tutor socrático, no oráculo

### Community 30 - "Carrito del frontend"
Cohesion: 0.36
Nodes (6): carrito, crearCarrito(), ItemCarrito, producto(), Producto, vitest

### Community 31 - "Hardware escáner USB HID"
Cohesion: 0.38
Nodes (7): src/lib/utils/scanner.ts — discriminación escáner vs teclado, Escáneres compatibles (Prosoft S2100, Titanika, Nictom — USB HID), Rationale: escáner HID nativo + timer JS (sin driver), crearListenerEscaner (buffer temporizado 100 ms, scanner.ts), Escáner USB HID (emulación de teclado, EAN-13), Integración de hardware: escáner USB 1D y API MP (sección 03), Rationale: buffer temporizado para discriminar escáner vs tipeo por velocidad

### Community 32 - "Sistema logging y backups"
Cohesion: 0.33
Nodes (7): infrastructure/system — logging, backup, keyring, autostart, Backup automático diario con wal_checkpoint(TRUNCATE), Sistema de logging rotativo (log + flexi_logger), Módulo 5 — Configuración, Rationale: PRAGMA wal_checkpoint antes de copiar el .db, Rationale: restore exige reinicio (SQLite no permite reemplazo con conexión abierta), Restore de backups (copia + reinicio de app)

### Community 33 - "Tipos de SvelteKit"
Cohesion: 0.29
Nodes (6): App, Error, Locals, PageData, PageState, Platform

### Community 34 - "Migraciones y repos SQLite"
Cohesion: 0.40
Nodes (6): infrastructure/db — conexión, PRAGMAs, migrations.rs, repos_sqlite, Migraciones versionadas de BD (v10), ports/ — traits producto_repo, venta_repo, pago_repo, cierre_repo, mp_cliente, Migraciones de esquema versionadas (MIGRATIONS v1–v10), SQLite con WAL, foreign_keys ON, busy_timeout y migraciones versionadas, Tabla schema_version (migraciones aplicadas)

### Community 35 - "Autostart en Rust"
Cohesion: 0.60
Nodes (5): autostart_habilitado(), deshabilitar_autostart(), habilitar_autostart(), AppHandle, AppResult

### Community 36 - "Listener del escáner"
Cohesion: 0.40
Nodes (3): CallbackEscaneo, crearListenerEscaneo(), TECLAS_CONTROL

### Community 37 - "Configuración de Svelte"
Cohesion: 0.40
Nodes (3): config, @sveltejs/adapter-static, @sveltejs/vite-plugin-svelte

### Community 38 - "Plugins Tauri npm"
Cohesion: 0.50
Nodes (4): dependencies, @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs

### Community 39 - "Iconos de la app"
Cohesion: 0.67
Nodes (4): Tauri App Icon 128x128, Tauri App Icon 128x128@2x (HiDPI), Tauri App Icon 32x32, Frontend Favicon

### Community 40 - "Logging en Rust"
Cohesion: 0.83
Nodes (3): init_logging(), AppResult, Path

## Ambiguous Edges - Review These
- `Tauri App Icon 128x128` → `Frontend Favicon`  [AMBIGUOUS]
  KiosquitodoApp/src-tauri/icons/128x128.png · relation: conceptually_related_to
- `Sistema de archivos` → `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)`  [AMBIGUOUS]
  diagramas/stack_tecnico_kiosco.svg · relation: conceptually_related_to
- `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` → `SQLite (vía rusqlite)`  [AMBIGUOUS]
  diagramas/stack_tecnico_kiosco.svg · relation: conceptually_related_to

## Knowledge Gaps
- **128 isolated node(s):** `CallbackEscaneo`, `Error`, `Locals`, `PageData`, `PageState` (+123 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 304 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Tauri App Icon 128x128` and `Frontend Favicon`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Sistema de archivos` and `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` and `SQLite (vía rusqlite)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `String` connect `Productos y validación barcode` to `Cierre de caja y CSV`, `Dominio de pagos MP`, `Conexión SQLite y PRAGMAs`, `Credenciales en keyring`?**
  _High betweenness centrality (0.104) - this node is a cross-community bridge._
- **Why does `ProductoSqlite` connect `Repositorio SQLite de productos` to `Ventas y devoluciones`, `Cierre de caja y CSV`, `Dominio de pagos MP`, `Puertos de productos`, `Comandos de productos`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **Why does `PagoMp` connect `Dominio de pagos MP` to `Productos y validación barcode`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._
- **What connects `CallbackEscaneo`, `Error`, `Locals` to the rest of the system?**
  _128 weakly-connected nodes found - possible documentation gaps or missing edges._