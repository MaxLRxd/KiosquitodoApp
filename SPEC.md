# SPEC.md — Especificación del sistema (punto de entrada)

## Rol de este documento

El SPEC define **qué** se construye y **cómo debe comportarse** el sistema. Este archivo es la **fuente de verdad única**: consolida las reglas irrompibles que todo código debe cumplir y el detalle completo por dominio. Las decisiones aquí documentadas se aprueban explícitamente y **no se cambian sin actualizar este documento primero**.

## Estructura del documento

| Sección | Contenido |
|---|---|
| [§1 Requerimientos](#sistema-de-gestion-de-kiosco-requerimientos-mvp) | Requerimientos funcionales y no funcionales, casos de uso, restricciones de infraestructura, convenciones y fuera del alcance |
| [§2 Stack y arquitectura](#sistema-de-gestion-de-kiosco-stack-tecnologico-y-arquitectura) | Decisión arquitectónica, stack completo, alternativas descartadas, esquema de base de datos, estructura del repo, IPC, backup/restore, logging, migraciones, testing |
| [§3 Hardware y Mercado Pago](#sistema-de-gestion-de-kiosco-integracion-de-hardware) | Escáner HID (discriminación vs teclado), API de MP, sync, conciliación y estrategias, atajos de teclado, conectividad |

## Reglas irrompibles

| # | Regla | Especifica |
|---|---|---|
| R1 | Todos los montos se modelan en **centavos** (`i64`/`INTEGER`); la conversión a decimales ocurre solo en la presentación | 02 |
| R2 | Los DTOs y contratos IPC usan `snake_case`; las tablas de BD y entidades de dominio usan español | 02 |
| R3 | Backend con arquitectura hexagonal: dependencias unidireccionales `domain → ports → application → infrastructure` | 02 |
| R4 | SQLite con WAL, `foreign_keys` ON, `busy_timeout` y migraciones versionadas mediante `schema_version` | 02 |
| R5 | El sync de MP es idempotente: `INSERT OR IGNORE` sobre `mp_id` único; se importan solo pagos `approved` + `accredited` | 03 |
| R6 | El monto de MP se convierte a la entrada: `(transaction_amount * 100.0).round() as i64` | 03 |
| R7 | Matching automático: monto ±100 centavos dentro de una ventana de ±10 minutos | 03 |
| R8 | Conciliación: diferencia > 1 centavo = "monto difiere" | 03 |
| R9 | Registrar venta con `medio_pago = mercado_pago`/`mixto` descuenta stock y guarda items en la misma transacción | 02 |
| R10 | El Access Token de MP se guarda cifrado en el credential store del SO (`keyring`) | 02/03 |
| R11 | El escáner opera por HID (emulación de teclado); discriminación escáner vs tipeo por velocidad (<50 ms por código) | 03 |
| R12 | App de escritorio 100% local (Tauri), sin servidor: integración MP por polling a `/v1/payments/search` | 02 |

## Stack fijo

Tauri v2 (Rust) · Svelte 5 + SvelteKit 2 · Vite 8 · Tailwind 3.4 · SQLite (`rusqlite` 0.32 bundled) · `reqwest` 0.12 · `tokio` 1 · `keyring` 2 · `flexi_logger` · detalles en `02`.

## Cómo leer los cambios

- Este documento precede a su implementación (metodología SPEC-driven).
- `IMPLEMENTATION.md` registra cómo el código cumple estas reglas.
- Para cambiar una regla o el detalle: actualizar `SPEC.md` antes de tocar código.

## Detalle del SPEC


---

## Sistema de Gestión de Kiosco — Requerimientos (MVP)

> **Versión:** 1.1  
> **Alcance:** MVP orientado a resolver el descontrol de transferencias Mercado Pago en días de alto tráfico, con gestión de inventario y ventas integrada.

---

## Contexto del negocio

El cliente opera un kiosco y actualmente lleva el inventario y las cuentas en papel. El principal punto de dolor es el registro manual de transferencias por Mercado Pago en días de alto tráfico, lo que genera descontrol y descuadres en la caja. La visión a futuro incluye la integración de un lector de código de barras USB 1D para agilizar el ingreso de productos, cobros y consultas de precio.

---

## Restricciones de infraestructura

| Ítem | Detalle |
|---|---|
| Hosting / servidores | $0 — sistema 100% local |
| Sistema operativo | Windows 11 |
| Procesador | Intel Celeron N4020 @ 1.10 GHz |
| RAM | 4 GB DDR4 2400 MT/s |
| Gráfica | Intel UHD Graphics 600 (512 MB integrada) |
| Almacenamiento | HDD 119 GB |
| Conexión a internet | Requerida solo para sincronizar con la API de Mercado Pago |

---

## Requerimientos funcionales

### Módulo 1 — Punto de Venta (POS)

- Registrar una venta ítem a ítem, ya sea por búsqueda de nombre o por escaneo de código de barras.
- Soportar transacciones mixtas: efectivo + Mercado Pago en una misma venta.
- Al cerrar una venta con Mercado Pago, registrar obligatoriamente el monto, la descripción opcional y la marca de tiempo. Este registro es el reemplazo directo del papel.
- Emitir un ticket de resumen visible en pantalla. Impresión opcional (fuera del alcance del MVP).
- Modo "consulta de precio" activo en cualquier pantalla: el escaneo de un producto que no esté en una venta abierta muestra un popup con nombre y precio, y se cierra solo.
- **Confirmación de acciones destructivas:** antes de descartar el carrito actual, anular una venta o cerrar la caja, debe mostrarse un diálogo de confirmación para evitar pérdidas accidentales de datos.
- **Atajos de teclado (shortcuts):** la operación principal debe poder realizarse sin mouse. Shortcuts mínimos: `F1` → POS, `F2` → Inventario, `F3` → Conciliación, `F4` → Cierre de Caja, `F5` → Configuración, `Escape` → cancelar/volver, `Enter` → confirmar/cobrar.
- **Navegación entre pantallas:** debe existir un menú lateral o barra de pestañas siempre visible que permita cambiar entre módulos (POS, Inventario, Conciliación, Cierre, Configuración) en 1 clic o 1 tecla.
- **Indicador de conectividad:** debe mostrarse permanentemente un badge o icono que indique si hay conexión a internet (verde = conectado, rojo = sin conexión, amarillo = sync en progreso).

### Módulo 2 — Inventario

- Alta, baja y modificación de productos con los siguientes campos: nombre, código de barras (EAN-13 o similar), precio de venta, precio de costo, stock actual, stock mínimo de alerta.
- **Categorías de productos:** cada producto debe pertenecer a una categoría (Bebidas, Alfajores, Snacks, Lácteos, etc.) para facilitar la búsqueda y futuros reportes. Las categorías son editables por el operador. Tabla `categorias` con `id`, `nombre`, `color` (opcional para identificación visual).
- Descuento automático de stock al confirmar cada venta.
- Alerta visual en la pantalla principal cuando un producto cae por debajo de su stock mínimo configurado.
- Búsqueda de productos por nombre parcial, código de barras o categoría.
- **Paginación en listado:** cuando el inventario supere los 100 productos, el backend debe aceptar `LIMIT` y `OFFSET`, y el frontend debe mostrar los resultados paginados (20-50 por página) para evitar lentitud en HDD y sobrecarga del DOM.
- **Ajuste manual de stock:** debe existir una función para corregir el stock de un producto sin pasar por una venta o devolución (ej: rotura, vencimiento, error de carga inicial). Cada ajuste se registra en una tabla `movimientos_stock` con producto_id, cantidad anterior, cantidad nueva, motivo y operador.
- Importación de catálogo inicial desde CSV (fuera del alcance del MVP, segunda iteración).

### Módulo 3 — Conciliación Mercado Pago

- Sincronización con la API de Mercado Pago para traer todos los pagos aprobados y acreditados del período seleccionado (ver documento de integración técnica).
- Pantalla de conciliación con dos columnas: "Ventas MP registradas por el operador" vs "Pagos recibidos según API de MP".
- Matching automático sugerido: si existe exactamente un pago de MP con el mismo monto ± $1 dentro de una ventana de ±10 minutos del registro manual, se sugiere la vinculación. El operador la confirma con un clic.
- Vinculación manual para los casos donde el matching automático no es concluyente.
- Indicador visual de estado por cada registro: `OK`, `SIN COINCIDENCIA EN MP`, `MONTO DIFIERE`.
- Exportación del resumen del día a CSV para cotejar contra el historial de la app de Mercado Pago.

### Módulo 4 — Cierre de Caja

- Resumen del día con los siguientes totales: ventas en efectivo, ventas en Mercado Pago, cantidad de transacciones, margen bruto estimado (precio de venta - precio de costo).
- Delta de conciliación MP: diferencia entre lo registrado y lo acreditado según la API.
- Historial de cierres anteriores, navegable por fecha.
- El cierre no bloquea el sistema; puede ejecutarse en cualquier momento del día.

### Sistema de notificaciones (transversal)

- La aplicación debe contar con un sistema de notificaciones toast/snackbar no intrusivo para comunicar al operador eventos como: producto no encontrado, sync completado, error de conexión, backup realizado, stock bajo.
- Las notificaciones deben auto-cerrarse después de 3-4 segundos, excepto las de error crítico que requieren acción del operador.
- El sistema de notificaciones debe ser un componente singleton accesible desde cualquier pantalla.
- **Feedback sonoro:** además de las notificaciones visuales, la app debe emitir sonidos cortos (beeps vía HTML5 Audio API o archivos `.wav` empaquetados) para: escaneo exitoso (beep agudo), producto no encontrado (doble beep grave), venta completada (melodía corta), error del sistema (sonido de error). Desactivables desde Configuración.

### Módulo 5 — Configuración

- Ingreso y almacenamiento seguro del Access Token de Mercado Pago (cifrado en el keyring de Windows).
- Configuración de la carpeta de backups automáticos.
- PIN de acceso al sistema (4 a 6 dígitos).
- Bloqueo automático por inactividad: después de N minutos sin interacción (teclado, mouse, escáner), la app se bloquea y requiere el PIN para reanudar. Valor por defecto: 5 minutos. Configurable (0 = desactivado).
- Intervalo de sincronización automática con MP (por defecto: 15 minutos, configurable).

### Módulo 6 — Devoluciones / Notas de Crédito

- Permitir la devolución total o parcial de una venta existente, seleccionando la venta original y los ítems a devolver.
- Al confirmar una devolución: el stock de los productos devueltos se incrementa automáticamente, y se genera un registro de devolución con fecha, motivo opcional y operador.
- Tabla `devoluciones` con: `id`, `venta_id` (referencia a la venta original), `fecha`, `motivo`, `items` (JSON o tabla separada con producto_id, cantidad, monto_devuelto).
- Las devoluciones se descuentan del total del día en el cierre de caja (efectivo o MP según el medio de pago original).
- No se requiere integración con Mercado Pago para reversos; la devolución es un registro interno que el operador compensa fuera del sistema.

---

## Requerimientos no funcionales

### Rendimiento

- Tiempo de inicio de la aplicación: menos de 5 segundos en el hardware del cliente.
- Respuesta a un escaneo de código de barras (desde el beep hasta la actualización de UI): menos de 300 ms.
- La sincronización con la API de MP no debe bloquear ni congelar la interfaz (debe ejecutarse en segundo plano).

### Disponibilidad y resiliencia

- El sistema debe funcionar al 100% en modo offline para todas las operaciones excepto la sincronización con MP.
- Un corte de luz o cierre inesperado no debe producir pérdida de datos (SQLite con journaling WAL habilitado).
- Backup automático del archivo `.db` a la carpeta configurada, una vez por día al iniciar la aplicación.
- **Restore de backups:** debe existir una función en Configuración para restaurar la base de datos desde un archivo `.db` de backup seleccionado por el operador. Antes de restaurar, se hace un backup del estado actual.
- **Arranque con Windows:** la aplicación debe poder configurarse para iniciarse automáticamente al encender la PC (mediante acceso directo en `shell:startup` o vía registro de Windows).

### Footprint de recursos

- Uso máximo de RAM en operación activa: 150 MB (deja ~1.85 GB libres para Windows 11 y otros procesos).
- Tamaño del instalador: menos de 25 MB.
- Tamaño de la base de datos después de 1 año de operación estimada: menos de 100 MB.

### Usabilidad

- La operación principal (registrar una venta con MP) debe completarse en 3 pasos o menos.
- La interfaz debe ser operable con una sola mano mientras la otra sostiene o dirige el escáner.
- Sin asistente de instalación complejo: ejecutable único o instalador de un clic.

### Mantenibilidad y calidad

- **Sistema de logging:** la aplicación debe registrar eventos clave en un archivo de log rotativo (diario): inicio/cierre de app, ventas registradas, syncs con MP, errores de conexión, backups realizados y cierres de caja. Los logs deben estar en `{carpeta_usuario}/KiosquitodoApp/logs/` con nombre `kiosco_YYYY-MM-DD.log`. Niveles: `INFO`, `WARN`, `ERROR`.
- **Testing:** el backend Rust debe tener tests unitarios para los comandos críticos (buscar producto, crear venta, sync MP) usando una BD SQLite en memoria. El frontend Svelte debe tener tests de componentes para POS e Inventario. Los tests deben ejecutarse antes de cada build.
- **Auto-update:** la aplicación debe soportar actualizaciones automáticas usando el plugin `tauri-plugin-updater`. Al iniciar, debe verificar si hay una nueva versión y, si la hay, ofrecer descargarla e instalarla en segundo plano. Las actualizaciones se distribuyen desde un archivo JSON en un hosting estático (GitHub Releases o similar).

### Seguridad

- El Access Token de Mercado Pago se almacena cifrado usando el keyring del sistema operativo, nunca en texto plano en el archivo `.db` ni en archivos de configuración.
- PIN de acceso local configurable. No se requiere autenticación de red ni cuentas de usuario.

---

## Casos de uso principales (MVP)

```
Operador → [Escanea producto] → Sistema busca en inventario → Agrega al carrito
Operador → [Confirma venta en MP] → Sistema registra monto + timestamp
Sistema → [Cada 15 min, si hay internet] → Consulta API de MP → Guarda pagos en DB
Operador → [Abre conciliación] → Ve ventas vs pagos → Vincula manualmente los no matcheados
Operador → [Cierre de caja] → Ve resumen del día + delta de MP → Exporta CSV si necesita
```

---

## Convenciones de implementación (aprobadas)

| Convención | Detalle |
|---|---|
| Montos en centavos | Todas las columnas monetarias y DTOs usan `INTEGER` / `i64` (centavos, ej: $1.99 = 199). La conversión a decimales se hace exclusivamente en la capa de presentación. |
| Nombrado | Capas y módulos técnicos en inglés (`application`, `ports`, `infrastructure`); entidades de dominio y tablas de BD en español (snake_case). |
| Backend | Arquitectura por capas en `src-tauri/`: `domain → ports → application → infrastructure`. |
| Frontend | SvelteKit en `KiosquitodoApp/` (subcarpeta del repo), `adapter-static` en modo SPA (`ssr=false`, fallback `index.html`). |

---

## Fuera del alcance del MVP

- Impresión de tickets físicos (térmica u otra).
- Gestión de múltiples usuarios con roles diferenciados.
- Reportes estadísticos avanzados (curvas de venta, top productos, etc.).
- Importación de catálogo desde CSV.
- Integración con QR de Mercado Pago para cobro generado desde el sistema.
- App móvil o acceso remoto.
- Soporte multi-sucursal.
- Venta a crédito / cuenta corriente.
- Integración con balanza electrónica (productos a granel).
- Módulo de proveedores y órdenes de compra.
- Facturación electrónica / soporte de impuestos (IVA, tasas municipales). El MVP no emite comprobantes fiscales ni calcula impuestos; el operador debe usar el sistema de facturación existente en paralelo. Si en el futuro se requiere, se recomienda agregar campos `tasa_iva`, `neto_gravado` e `iva_total` en las tablas de productos y ventas.

---

## Sistema de Gestión de Kiosco — Stack Tecnológico y Arquitectura

> **Versión:** 1.1  
> **Premisa:** Sistema de escritorio 100% local, sin servidor, sin costos de hosting. Optimizado para hardware de gama baja (Celeron N4020, 4 GB RAM, HDD).

---

## Decisión arquitectónica central

El sistema se construye como una **aplicación de escritorio nativa para Windows 11** usando **Tauri v2**, con lógica de negocio en **Rust** y frontend en **Svelte + Tailwind CSS**, con **SQLite** como base de datos embebida. Esta combinación es la única viable que cumple simultáneamente las restricciones de memoria, rendimiento y costo operativo.

---

## Stack completo

| Capa | Tecnología | Versión |
|---|---|---|
| Framework de escritorio | Tauri | v2.11.x |
| Lenguaje de backend | Rust | stable (1.77+) |
| Framework de UI | Svelte + SvelteKit | v5.x / v2.x |
| Estilos | Tailwind CSS | v3.4.x |
| Base de datos | SQLite (vía `rusqlite`) | 0.32.x (`bundled`) |
| HTTP client (sync MP) | `reqwest` (async, `rustls-tls`) | 0.12.x |
| Runtime async | `tokio` | 1.x |
| Serialización JSON | `serde` + `serde_json` | 1.x |
| Cifrado de credenciales | `keyring` (Windows Credential Store) | 2.x |
| Bundler de frontend | Vite | 8.x |
| Logging | `log` + `flexi_logger` | 0.4.x / 0.29.x |
| Errores tipados | `thiserror` | 2.x |
| Auto-update | `tauri-plugin-updater` | 2.x |
| Autostart | `tauri-plugin-autostart` | 2.x |
| Testing (Rust) | `cargo test` + `rusqlite::Connection::open_in_memory` | — |
| Testing (Svelte) | `vitest` + `@testing-library/svelte` | 5.x |

---

## Análisis de alternativas descartadas

### ¿Por qué no Electron?

Electron embebe una instancia completa de Chromium en cada aplicación. En idle consume entre 150 y 300 MB de RAM. En un equipo con 4 GB donde Windows 11 ya ocupa ~2 GB, esto deja un margen de operación menor a 700 MB, lo que produce cuellos de botella severos en operación simultánea con antivirus y otros procesos del sistema.

Tauri resuelve esto usando el **motor WebView nativo del SO** (en Windows 11, WebView2 basado en Edge/Blink, que ya está instalado y compartido en memoria con el sistema). El resultado es una aplicación que consume entre 50 y 100 MB de RAM en total.

### ¿Por qué no Python/Flask o Django?

- El intérprete de Python más sus dependencias típicas agregan 100–200 MB de RAM.
- Distribuir una aplicación de escritorio Python en Windows sin instalar Python en la máquina del cliente requiere PyInstaller u alternativas, que generan ejecutables lentos de iniciar (3–8 segundos de boot).
- El ecosistema de UI de escritorio de Python (Tkinter, PyQt) es inferior a Svelte para crear interfaces modernas y mantenibles.

### ¿Por qué no Node.js?

Un proceso Node en idle consume entre 40 y 80 MB. Combinado con Electron sería el peor escenario. Combinado con otras soluciones como NW.js el problema es el mismo.

### ¿Por qué no .NET/WinForms o WPF?

Técnicamente viable, pero con desventajas importantes para este proyecto: la curva de desarrollo es mayor, el ecosistema de componentes de UI modernos es inferior, y el binario resultante tiene dependencias del runtime .NET que pueden no estar instaladas en la máquina del cliente.

---

## Justificación de cada componente

### Tauri v2

- Usa el WebView nativo del SO, sin empaquetar Chromium.
- El binario resultante pesa entre 3 y 10 MB en disco.
- Permite comunicación segura entre frontend (JS/Svelte) y backend (Rust) a través de IPC tipo `invoke("comando", args)`.
- Produce un instalador `.exe` estándar para Windows.
- Footprint en RAM: **50–100 MB total** incluyendo frontend y backend.

### Rust (backend de Tauri)

- Sin garbage collector, por lo tanto sin pausas de GC. En hardware lento, las pausas de GC en lenguajes como Go, Java o .NET se perciben como congelamientos de UI.
- Las operaciones críticas (buscar producto, registrar venta, actualizar stock) se ejecutan en hilos nativos sin overhead de runtime.
- El manejo explícito de errores con `Result<T, E>` elimina crashes inesperados por excepciones no manejadas.
- Footprint del proceso Rust en RAM: **20–40 MB**.

### Svelte v5 (frontend)

- A diferencia de React o Vue, Svelte **compila la UI a JavaScript puro en tiempo de build**. No hay Virtual DOM en memoria en tiempo de ejecución.
- Los bundles resultantes son de 30–80 KB, que el WebView renderiza casi instantáneamente.
- La reactividad de Svelte es nativa del lenguaje, sin `useState` ni `useEffect` externos.
- Footprint en RAM (WebView): **20–40 MB**.

### SQLite (base de datos)

- Es una librería enlazada estáticamente al binario de Rust: no hay servidor de base de datos corriendo, no hay proceso adicional en memoria.
- El archivo `.db` de este sistema no superará 50 MB en años de uso normal.
- Las queries de este sistema son de complejidad baja (sin joins complejos de más de 3 tablas), y SQLite las ejecuta en microsegundos incluso sobre HDD.
- Soporta journaling WAL (Write-Ahead Logging) para durabilidad ante cortes de energía.
- Footprint en RAM: **3–5 MB** (buffer de cache configurable).

> **✅ Precisión monetaria (decisión adoptada):** Todos los montos se modelan desde el día 1 como **centavos** (`INTEGER` / `i64`, ej: $1.99 = 199) para evitar errores de redondeo acumulativos de `f64` (IEEE 754). La conversión a decimales ocurre **exclusivamente** en la capa de presentación (`lib/utils/moneda.ts`). Los umbrales de conciliación se expresan en centavos: ±$1 = ±100, y una diferencia >1 centavo marca "monto difiere".

---

## Uso estimado de RAM en producción

| Componente | RAM estimada |
|---|---|
| Windows 11 (base) | ~2.000 MB |
| Antivirus / procesos del sistema | ~200 MB |
| Tauri backend (Rust) | ~30 MB |
| WebView2 + Svelte UI | ~40 MB |
| SQLite (en proceso Rust) | ~5 MB |
| **Total estimado** | **~2.275 MB** |
| **Margen libre sobre 4 GB** | **~1.725 MB** |

---

## Esquema de base de datos (SQLite)

```sql
-- ⚠️ TODOS los montos están expresados en CENTAVOS (INTEGER), nunca en decimales.

-- Productos del inventario
CREATE TABLE productos (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre      TEXT NOT NULL,
    barcode     TEXT UNIQUE,
    precio_venta  INTEGER NOT NULL,  -- centavos
    precio_costo  INTEGER,           -- centavos
    stock         INTEGER NOT NULL DEFAULT 0,
    stock_minimo  INTEGER NOT NULL DEFAULT 5,
    activo        INTEGER NOT NULL DEFAULT 1,
    creado_en     TEXT DEFAULT (datetime('now', 'localtime')),
    actualizado_en TEXT
);

CREATE INDEX idx_productos_barcode ON productos(barcode);
CREATE INDEX idx_productos_nombre  ON productos(nombre);

-- Categorías de productos
CREATE TABLE categorias (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre  TEXT NOT NULL UNIQUE,
    color   TEXT   -- opcional: código hex para identificación visual
);

ALTER TABLE productos ADD COLUMN categoria_id INTEGER REFERENCES categorias(id);
CREATE INDEX idx_productos_categoria ON productos(categoria_id);

-- Ventas (cabecera)
CREATE TABLE ventas (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    total         INTEGER NOT NULL,  -- centavos
    medio_pago    TEXT NOT NULL CHECK(medio_pago IN ('efectivo', 'mercado_pago', 'mixto')),
    monto_efectivo   INTEGER DEFAULT 0,  -- centavos
    monto_mp         INTEGER DEFAULT 0,  -- centavos
    descripcion_mp   TEXT,
    hora          TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    cerrada       INTEGER NOT NULL DEFAULT 0
);

-- Ítems de cada venta
CREATE TABLE venta_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id    INTEGER NOT NULL REFERENCES ventas(id),
    producto_id INTEGER NOT NULL REFERENCES productos(id),
    cantidad    INTEGER NOT NULL,
    precio_unitario INTEGER NOT NULL  -- centavos
);

-- Devoluciones / Notas de crédito
CREATE TABLE devoluciones (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id INTEGER NOT NULL REFERENCES ventas(id),
    fecha    TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    motivo   TEXT,
    creado_en TEXT DEFAULT (datetime('now', 'localtime'))
);

CREATE TABLE devolucion_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    devolucion_id   INTEGER NOT NULL REFERENCES devoluciones(id),
    producto_id     INTEGER NOT NULL REFERENCES productos(id),
    cantidad        INTEGER NOT NULL,
    monto_devuelto  INTEGER NOT NULL  -- centavos
);

-- Pagos importados desde la API de Mercado Pago
CREATE TABLE mp_pagos (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    mp_id            TEXT UNIQUE NOT NULL,
    fecha_aprobacion TEXT,
    monto            INTEGER NOT NULL,  -- centavos
    descripcion      TEXT,
    venta_id         INTEGER REFERENCES ventas(id),
    importado_en     TEXT DEFAULT (datetime('now', 'localtime'))
);

CREATE INDEX idx_mp_pagos_fecha ON mp_pagos(fecha_aprobacion);
CREATE INDEX idx_mp_pagos_monto ON mp_pagos(monto);

-- Configuración del sistema
CREATE TABLE configuracion (
    clave TEXT PRIMARY KEY,
    valor TEXT
);

-- Cierres de caja
CREATE TABLE cierres_caja (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha            TEXT NOT NULL,
    total_efectivo   INTEGER DEFAULT 0,  -- centavos
    total_mp         INTEGER DEFAULT 0,  -- centavos
    total_ventas     INTEGER DEFAULT 0,
    margen_bruto     INTEGER DEFAULT 0,  -- centavos
    delta_mp         INTEGER DEFAULT 0,  -- centavos
    creado_en        TEXT DEFAULT (datetime('now', 'localtime'))
);

-- Movimientos de stock (ajustes manuales, roturas, vencimientos)
CREATE TABLE movimientos_stock (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    producto_id  INTEGER NOT NULL REFERENCES productos(id),
    stock_anterior INTEGER NOT NULL,
    stock_nuevo  INTEGER NOT NULL,
    motivo       TEXT NOT NULL CHECK(motivo IN ('recuento_fisico', 'rotura', 'vencimiento', 'error_carga', 'otro')),
    operador     TEXT,
    creado_en    TEXT DEFAULT (datetime('now', 'localtime'))
);
```


## Arquitectura de carpetas del proyecto

Estructura del repo (aprobada): los documentos técnicos viven en la **raíz** del repositorio y todo el proyecto (frontend + backend) en la subcarpeta `KiosquitodoApp/`. Dentro de ella, el backend está en `src-tauri/` (layout estándar de Tauri, cero fricción con el CLI) y el frontend SvelteKit en su raíz. El backend sigue **arquitectura hexagonal / clean architecture**, adaptando las convenciones Spring Boot a nombres idiomáticos de Rust:

| Capa Spring Boot | Equivalente Rust | Responsabilidad |
|---|---|---|
| `controllers` | `infrastructure/api/` (comandos Tauri) | Capa de entrada IPC, delgada: valida y delega |
| `services` | `application/` (casos de uso) | Orquesta reglas de negocio vía puertos (traits) |
| `repositories` | `ports/` (traits) + `infrastructure/db/` | Contratos + adaptadores SQLite |
| `entities/models` | `domain/` | Entidades y lógica de negocio pura (sin I/O, sin SQL) |
| `config` / `errors` | `config/`, `errors/` | Configuración de rutas/defaults y errores tipados |

```
<repo>/                                # Raíz del repositorio
├── README.md                          # Documentación técnica (índice y visión general)
├── SPEC.md                            # SPEC único: reglas irrompibles R1–R12 + detalle §1–§3
├── IMPLEMENTATION.md                  # Cómo está implementado el código (guía viva)
├── diagramas/                         # Diagramas SVG/PNG de la documentación
└── KiosquitodoApp/                    # Proyecto completo (frontend + backend)
    ├── src/                           # Frontend SvelteKit
    │   ├── routes/                    #   +layout (sidebar + barra estado + atajos F1–F6) y páginas de módulos
    │   ├── lib/                       #   components/, stores/, api/, utils/, types/
    │   ├── app.html · app.css · app.d.ts
    ├── static/                        # Assets estáticos (favicon.png, sounds/*.wav)
    ├── package.json · svelte.config.js · vite.config.ts (puerto 1420, strictPort)
    ├── tailwind.config.js · postcss.config.js
    ├── src-tauri/                     # Backend Rust (Tauri v2)
    │   ├── Cargo.toml · build.rs · tauri.conf.json (devUrl 1420, frontendDist ../build)
    │   ├── capabilities/default.json  #   Permisos IPC (core:default)
    │   ├── icons/                     #   Iconos del instalador
    │   └── src/
    │       ├── main.rs                # Binario mínimo → llama a run()
    │       ├── lib.rs                 # Composición raíz: logging/backup/autostart + wiring de dependencias
    │       ├── config/                # Rutas (data, backups, logs) y valores por defecto
    │       ├── errors/                # AppError, AppResult<T> y conversiones From
    │       ├── domain/                # producto, venta, pago, conciliacion, cierre — reglas puras
    │       ├── ports/                 # traits: producto_repo, venta_repo, pago_repo, cierre_repo, mp_cliente, credencial_repo
    │       ├── application/           # producto_service, venta_service, conciliacion_service, cierre_service
    │       └── infrastructure/
    │           ├── api/               # Comandos Tauri (producto_cmd, venta_cmd, mercadopago_cmd, caja_cmd, config_cmd)
    │           ├── db/                # mod.rs (conexión + PRAGMAs WAL), migrations.rs, repos/*_sqlite.rs
    │           ├── mercadopago/       # Cliente HTTP real de MP (reqwest)
    │           └── system/            # logging, backup, keyring, autostart
```

> ⚠️ Nota de mantenimiento: un `cargo clean` (o borrar `src-tauri/target/`) es **obligatorio** tras mover la carpeta del proyecto de ubicación, porque Cargo incrusta rutas absolutas en su caché de build y puede fallar apuntando a la ruta anterior.

---

## Comunicación frontend ↔ backend (IPC de Tauri)

```typescript
// Desde el frontend SvelteKit: invocar un comando Rust.
// Nota: los argumentos se envían en snake_case (Tauri v2 los mapea automáticamente)
// y todos los montos van en CENTAVOS.
import { invoke } from '@tauri-apps/api/core';

// Buscar producto por código de barras
const producto = await invoke('buscar_producto', { barcode: '7790001234567' });

// Registrar venta
const ventaId = await invoke('crear_venta', {
  items: carrito,
  medio_pago: 'mercado_pago',
  monto_pago: 1500,        // = $15.00 (centavos)
  descripcion_mp: 'Compra kiosco'
});

// Sincronizar con Mercado Pago
const resultado = await invoke('sync_mercadopago');
// { total_encontrados: 12, nuevos_insertados: 3 }
```

```rust
// En el backend Rust: definición del comando
#[tauri::command]
fn buscar_producto(barcode: String, db: State<DbConn>) -> Result<Option<Producto>, String> {
    let conn = db.lock().unwrap();
    // SELECT * FROM productos WHERE barcode = ?1 AND activo = 1
    // ...
}
```

---

## Backup automático

Al iniciar la aplicación, Rust verifica si ya existe un backup del día. Si no existe, copia el archivo `.db` a la carpeta configurada con el nombre `kiosco_backup_YYYY-MM-DD.db`. Los backups más antiguos de 30 días se eliminan automáticamente para no saturar el disco.

> **⚠️ Importante — consistencia con WAL:** Con WAL habilitado, los cambios recientes pueden estar en el archivo `-wal` y no en el `.db` principal. Copiar solo el `.db` produce un backup incompleto. Por eso, **antes de copiar se ejecuta `PRAGMA wal_checkpoint(TRUNCATE);`** para forzar la escritura de todos los cambios pendientes al archivo principal.

```rust
// Ruta de backup: C:\Users\<usuario>\Documents\KioscoBackups\
// o la carpeta configurada por el usuario en Settings
fn hacer_backup_si_necesario(db_path: &Path, backup_dir: &Path, db: &Connection) -> Result<(), String> {
    let hoy = Local::now().format("%Y-%m-%d").to_string();
    let backup_path = backup_dir.join(format!("kiosco_backup_{}.db", hoy));
    if !backup_path.exists() {
        // Forzar escritura de todos los cambios pendientes del WAL al archivo principal
        db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| format!("Error al checkpointear WAL: {}", e))?;

        std::fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;
        std::fs::copy(db_path, &backup_path)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

---

## Restore de backups

Función accesible desde Configuración que permite al operador seleccionar un archivo `.db` de backup y restaurarlo como base de datos activa.

> **Restricción técnica:** SQLite no permite reemplazar el archivo `.db` mientras la conexión está abierta (en Windows falla con "archivo en uso"). Por lo tanto, el restore se implementa copiando el backup sobre el archivo original y **reiniciando la aplicación** automáticamente para que Tauri abra una nueva conexión.

```rust
use std::fs;
use std::path::Path;
use tauri::Manager;
use std::process::Command;

#[tauri::command]
async fn restaurar_backup(app: tauri::AppHandle, ruta_backup: String) -> Result<(), String> {
    let db_path = app.path().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("kiosco.db");

    let backup_path = Path::new(&ruta_backup);
    if !backup_path.exists() {
        return Err("El archivo de backup no existe".to_string());
    }

    // 1. Hacer backup del estado actual antes de restaurar
    let backup_previo = db_path.with_extension("db.pre_restore");
    fs::copy(&db_path, &backup_previo).map_err(|e| e.to_string())?;

    // 2. Copiar backup sobre el archivo actual (puede fallar si la BD está bloqueada)
    fs::copy(&backup_path, &db_path).map_err(|e| {
        format!("No se pudo reemplazar la BD (puede estar en uso): {}", e)
    })?;

    // 3. Reiniciar la aplicación para que abra una nueva conexión
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    Command::new(exe)
        .spawn()
        .map_err(|e| format!("Error al reiniciar: {}", e))?;
    app.exit(0);

    Ok(())
}
```

> **Alternativa más simple (recomendada MVP):** El restore se realiza fuera de la app: el operador cierra la aplicación, reemplaza manualmente el archivo `kiosco.db` con el backup, y vuelve a abrir la app. La función de restore en la UI solo abre el explorador de archivos y muestra las instrucciones.

---

## Navegación entre pantallas

La aplicación usa el enrutador nativo de **SvelteKit** con un layout fijo. El sidebar se define en `+layout.svelte` y las rutas en `src/routes/`. Cada módulo es una subruta con su propio `+page.svelte`.

| Icono | Label | Tecla | Ruta |
|-------|-------|-------|------|
| 🛒 | POS | `F1` | `/` |
| 📦 | Inventario | `F2` | `/inventario` |
| 🔄 | Conciliación | `F3` | `/conciliacion` |
| 💰 | Cierre de Caja | `F4` | `/cierre` |
| ⚙️ | Configuración | `F5` | `/configuracion` |
| ↩️ | Devoluciones | `F6` | `/devoluciones` |

La navegación se realiza con `goto()` de SvelteKit (`$app/navigation`). El sidebar y los atajos de teclado se implementan en `+layout.svelte` y son persistentes entre rutas.

### Layout base (`+layout.svelte`)
```
┌──────────┬─────────────────────────────────────┐
│  Menú    │                                     │
│  lateral │      <slot /> (ruta activa)         │
│  200px   │                                     │
│          │                                     │
│ [POS]    │                                     │
│ [Inv.]   │                                     │
│ [Conc.]  │                                     │
│ [Cierre] │                                     │
│ [Config] │                                     │
├──────────┴─────────────────────────────────────┤
│   Barra inferior: conectividad │ sync │ reloj  │
└────────────────────────────────────────────────┘
```

---

## Sistema de logging

La aplicación registra eventos en archivos de log rotativos usando los crates `log` + `flexi_logger`.

### Configuración en `main.rs`
```rust
use flexi_logger::{Logger, WriteMode};
use std::env;

fn init_logging() {
    let log_dir = format!("{}/KiosquitodoApp/logs",
        env::var("USERPROFILE").unwrap_or_else(|_| "C:/Users/default".to_string()));

    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default()
            .directory(&log_dir)
            .basename("kiosco")
            .suffix("log")
            .suppress_timestamp())
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(30),
        )
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .unwrap();
}
```

### Eventos logueados
| Nivel | Evento |
|-------|--------|
| `INFO` | Inicio/cierre de app, venta creada, sync completado, backup realizado, cierre de caja |
| `WARN` | Producto no encontrado, sync sin internet, stock bajo |
| `ERROR` | Error de BD, error de API MP, fallo de backup |

---

## Migraciones de esquema versionadas

En lugar de una sola función `crear_tablas()`, se implementa un sistema de migraciones con versión. La tabla `schema_version` registra qué migraciones se han aplicado.

```sql
CREATE TABLE IF NOT EXISTS schema_version (
    version   INTEGER PRIMARY KEY,
    aplicado_en TEXT DEFAULT (datetime('now', 'localtime'))
);
```

```rust
// src-tauri/src/infrastructure/db/migrations.rs
const MIGRATIONS: &[(i32, &str)] = &[
    (1, "CREATE TABLE IF NOT EXISTS productos (...)"),
    (2, "CREATE TABLE IF NOT EXISTS ventas (...)"),
    (3, "CREATE TABLE IF NOT EXISTS venta_items (...)"),
    (4, "CREATE TABLE IF NOT EXISTS mp_pagos (...)"),
    (5, "CREATE TABLE IF NOT EXISTS configuracion (...)"),
    (6, "CREATE TABLE IF NOT EXISTS cierres_caja (...)"),
    (7, "CREATE TABLE IF NOT EXISTS categorias (...)"),
    (8, "ALTER TABLE productos ADD COLUMN categoria_id INTEGER"),
    (9, "CREATE TABLE IF NOT EXISTS devoluciones (...)"),
    (10, "CREATE TABLE IF NOT EXISTS devolucion_items (...)"),
];

pub fn ejecutar_migraciones(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY);"
    ).map_err(|e| e.to_string())?;

    let version_actual: i32 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))
        .unwrap_or(0);

    for (version, sql) in MIGRATIONS {
        if *version > version_actual {
            conn.execute_batch(sql).map_err(|e| format!("Migración {} falló: {}", version, e))?;
            conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [*version])
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
```

---

## Mantenimiento de base de datos (VACUUM)

Para evitar la fragmentación del archivo `.db` con el tiempo, se ejecuta `VACUUM` periódicamente:

```rust
// Se ejecuta después de cada cierre de caja
#[tauri::command]
fn mantener_bd(db: State<DbConn>) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA auto_vacuum=INCREMENTAL; VACUUM;")
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

El auto-vacuum en modo INCREMENTAL mantiene el tamaño del archivo acotado sin bloquear la BD durante operación normal.

---

## Auto-update (actualización automática)

Se utiliza `tauri-plugin-updater` para distribuir actualizaciones. El flujo es:

1. El desarrollador genera un nuevo build (`npm run tauri build`).
2. Sube el instalador `.exe` y `.msi` a GitHub Releases.
3. Actualiza el archivo `update.json` en un hosting estático (GitHub Pages, Vercel, Netlify, etc.).
4. Al iniciar la app, Tauri verifica el `update.json` comparando la versión actual con la disponible.
5. Si hay actualización, se muestra un diálogo "Nueva versión disponible. ¿Descargar ahora?".
6. La descarga e instalación ocurren en segundo plano; la app se reinicia automáticamente.

### Ejemplo de `update.json`
```json
{
  "version": "1.1.0",
  "notes": "Nueva versión: mejoras en conciliación y corrección de errores.",
  "pub_date": "2026-07-01T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "url": "https://github.com/user/kiosco-app/releases/download/v1.1.0/app_1.1.0_x64.msi"
    }
  }
}
```

---

## Testing

### Backend Rust (`cargo test`)

Los tests usan una BD SQLite en memoria para no depender de archivos externos:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::migrations;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::ejecutar_migraciones(&conn).unwrap();
        conn
    }

    #[test]
    fn test_buscar_producto_inexistente() {
        let conn = setup_db();
        let db = DbConn(Mutex::new(conn));
        let result = buscar_producto("9999999999999".to_string(), State::from(&db));
        assert!(result.unwrap().is_none());
    }
}
```

### Frontend Svelte (`vitest` + `@testing-library/svelte`)

```bash
npm install -D vitest @testing-library/svelte jsdom
```

Tests de componentes: renderizar POS con productos mock, simular escaneo, verificar que se agrega al carrito.

---

## Arranque con Windows

La aplicación puede configurarse para iniciar automáticamente con Windows. En Configuración hay un toggle "Iniciar con Windows" que:

1. Agrega un acceso directo a `shell:startup` apuntando al ejecutable.
2. Al desactivarlo, elimina el acceso directo.

Implementación en Rust usando `tauri-plugin-autostart` o manualmente:

```rust
use std::fs;
use std::os::windows::fs::MetadataExt;

fn habilitar_autostart(exe_path: &str) -> Result<(), String> {
    let startup_dir = std::env::var("APPDATA")
        .map_err(|e| e.to_string())? + r"\Microsoft\Windows\Start Menu\Programs\Startup";
    let lnk_path = format!(r"{}\KiosquitodoApp.lnk", startup_dir);
    // Crear acceso directo (requiere crate `lnk` o shell::CreateShortcut)
    Ok(())
}
```

---

## Sistema de Gestión de Kiosco — Integración de Hardware

> **Versión:** 1.1  
> **Alcance:** Integración del lector de código de barras USB 1D y la API de Mercado Pago.  
> **Nota:** Todos los montos se manejan en **centavos** (`i64`); el frontend formatea a pesos solo para mostrar.

---

## Parte 1 — Lector de código de barras USB 1D

### Modelos en consideración

| Modelo | Interfaz | Modo de operación |
|---|---|---|
| Prosoft S2100 | USB | HID (emulación de teclado) |
| Titanika Scan Rocket 1D USB | USB | HID (emulación de teclado) |
| Nictom YHD-8200 USB 1D con base | USB | HID (emulación de teclado) |

### Cómo funciona la emulación de teclado

Los tres modelos operan en modo **HID (Human Interface Device)**. Para Windows, son indistinguibles de un teclado USB. Cuando el escáner lee un código EAN-13, envía la secuencia de 13 caracteres como si el usuario los hubiera tecleado a ~1000 caracteres por segundo, seguida de un carácter terminador (`Enter` por defecto, configurable en algunos modelos vía software de configuración del fabricante).

**No se requiere ningún driver adicional.** Windows 11 detecta el escáner automáticamente con el driver HID genérico. El sistema no necesita ninguna biblioteca de terceros para comunicarse con el hardware.

---

### El problema central: distinguir escáner de teclado

Dado que el escáner emula un teclado, el WebView (Svelte) recibe los eventos `keydown` exactamente igual que si el usuario escribiera a mano. La clave para discriminar está en la **velocidad de entrada**:

| Tipo de entrada | Velocidad | Terminador |
|---|---|---|
| Usuario escribiendo | 5–10 caracteres / segundo | Ninguno (o Enter manual) |
| Escáner EAN-13 | 13 caracteres en < 50 ms | `Enter` automático |

La solución es un **buffer temporizado con timer de 80–100 ms** implementado en el componente Svelte del POS.

---

### Implementación: `scanner.ts`

```typescript
// src/lib/utils/scanner.ts
// Módulo reutilizable para discriminar input de escáner vs teclado manual

export function crearListenerEscaner(onScan: (codigo: string) => void): () => void {
  let buffer = '';
  let timer: ReturnType<typeof setTimeout> | null = null;
  const TIMEOUT_MS = 100; // Imposible para un escáner, holgado para un humano

  function onKeyDown(e: KeyboardEvent) {
    // Ignorar teclas de control que no forman parte del código
    if (['Shift', 'Control', 'Alt', 'Tab', 'CapsLock', 'Meta'].includes(e.key)) return;

    if (e.key === 'Enter') {
      // El terminador llegó: verificar que el buffer tenga longitud mínima
      // EAN-8 tiene 8 dígitos, EAN-13 tiene 13, UPC-A tiene 12
      if (buffer.length >= 8) {
        if (timer) clearTimeout(timer);
        const codigo = buffer.trim();
        buffer = '';
        onScan(codigo); // Callback con el código completo
        e.preventDefault(); // Evitar que el Enter afecte al formulario
      } else {
        // Buffer muy corto: era un Enter manual del usuario
        buffer = '';
      }
      return;
    }

    if (e.key.length === 1) {
      buffer += e.key;
      // Reiniciar el timer en cada keystroke
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        // Si el timer expira sin Enter → era input manual del teclado
        // Los caracteres ya se habrán ido al campo de texto activo normalmente
        buffer = '';
      }, TIMEOUT_MS);
    }
  }

  // Adjuntar al documento para captura global (funciona en cualquier pantalla)
  document.addEventListener('keydown', onKeyDown);

  // Retornar función de cleanup para usar en onDestroy de Svelte
  return () => document.removeEventListener('keydown', onKeyDown);
}
```

### Uso en el componente POS (`src/routes/+page.svelte`)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { crearListenerEscaner } from '$lib/utils/scanner';

  let carrito: any[] = $state([]);
  let ultimoProductoEscaneado: any = $state(null);
  let productoNoEncontrado = $state(false);

  async function handleScan(barcode: string) {
    productoNoEncontrado = false;
    try {
      const producto = await invoke('buscar_producto', { barcode });
      if (producto) {
        agregarAlCarrito(producto);
        ultimoProductoEscaneado = producto;
      } else {
        productoNoEncontrado = true;
        setTimeout(() => productoNoEncontrado = false, 3000);
      }
    } catch (err) {
      console.error('Error al buscar producto:', err);
    }
  }

  function agregarAlCarrito(producto: any) {
    const existente = carrito.find(i => i.producto_id === producto.id);
    if (existente) {
      existente.cantidad += 1;
      carrito = [...carrito];
    } else {
      carrito = [...carrito, {
        producto_id: producto.id,
        nombre: producto.nombre,
        precio_unitario: producto.precio_venta,
        cantidad: 1,
      }];
    }
  }

  let destruirListener: (() => void) | null = null;
  onMount(() => {
    destruirListener = crearListenerEscaner(handleScan);
  });
  onDestroy(() => destruirListener?.());
</script>
```

### Uso en modo "consulta de precio" (cualquier pantalla)

El mismo listener puede estar activo globalmente desde `+layout.svelte`. Si el escáner detecta un código fuera de la pantalla POS, se dispara un toast no intrusivo:

```svelte
<!-- src/routes/+layout.svelte -->
<script lang="ts">
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import { crearListenerEscaner } from '$lib/utils/scanner';
  import { formatearPesos } from '$lib/utils/moneda';

  let consultaActiva: any = $state(null);

  async function handleScanGlobal(barcode: string) {
    // Si el POS está activo, dejarlo manejar el scan (el POS tiene su propio listener)
    // Este listener solo actúa en otras pantallas
    if ($page.url.pathname === '/') return;

    const producto = await invoke('buscar_producto', { barcode });
    if (producto) {
      consultaActiva = producto;
      setTimeout(() => consultaActiva = null, 4000); // El toast se cierra solo
    }
  }
</script>

{#if consultaActiva}
  <div class="toast-precio">
    <span>{consultaActiva.nombre}</span>
    <span class="precio">{formatearPesos(consultaActiva.precio_venta)}</span>
  </div>
{/if}
```

---

### Comando Rust: `buscar_producto`

```rust
// src-tauri/src/infrastructure/api/producto_cmd.rs

use tauri::State;
use crate::infrastructure::db::DbConn;

#[derive(serde::Serialize)]
pub struct Producto {
    pub id: i64,
    pub nombre: String,
    pub barcode: Option<String>,
    pub precio_venta: i64, // centavos
    pub stock: i64,
}

#[tauri::command]
pub fn buscar_producto(
    barcode: String,
    db: State<DbConn>,
) -> Result<Option<Producto>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;

    let resultado = conn.query_row(
        "SELECT id, nombre, barcode, precio_venta, stock
         FROM productos
         WHERE barcode = ?1 AND activo = 1",
        rusqlite::params![barcode],
        |row| Ok(Producto {
            id:           row.get(0)?,
            nombre:       row.get(1)?,
            barcode:      row.get(2)?,
            precio_venta: row.get(3)?,
            stock:        row.get(4)?,
        }),
    );

    match resultado {
        Ok(producto)                      => Ok(Some(producto)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e)                            => Err(e.to_string()),
    }
}
```

### Latencia del ciclo completo

```
Escaneo físico (luz roja)
    ↓ ~20 ms   — el escáner decodifica el código
Envío de chars vía HID
    ↓ ~30 ms   — Windows procesa los keydown events
Timer de 100 ms en Svelte
    ↓ 100 ms   — discriminación escáner vs teclado
invoke('buscar_producto')
    ↓ ~2 ms    — IPC Tauri (misma máquina, pipe local)
SELECT en SQLite (sobre HDD)
    ↓ ~5 ms    — query simple con índice en barcode
Respuesta → actualización de UI
    ↓ ~1 ms    — Svelte actualiza el DOM

TOTAL: ~160 ms (muy por debajo del umbral perceptible de 300 ms)
```

---

### Configuración del terminador en el escáner

Por defecto, los tres modelos envían `Enter` como terminador. Si el cliente necesita cambiar este comportamiento (por ejemplo, para usar `Tab` como terminador en algún flujo específico), puede escanearse un código de configuración especial incluido en el manual de cada modelo. No se requiere software adicional.

---

## Parte 2 — Integración con la API de Mercado Pago

### Prerrequisito: obtener el Access Token

1. Ingresar a [mercadopago.com/developers](https://www.mercadopago.com/developers) con la cuenta del negocio.
2. Ir a **Mis aplicaciones → Crear aplicación** (nombre libre, por ejemplo "Kiosco Don Juan").
3. Copiar el **Access Token de producción** (formato: `APP_USR-123456789...`).
4. Ingresarlo en la pantalla de Configuración del sistema. Se guarda cifrado en el Windows Credential Store.

> **Importante:** el Access Token es equivalente a la contraseña de la cuenta de Mercado Pago. No debe compartirse, no debe guardarse en texto plano, y no debe incluirse en backups sin cifrado.

---

### Endpoint principal: `GET /v1/payments/search`

```
GET https://api.mercadopago.com/v1/payments/search
Authorization: Bearer {ACCESS_TOKEN}

Parámetros relevantes:
  status=approved           → solo pagos aprobados
  begin_date=NOW-1DAYS      → últimas 24 horas (soporta NOW-Xminutes/hours/days)
  end_date=NOW
  sort=date_approved
  criteria=desc
  limit=50                  → máximo por llamada (paginar si hay más)
  offset=0
```

Campos clave de cada pago en la respuesta:

| Campo | Descripción |
|---|---|
| `id` | ID único del pago en MP (se usa como clave para evitar duplicados) |
| `date_approved` | Timestamp ISO 8601 de aprobación |
| `transaction_amount` | Monto recibido |
| `status` | `approved`, `pending`, `rejected` |
| `status_detail` | `accredited` = dinero efectivamente en la cuenta |
| `description` | Descripción que ingresó el pagador (si la hay) |
| `payment_type_id` | `account_money` (transferencia desde cuenta MP), `debit_card`, etc. |
| `payer.email` | Email del pagador (si está disponible) |

> **Distinción crítica:** `status = approved` no garantiza que el dinero esté disponible. El campo definitivo es `status_detail = accredited`. Solo deben importarse pagos con ambas condiciones cumplidas.

---

### Implementación Rust: comando `sync_mercadopago`

```rust
// src-tauri/src/infrastructure/api/mercadopago_cmd.rs

use tauri::State;
use crate::infrastructure::db::DbConn;

#[derive(serde::Deserialize)]
struct MpPago {
    id: u64,
    date_approved: Option<String>,
    transaction_amount: f64, // la API de MP devuelve decimales
    description: Option<String>,
    status: String,
    status_detail: String,
}

#[derive(serde::Deserialize)]
struct MpPaging {
    total: u64,
    limit: u64,
    offset: u64,
}

#[derive(serde::Deserialize)]
struct MpSearchResponse {
    results: Vec<MpPago>,
    paging: MpPaging,
}

#[derive(serde::Serialize)]
pub struct SyncResultado {
    pub total_encontrados: usize,
    pub nuevos_insertados: usize,
    pub errores: Vec<String>,
}

#[tauri::command]
pub async fn sync_mercadopago(
    access_token: String,
    db: State<'_, DbConn>,
) -> Result<SyncResultado, String> {

    let url = "https://api.mercadopago.com/v1/payments/search\
               ?status=approved\
               &begin_date=NOW-1DAYS\
               &end_date=NOW\
               &sort=date_approved\
               &criteria=desc\
               &limit=50";

    // La llamada HTTP es async: NO se adquiere el lock de la BD durante la espera de red.
    // Si se tomara el lock aquí, la UI se congelaría durante ~15s (timeout HTTP).
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Sin conexión a internet: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Error de API de MP: {} — verificar que el Access Token sea válido",
            response.status()
        ));
    }

    let data: MpSearchResponse = response
        .json()
        .await
        .map_err(|e| format!("Error al parsear respuesta MP: {}", e))?;

    // Filtrar solo los efectivamente acreditados
    let pagos_validos: Vec<&MpPago> = data.results
        .iter()
        .filter(|p| p.status == "approved" && p.status_detail == "accredited")
        .collect();

    // El lock de la BD se adquiere SOLO para escribir, después de la llamada HTTP.
    // Esto evita bloquear la UI durante el tiempo de red (~15s de timeout).
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut insertados = 0;
    let mut errores = vec![];

    for pago in &pagos_validos {
        // Conversión A LA ENTRADA: el monto decimal de la API pasa a CENTAVOS (i64).
        // redondeo con .round() para absorver ruido de punto flotante (ej: 1500.0000001).
        let monto_centavos = (pago.transaction_amount * 100.0).round() as i64;

        // INSERT OR IGNORE: si el mp_id ya existe en la tabla, la operación
        // no hace nada. Esto garantiza idempotencia: N syncs nunca duplican datos.
        match conn.execute(
            "INSERT OR IGNORE INTO mp_pagos
             (mp_id, fecha_aprobacion, monto, descripcion)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                pago.id.to_string(),
                pago.date_approved,
                monto_centavos,       // INTEGER centavos
                pago.description,
            ],
        ) {
            Ok(1) => insertados += 1,
            Ok(0) => {} // Ya existía, se ignora silenciosamente
            Err(e) => errores.push(format!("mp_id {}: {}", pago.id, e)),
        }
    }

    // Intentar matching automático después de importar
    aplicar_matching_automatico(&conn);

    Ok(SyncResultado {
        total_encontrados: pagos_validos.len(),
        nuevos_insertados: insertados,
        errores,
    })
}

// Matching automático: vincula ventas MP con pagos MP cuando
// el monto coincide ±100 centavos (±$1) y el horario difiere en ≤10 minutos
fn aplicar_matching_automatico(conn: &rusqlite::Connection) {
    let _ = conn.execute_batch("
        UPDATE mp_pagos
        SET venta_id = (
            SELECT v.id
            FROM ventas v
            WHERE v.medio_pago IN ('mercado_pago', 'mixto')
              AND mp_pagos.venta_id IS NULL
              AND ABS(v.monto_mp - mp_pagos.monto) <= 100 -- 100 centavos = $1
              AND ABS(
                  strftime('%s', v.hora) - strftime('%s', mp_pagos.fecha_aprobacion)
              ) <= 600  -- 10 minutos en segundos
            LIMIT 1
        )
        WHERE venta_id IS NULL;
    ");
}
```

---

### Query de conciliación en la UI

```sql
-- Vista de conciliación del día
-- Muestra todas las ventas MP y su estado de coincidencia con la API
SELECT
    v.id                              AS venta_id,
    v.hora                            AS hora_venta,
    v.monto_mp                        AS monto_registrado,
    v.descripcion_mp                  AS descripcion_operador,
    mp.mp_id                          AS mp_id,
    mp.fecha_aprobacion               AS hora_mp,
    mp.monto                          AS monto_mp_api,
    mp.descripcion                    AS descripcion_pagador,
    CASE
        WHEN mp.mp_id IS NULL
            THEN 'SIN COINCIDENCIA EN MP'
        WHEN ABS(v.monto_mp - mp.monto) > 1 -- > 1 centavo = monto difiere
            THEN 'MONTO DIFIERE'
        ELSE 'OK'
    END                               AS estado
FROM ventas v
LEFT JOIN mp_pagos mp ON mp.venta_id = v.id
WHERE v.medio_pago IN ('mercado_pago', 'mixto')
  AND DATE(v.hora) = DATE('now', 'localtime')
ORDER BY v.hora DESC;
```

---

### Estrategias de conciliación

#### Estrategia A — Vinculación manual asistida (MVP)

La UI muestra dos listas lado a lado. El operador hace clic en una venta y luego en el pago de MP correspondiente para vincularlos. El sistema calcula el estado automáticamente.

**Ideal para el MVP** porque no requiere cambios en la operatoria actual del cliente.

#### Estrategia B — Matching automático por monto + ventana de tiempo (MVP mejorado)

Si existe exactamente un pago de MP con el mismo monto ± 100 centavos (± $1) dentro de una ventana de ±10 minutos del registro manual, se vinculan automáticamente y se marcan como "sugerido". El operador confirma con un clic.

El algoritmo está implementado en `aplicar_matching_automatico()` arriba.

#### Estrategia C — QR con `external_reference` (segunda iteración)

Si el cliente migra a cobrar con el QR propio de Mercado Pago (generado desde la API), el sistema puede incluir un `external_reference` igual al ID de la venta en cada cobro. Cuando el cliente paga, ese ID viaja en el pago y la conciliación es automática al 100%.

Esta es la integración ideal a largo plazo, pero requiere que el cliente cambie su operatoria de cobro.

---

### Manejo de errores y casos borde

| Situación | Comportamiento |
|---|---|
| Sin conexión a internet | El sync falla silenciosamente; se muestra un badge "Sin sync" en la UI. El sistema sigue operando offline. |
| Access Token inválido o vencido | La API devuelve HTTP 401; el sistema muestra un alerta para que el operador actualice el token en Configuración. |
| Más de 50 pagos en 24 horas | El sistema realiza múltiples llamadas con `offset` incremental hasta traer todos los resultados (paginación automática). |
| Pago en MP que no tiene venta registrada | Aparece como "huérfano" en la conciliación (fila sin venta_id). El operador puede registrar la venta retroactivamente. |
| Venta registrada que no tiene pago en MP | Estado `SIN COINCIDENCIA EN MP`. Puede ser un pago aún no procesado, o un error de registro. |
| Sync ejecutado múltiples veces | `INSERT OR IGNORE` garantiza que el mismo pago nunca se duplica en la base de datos. |

---

### Diagrama de flujo completo

```
[Operador registra venta en MP]
         ↓
[SQLite: INSERT en ventas (medio_pago='mercado_pago')]
         ↓
[Cada 15 min o botón manual]
         ↓
[Rust: sync_mercadopago()]
         ↓
[HTTPS GET → api.mercadopago.com/v1/payments/search]
         ↓ (falla silenciosa si sin internet)
[Filtrar status=approved AND status_detail=accredited]
         ↓
[SQLite: INSERT OR IGNORE INTO mp_pagos]
         ↓
[aplicar_matching_automatico()]
         ↓
[UI Svelte: pantalla de conciliación]
  ├── Ventas registradas  ←→  Pagos recibidos de MP
  ├── Estado: OK / SIN COINCIDENCIA / MONTO DIFIERE
  └── Vinculación manual para los casos sin match
         ↓
[Cierre de caja: delta = Σ mp_pagos.monto - Σ ventas.monto_mp]  // en centavos
```

---

## Atajos de teclado del sistema

La aplicación debe responder a los siguientes atajos globales (implementados con un listener `keydown` en `App.svelte`):

| Tecla | Acción |
|-------|--------|
| `F1` | Abrir POS |
| `F2` | Abrir Inventario |
| `F3` | Abrir Conciliación |
| `F4` | Abrir Cierre de Caja |
| `F5` | Abrir Configuración |
| `F6` | Abrir Devoluciones |
| `Escape` | Cancelar operación actual / cerrar modal |
| `Enter` (en POS con carrito) | Abrir diálogo de cobro |
| `Ctrl+E` | Cobrar en efectivo (desde diálogo de cobro) |
| `Ctrl+M` | Cobrar con Mercado Pago (desde diálogo de cobro) |

```typescript
// src/lib/stores/app.svelte.ts
import { goto } from '$app/navigation';

export function configurarAtajos() {
  document.addEventListener('keydown', (e: KeyboardEvent) => {
    if (e.key === 'F1') { e.preventDefault(); goto('/'); }
    if (e.key === 'F2') { e.preventDefault(); goto('/inventario'); }
    if (e.key === 'F3') { e.preventDefault(); goto('/conciliacion'); }
    if (e.key === 'F4') { e.preventDefault(); goto('/cierre'); }
    if (e.key === 'F5') { e.preventDefault(); goto('/configuracion'); }
    if (e.key === 'F6') { e.preventDefault(); goto('/devoluciones'); }
    if (e.key === 'Escape') { cerrarModal(); }
  });
}
```

---

## Indicador de conectividad

La aplicación debe mostrar permanentemente el estado de conectividad a internet en la barra inferior. En lugar de un endpoint separado, se deduce del resultado del último sync automático con Mercado Pago.
### Implementación en Svelte (`+layout.svelte`)

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  let conectado = $state(true);
  let ultimoSyncOk = $state<boolean | null>(null);
  let sincronizando = $state(false);

  function onSyncResult(exito: boolean) {
    conectado = exito;
    ultimoSyncOk = exito;
  }
</script>

<div class="barra-estado">
  <span class:conectado class:desconectado={!conectado}>
    {conectado ? '● Conectado' : '● Sin conexión'}
  </span>
  {#if sincronizando}
    <span>Sincronizando MP...</span>
  {/if}
</div>
```

> **Nota:** No se implementa `verificar_conectividad()` como comando separado porque el sync con MP (`sync_mercadopago`) ya detecta la falta de conexión. Un endpoint adicional agregaría latencia y consumo de API innecesarios. El badge de conectividad se actualiza con el resultado del último sync.
