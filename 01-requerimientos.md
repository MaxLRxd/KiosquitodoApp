# Sistema de Gestión de Kiosco — Requerimientos (MVP)

> **Versión:** 1.1  
> **Estado:** Aprobado (base del desarrollo en curso)  
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
| Frontend | SvelteKit en la raíz del repo, `adapter-static` en modo SPA (`ssr=false`, fallback `index.html`). |

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
