# Sistema de Gestión de Kiosco — Stack Tecnológico y Arquitectura

> **Versión:** 1.1  
> **Premisa:** Sistema de escritorio 100% local, sin servidor, sin costos de hosting. Optimizado para hardware de gama baja (Celeron N4020, 4 GB RAM, HDD).  
> **Estado:** En desarrollo — estructura aprobada; Fase 1 backend + scaffold frontend implementados.

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

Estructura implementada (aprobada): backend en `src-tauri/` (layout estándar de Tauri, cero fricción con el CLI) y frontend SvelteKit en la raíz. El backend sigue **arquitectura hexagonal / clean architecture**, adaptando las convenciones Spring Boot a nombres idiomáticos de Rust:

| Capa Spring Boot | Equivalente Rust | Responsabilidad |
|---|---|---|
| `controllers` | `infrastructure/api/` (comandos Tauri) | Capa de entrada IPC, delgada: valida y delega |
| `services` | `application/` (casos de uso) | Orquesta reglas de negocio vía puertos (traits) |
| `repositories` | `ports/` (traits) + `infrastructure/db/` | Contratos + adaptadores SQLite |
| `entities/models` | `domain/` | Entidades y lógica de negocio pura (sin I/O, sin SQL) |
| `config` / `errors` | `config/`, `errors/` | Configuración de rutas/defaults y errores tipados |

```
KiosquitodoApp/
├── src/                            # Frontend SvelteKit (raíz)
│   ├── routes/                     #   +layout (sidebar + barra estado + atajos F1–F6) y páginas de módulos
│   ├── lib/                        #   components/, stores/, api/, utils/, types/
│   ├── app.html · app.css · app.d.ts
├── static/                         # Assets estáticos (favicon.png, sounds/*.wav)
├── package.json · svelte.config.js · vite.config.ts (puerto 1420, strictPort)
├── tailwind.config.js · postcss.config.js
├── src-tauri/                      # Backend Rust (Tauri v2)
│   ├── Cargo.toml · build.rs · tauri.conf.json (devUrl 1420, frontendDist ../build)
│   ├── capabilities/default.json   #   Permisos IPC (core:default)
│   ├── icons/                      #   Iconos del instalador
│   └── src/
│       ├── main.rs                 # Binario mínimo → llama a run()
│       ├── lib.rs                  # Composición raíz: logging/backup/autostart + wiring de dependencias
│       ├── config/                 # Rutas (data, backups, logs) y valores por defecto
│       ├── errors/                 # AppError, AppResult<T> y conversiones From
│       ├── domain/                 # producto, venta, pago, conciliacion, cierre — reglas puras
│       ├── ports/                  # traits: producto_repo, venta_repo, pago_repo, cierre_repo, mp_cliente, credencial_repo
│       ├── application/            # producto_service, venta_service, conciliacion_service, cierre_service
│       └── infrastructure/
│           ├── api/                # Comandos Tauri (producto_cmd, venta_cmd, mercadopago_cmd, caja_cmd, config_cmd)
│           ├── db/                 # mod.rs (conexión + PRAGMAs WAL), migrations.rs, repos/*_sqlite.rs
│           ├── mercadopago/        # Cliente HTTP real de MP (reqwest)
│           └── system/             # logging, backup, keyring, autostart
```

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
