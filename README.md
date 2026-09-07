# Sistema de Gestión de Kiosco — Documentación Técnica

> **Proyecto:** Sistema de gestión local para kiosco  
> **Última actualización:** Septiembre 2026


## Resumen ejecutivo

### El problema

El cliente opera un kiosco y lleva el inventario y las cuentas en papel. En días de alto tráfico, el registro manual de transferencias por Mercado Pago genera descontrol y descuadres en la caja.

### La solución

Una aplicación de escritorio local (sin servidor, sin costos de hosting) que reemplaza el papel con:

- Un punto de venta con soporte de escáner de código de barras.
- Sincronización automática con la API de Mercado Pago para traer los pagos recibidos.
- Una pantalla de conciliación que compara lo registrado por el operador contra lo acreditado por MP.
- Cierre de caja diario con el delta de conciliación explícito.

### Decisiones clave

| Decisión | Elección | Razón principal |
|---|---|---|
| Framework de escritorio | Tauri v2 | Sin Chromium embebido → 50–100 MB RAM total |
| Lenguaje backend | Rust | Sin GC, sin runtime, máximo rendimiento en hardware bajo |
| UI | Svelte v5 | Sin Virtual DOM en ejecución, bundles mínimos |
| Base de datos | SQLite | Sin servidor, archivo único, cero configuración |
| Integración escáner | HID nativo + timer JS | Sin driver adicional, plug-and-play |
| Integración MP | Polling a `/v1/payments/search` | Sin servidor público para recibir webhooks |

### Decisiones de implementación (aprobadas — septiembre 2026)

| Decisión | Detalle |
|---|---|
| Precisión monetaria | Todos los montos se modelan en **centavos** (`INTEGER`/`i64`) desde el día 1; conversión a decimales solo en la UI. |
| Backend | `src-tauri/` con arquitectura en capas: `domain → ports → application → infrastructure`. |
| Frontend | En `KiosquitodoApp/` (subcarpeta del repo): SvelteKit + `adapter-static` en modo SPA (`ssr=false`). |
| Credenciales | `keyring` v2 usa el Windows Credential Store por defecto (sin features extra). |
| Stack resuelto | SvelteKit 2.x + Vite 8.x (Vite 5 quedó EOL) + Tailwind 3.4. |

### Uso estimado de RAM

```
Windows 11 base:        ~2.000 MB
Antivirus / SO:           ~200 MB
Aplicación completa:      ~100 MB
─────────────────────────────────
Total:                 ~2.300 MB
Margen libre (4 GB):  ~1.700 MB
```

---

## Stack rápido

```
Tauri v2
├── Backend: Rust (src-tauri/)
│   ├── rusqlite 0.32     (SQLite embebida, "bundled")
│   ├── reqwest 0.12      (HTTP async para API de MP, rustls-tls)
│   ├── serde_json 1.x    (serialización)
│   ├── tokio 1.x         (runtime async)
│   ├── thiserror 2.x     (errores tipados)
│   ├── keyring 2.x       (Access Token en Windows Credential Store)
│   └── flexi_logger 0.29 (logs rotativos)
└── Frontend (KiosquitodoApp/): Svelte 5 + SvelteKit 2 + Tailwind CSS 3.4 + Vite 8
```

---

## Hardware compatible (escáneres)

- Prosoft S2100 (USB HID)
- Titanika Scan Rocket 1D USB (USB HID)
- Nictom YHD-8200 USB 1D con base (USB HID)

Cualquier escáner USB que opere en modo HID (emulación de teclado) es compatible sin configuración adicional.

---

## Metodología de desarrollo: SPEC driven con IA

Este proyecto se desarrolla con un flujo **spec-first con asistencia de IA**, distinto del enfoque tradicional (documento completo → equipo → implementación). El patrón es:

1. **Especificación primero**: el **SPEC** (`SPEC.md`) en la raíz del repo es la **fuente de verdad**: se definen compromisos y reglas antes de escribir código. `IMPLEMENTATION.md` registra después cómo el código los cumple.
2. **Decisión por decisión**: cada regla importante (ej. montos en centavos, `src-tauri/`, capas `domain → ports → application → infrastructure`) se discute, se aprueba explícitamente y se documenta antes de implementarla.
3. **Fases incrementales**: el desarrollo avanza en fases pequeñas y verificables (Fase 1 backend, scaffold frontend, etc.). Cada fase termina con verificación real (`cargo check`, `npm run check`, build) antes de seguir.
4. **La IA como par**: un agente implementa siguiendo la spec, el humano aprueba; cuando una decisión cambia, **la documentación se actualiza primero** y el código se ajusta después.

### Diferencias con el flujo tradicional

| Aspecto | Tradicional | SPEC driven con IA |
|---|---|---|
| Documentación | Documento único y extenso, escrito por adelantado | Especificaciones **vivas y acotadas** por módulo/iteración |
| Ciclo | Spec → equipo → código | Spec → aprobación → implementación → verificación |
| Cambios de decisiones | Reuniones y re-aprobación del doc maestro | `Update` documental inmediato + ajuste de código |
| Verificación | En entregas | Continua, por fase (`cargo test`, `npm run check`) |

> **Resultado:** especificaciones siempre alineadas con la realidad del código y decisiones rastreables en el histórico de la documentación.