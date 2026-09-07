# Sistema de Gestión de Kiosco — Integración de Hardware

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
