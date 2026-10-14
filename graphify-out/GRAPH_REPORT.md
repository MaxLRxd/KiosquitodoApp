# Graph Report - KiosquitodoApp  (2026-10-13)

## Corpus Check
- 17 files · ~81,360 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1059 nodes · 2092 edges · 76 communities (43 shown, 16 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 42 edges (avg confidence: 0.87)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Conciliación Mercado Pago
- Cierre de Caja
- Dominio Ventas y Pagos
- Dominio Producto y Barcode
- Venta Service
- Producto Service
- Repositorio SQLite de Ventas
- Mocks del Frontend
- AppConfig y Directorios
- Repositorio SQLite de Productos
- Repositorio SQLite de Pagos
- Credenciales en Keyring
- Estado Global y Toasts
- Repositorio SQLite de Cierres
- Conexión SQLite y PRAGMAs
- Deudas Técnicas del Backend
- Flujo del Escáner de Barcode
- Convenciones y Estrategia MP
- Metadatos del Frontend
- Puertos de Productos
- Comandos de Productos
- Módulos de la Aplicación
- Arquitectura Hexagonal
- Cliente API del Frontend
- Configuración de Tauri
- Dependencias de Desarrollo
- Dominio de Conciliación
- Dominio de Pagos MP
- Configuración TypeScript
- Tipos Compartidos del Frontend
- Scripts npm
- Skill de Aprendizaje Activo
- Carrito del Frontend
- Escáner USB HID
- Sistema Logging y Backups
- Tipos de SvelteKit
- Migraciones y Base de Datos
- Autostart en Rust
- Listener del Escáner
- Configuración de Svelte
- Plugins Tauri npm
- Iconos de la App
- Logging en Rust
- Layout SSR
- Tipo AppConfig
- Tipo Future
- Tipo Output
- Tipo Arc
- Tipo AppHandle
- Trait From
- Tipo Categoria
- Tipo Venta
- Tipo Connection
- Tipo Sync
- Tipo Venta (dup)
- Favicon de la App
- PagoMp
- Tipo PathBuf
- Paquete kiosco-app

## God Nodes (most connected - your core abstractions)
1. `PagoMp` - 28 edges
2. `ejecutarMock()` - 21 edges
3. `Query: analiza el estado actual del proyecto y determina que falta por hacer` - 20 edges
4. `ejecutar()` - 19 edges
5. `VentaSqlite` - 18 edges
6. `service()` - 17 edges
7. `ProductoSqlite` - 16 edges
8. `MockProductoRepo` - 16 edges
9. `MockVentas` - 15 edges
10. `Venta` - 14 edges

## Surprising Connections (you probably didn't know these)
- `infrastructure/api — comandos Tauri (producto/venta/caja/config)` --conceptually_related_to--> `Comando Rust sync_mercadopago (import + matching)`  [INFERRED]
  IMPLEMENTATION.md → SPEC.md
- `src/routes/* — páginas de módulos (placeholders sin tests)` --conceptually_related_to--> `Módulo 3 — Conciliación Mercado Pago`  [INFERRED]
  IMPLEMENTATION.md → SPEC.md
- `Rationale: polling /v1/payments/search (sin servidor para webhooks)` --rationale_for--> `Sync con API Mercado Pago por polling a /v1/payments/search`  [EXTRACTED]
  README.md → SPEC.md
- `app.html — shell HTML de SvelteKit` --conceptually_related_to--> `src/routes/* — páginas de módulos (placeholders sin tests)`  [INFERRED]
  KiosquitodoApp/src/app.html → IMPLEMENTATION.md
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

## Communities (76 total, 16 thin omitted)

### Community 0 - "Conciliación Mercado Pago"
Cohesion: 0.07
Nodes (43): Future, ConciliacionService, ConciliacionService<M, P, V>, delta_dia_ignora_ventas_en_efectivo(), delta_dia_positivo_cuando_api_muestra_mas(), MockMp, MockPagos, MockVentas (+35 more)

### Community 1 - "Cierre de Caja"
Cohesion: 0.07
Nodes (49): C, cerrar_caja_persiste_resumen_y_vacua(), CierreService, CierreService<C>, dentro_de_rango(), EstadoMock, exportar_csv_filtra_por_rango(), exportar_csv_incluye_encabezado_y_datos() (+41 more)

### Community 2 - "Dominio Ventas y Pagos"
Cohesion: 0.06
Nodes (36): calcular_margen(), calcular_total(), DevolucionRegistrada, item(), MedioPago, NuevaVenta, Option, Result (+28 more)

### Community 3 - "Dominio Producto y Barcode"
Cohesion: 0.06
Nodes (41): Display, Formatter, Into, Categoria, config_default(), estado_stock(), estado_stock_valores(), EstadoStock (+33 more)

### Community 4 - "Venta Service"
Cohesion: 0.11
Nodes (29): Arc, Arc<MockProductos>, item(), MockProductos, MockVentas, registrar_devolucion_rechaza_venta_inexistente(), registrar_venta_efectivo_valida_y_persiste(), registrar_venta_mp_y_mixto() (+21 more)

### Community 5 - "Producto Service"
Cohesion: 0.14
Nodes (25): actualizar_aplica_cambios(), actualizar_rechaza_reglas_de_negocio(), ajustar_stock_valida_y_aplica(), buscar_por_barcode_devuelve_producto(), buscar_por_id_devuelve_none_si_inexistente(), crear_rechaza_barcode_duplicado(), crear_rechaza_datos_invalidos(), crear_valida_y_persiste() (+17 more)

### Community 6 - "Repositorio SQLite de Ventas"
Cohesion: 0.14
Nodes (28): Connection, crear_producto(), crear_venta_con_stock_insuficiente_rollback(), crear_venta_descuenta_stock_en_transaccion(), devolucion_recupera_stock(), devolucion_registra_movimiento(), fila_a_venta(), listar_ventas_dia() (+20 more)

### Community 7 - "Mocks del Frontend"
Cohesion: 0.13
Nodes (30): crearCategoria(), crearCierre(), crearProducto(), crearTienda(), ejecutarMock(), horaLocalIso(), mockActualizarProducto(), mockAjustarStock() (+22 more)

### Community 8 - "AppConfig y Directorios"
Cohesion: 0.09
Nodes (31): AppHandle, Error, From, AppConfig, BACKUP_RETENCION_DIAS, carpeta_datos_proyecto(), dirs_home(), home_dir_resuelto() (+23 more)

### Community 9 - "Repositorio SQLite de Productos"
Cohesion: 0.15
Nodes (23): ajustar_stock_registra_movimiento(), barcode_unico_rechaza_duplicado(), buscar_barcode_inexistente_devuelve_none(), buscar_por_id(), COLUMNAS_PRODUCTO, costos_de_productos_devuelve_pares(), crear_y_buscar_por_barcode(), eliminar_baja_logica() (+15 more)

### Community 10 - "Repositorio SQLite de Pagos"
Cohesion: 0.17
Nodes (24): crear_producto_para_venta(), crear_venta_mp(), fila_a_pago(), matching_automatico_une_por_monto_y_tiempo(), matching_no_vincula_dos_pagos_a_la_misma_venta(), matching_no_vincula_monto_muy_distinto(), matching_normaliza_offset_de_la_api(), obtener_por_mp_id() (+16 more)

### Community 11 - "Credenciales en Keyring"
Cohesion: 0.09
Nodes (19): Entry, CUENTA, KeyringCredenciales, AppResult, Option, Self, SERVICIO, CredencialRepo (+11 more)

### Community 12 - "Estado Global y Toasts"
Cohesion: 0.09
Nodes (9): appState, cerrar(), Notificacion, notificar(), TipoNotificacion, RUTA, TipoSonido, Modulo (+1 more)

### Community 13 - "Repositorio SQLite de Cierres"
Cohesion: 0.16
Nodes (21): CierrePersistido, CierreRepo, CierreSqlite, crear_producto(), delta_mp_positivo_en_resumen(), fila_a_cierre(), guardar_y_leer_cierres(), resumen_dia_con_ventas_y_pagos_mp() (+13 more)

### Community 14 - "Conexión SQLite y PRAGMAs"
Cohesion: 0.14
Nodes (17): abrir_en_archivo_aplica_wal(), abrir_en_memoria_migra(), checkpoint_wal_no_falla_en_memoria(), DbConn, Arc, Connection, Mutex, Path (+9 more)

### Community 15 - "Deudas Técnicas del Backend"
Cohesion: 0.13
Nodes (25): config_cmd.rs (P16, no compila — 4 errores), config_repo.rs (pendiente), Backend P1-P14 implementado, Content-Security-Policy null, Query: analiza el estado actual del proyecto y determina que falta por hacer, existe_pago: código muerto, Frontend sin tests, map_err pendiente x3 (+17 more)

### Community 16 - "Flujo del Escáner de Barcode"
Cohesion: 0.15
Nodes (21): Campo de texto normal (input manual), Escáner USB (HID · emula teclado), Handler Rust (SELECT barcode), Lógica de discriminación (JS · buffer + timer 80–100 ms), Decisión de diseño: si llega Enter → es escaneo; si no → es teclado manual (buffer + timer 80–100 ms), SQLite · responde en <1 ms, Tauri IPC · invoke buscar_producto, Tauri IPC · invoke("buscar_producto") · barcode string → comando Rust (+13 more)

### Community 17 - "Convenciones y Estrategia MP"
Cohesion: 0.14
Nodes (21): Convención: montos en centavos (i64 / INTEGER / number), Convención de nombrado (camelCase / snake_case DTOs / español), infrastructure/mercadopago — cliente reqwest (diferido), Precisión monetaria: montos en centavos (i64 / INTEGER), Comando Rust sync_mercadopago (import + matching), Estados de conciliación: OK / SIN COINCIDENCIA EN MP / MONTO DIFIERE, Estrategias de conciliación (A manual asistida, B matching auto, C QR), Sync MP idempotente (INSERT OR IGNORE sobre mp_id, approved+accredited) (+13 more)

### Community 18 - "Metadatos del Frontend"
Cohesion: 0.10
Nodes (20): description, name, type, version, autoprefixer, jsdom, postcss, svelte (+12 more)

### Community 19 - "Puertos de Productos"
Cohesion: 0.21
Nodes (10): CategoriaRepo, ProductoRepo, AppResult, Categoria, NuevoProducto, Option, Producto, Send (+2 more)

### Community 20 - "Comandos de Productos"
Cohesion: 0.32
Nodes (19): Categoria, actualizar_producto(), ajustar_stock(), buscar_producto_por_barcode(), CategoriaNueva, crear_categoria(), crear_producto(), eliminar_producto() (+11 more)

### Community 21 - "Módulos de la Aplicación"
Cohesion: 0.16
Nodes (19): infrastructure/api — comandos Tauri (producto/venta/caja/config), src/routes/* — páginas de módulos (placeholders sin tests), El problema y la solución (inventario en papel → app local), Ajuste manual de stock (recuento, rotura, vencimiento), Atajos de teclado (F1–F6, Escape, Enter, Ctrl+E/M), Comando Rust buscar_producto (Tauri invoke), Módulo 4 — Cierre de Caja, Módulo 6 — Devoluciones / Notas de Crédito (+11 more)

### Community 22 - "Arquitectura Hexagonal"
Cohesion: 0.15
Nodes (19): config/ — rutas data/backups/logs y defaults, domain/ — Producto, Venta, Pago, Conciliacion, Cierre, errors/ — AppError y AppResult<T>, app.html — shell HTML de SvelteKit, Decisiones clave del stack (Tauri, Rust, Svelte, SQLite, HID, polling MP), Rationale: polling /v1/payments/search (sin servidor para webhooks), Rationale: Tauri sin Chromium → 50–100 MB RAM, Arquitectura hexagonal: domain → ports → application → infrastructure (+11 more)

### Community 23 - "Cliente API del Frontend"
Cohesion: 0.20
Nodes (18): actualizarProducto(), ajustarStock(), buscarProductoPorBarcode(), cerrarCaja(), crearCategoria(), crearProducto(), ejecutar(), eliminarProducto() (+10 more)

### Community 24 - "Configuración de Tauri"
Cohesion: 0.11
Nodes (18): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+10 more)

### Community 25 - "Dependencias de Desarrollo"
Cohesion: 0.12
Nodes (17): devDependencies, autoprefixer, jsdom, postcss, svelte, svelte-check, @sveltejs/adapter-static, @sveltejs/kit (+9 more)

### Community 26 - "Dominio de Conciliación"
Cohesion: 0.13
Nodes (5): Candidato, es_candidato_valido(), estado_conciliacion(), EstadoConciliacion, Option

### Community 27 - "Dominio de Pagos MP"
Cohesion: 0.22
Nodes (11): a_centavos(), descripcion_y_fecha_opcionales(), desde_api(), desde_api_elegible(), es_elegible(), MpPagoApi, normalizar_convierte_offset_a_local_naive(), normalizar_fecha_local() (+3 more)

### Community 28 - "Configuración TypeScript"
Cohesion: 0.15
Nodes (12): compilerOptions, allowJs, checkJs, esModuleInterop, forceConsistentCasingInFileNames, moduleResolution, resolveJsonModule, skipLibCheck (+4 more)

### Community 29 - "Tipos Compartidos del Frontend"
Cohesion: 0.17
Nodes (11): Categoria, CategoriaNueva, CierrePersistido, DevolucionRegistrada, ItemVenta, MedioPago, MpPago, NuevaVenta (+3 more)

### Community 30 - "Scripts npm"
Cohesion: 0.18
Nodes (11): scripts, build, check, check:watch, dev, preview, tauri, tauri:build (+3 more)

### Community 31 - "Skill de Aprendizaje Activo"
Cohesion: 0.46
Nodes (8): SKILL.md — Aprendizaje activo con IA (skill de opencode), Calibración según nivel del usuario (junior vs experto), Diagnóstico rápido: modo 'nuevo' vs modo 'rutina', Modo 'es nuevo para mí', Modo 'esto es rutina', Proteger el esfuerzo de generar (aprender haciendo), Revisión crítica, no aceptación pasiva, Tutor socrático, no oráculo

### Community 32 - "Carrito del Frontend"
Cohesion: 0.36
Nodes (6): carrito, crearCarrito(), ItemCarrito, producto(), Producto, vitest

### Community 33 - "Escáner USB HID"
Cohesion: 0.38
Nodes (7): src/lib/utils/scanner.ts — discriminación escáner vs teclado, Escáneres compatibles (Prosoft S2100, Titanika, Nictom — USB HID), Rationale: escáner HID nativo + timer JS (sin driver), crearListenerEscaner (buffer temporizado 100 ms, scanner.ts), Escáner USB HID (emulación de teclado, EAN-13), Integración de hardware: escáner USB 1D y API MP (sección 03), Rationale: buffer temporizado para discriminar escáner vs tipeo por velocidad

### Community 34 - "Sistema Logging y Backups"
Cohesion: 0.33
Nodes (7): infrastructure/system — logging, backup, keyring, autostart, Backup automático diario con wal_checkpoint(TRUNCATE), Sistema de logging rotativo (log + flexi_logger), Módulo 5 — Configuración, Rationale: PRAGMA wal_checkpoint antes de copiar el .db, Rationale: restore exige reinicio (SQLite no permite reemplazo con conexión abierta), Restore de backups (copia + reinicio de app)

### Community 35 - "Tipos de SvelteKit"
Cohesion: 0.29
Nodes (6): App, Error, Locals, PageData, PageState, Platform

### Community 36 - "Migraciones y Base de Datos"
Cohesion: 0.40
Nodes (6): infrastructure/db — conexión, PRAGMAs, migrations.rs, repos_sqlite, Migraciones versionadas de BD (v10), ports/ — traits producto_repo, venta_repo, pago_repo, cierre_repo, mp_cliente, Migraciones de esquema versionadas (MIGRATIONS v1–v10), SQLite con WAL, foreign_keys ON, busy_timeout y migraciones versionadas, Tabla schema_version (migraciones aplicadas)

### Community 37 - "Autostart en Rust"
Cohesion: 0.60
Nodes (5): autostart_habilitado(), deshabilitar_autostart(), habilitar_autostart(), AppHandle, AppResult

### Community 38 - "Listener del Escáner"
Cohesion: 0.40
Nodes (3): CallbackEscaneo, crearListenerEscaneo(), TECLAS_CONTROL

### Community 39 - "Configuración de Svelte"
Cohesion: 0.40
Nodes (3): config, @sveltejs/adapter-static, @sveltejs/vite-plugin-svelte

### Community 40 - "Plugins Tauri npm"
Cohesion: 0.50
Nodes (4): dependencies, @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs

### Community 41 - "Iconos de la App"
Cohesion: 0.67
Nodes (4): Tauri App Icon 128x128, Tauri App Icon 128x128@2x (HiDPI), Tauri App Icon 32x32, Frontend Favicon

### Community 42 - "Logging en Rust"
Cohesion: 0.83
Nodes (3): init_logging(), AppResult, Path

## Ambiguous Edges - Review These
- `Sistema de archivos` → `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)`  [AMBIGUOUS]
  diagramas/stack_tecnico_kiosco.svg · relation: conceptually_related_to
- `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` → `SQLite (vía rusqlite)`  [AMBIGUOUS]
  diagramas/stack_tecnico_kiosco.svg · relation: conceptually_related_to
- `Tauri App Icon 128x128` → `Frontend Favicon`  [AMBIGUOUS]
  KiosquitodoApp/src-tauri/icons/128x128.png · relation: conceptually_related_to

## Knowledge Gaps
- **128 isolated node(s):** `Error`, `Locals`, `PageData`, `PageState`, `Platform` (+123 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 311 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Sistema de archivos` and `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Hardware (Celeron N4020 · 4 GB RAM · HDD 119 GB · Windows 11)` and `SQLite (vía rusqlite)`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Tauri App Icon 128x128` and `Frontend Favicon`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `String` connect `Dominio Producto y Barcode` to `Cierre de Caja`, `Credenciales en Keyring`, `Conexión SQLite y PRAGMAs`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Why does `PagoSqlite` connect `Repositorio SQLite de Pagos` to `Conciliación Mercado Pago`, `Repositorio SQLite de Cierres`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Why does `PagoMp` connect `Conciliación Mercado Pago` to `Credenciales en Keyring`, `Repositorio SQLite de Pagos`, `Dominio de Pagos MP`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **What connects `Error`, `Locals`, `PageData` to the rest of the system?**
  _128 weakly-connected nodes found - possible documentation gaps or missing edges._