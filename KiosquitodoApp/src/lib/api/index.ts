import { invoke, isTauri } from '@tauri-apps/api/core';
import type {
	Categoria,
	ConfiguracionApp,
	CategoriaNueva,
	CierrePersistido,
	DevolucionRegistrada,
	ItemVenta,
	NuevoProducto,
	NuevaVenta,
	Producto,
	ResumenCierre,
	RutasApp,
	Venta
} from '$lib/types';
import { ejecutarMock } from './mock';

/**
 * Capa de acceso al backend.
 *
 * Dentro del runtime Tauri delega en `invoke` (commands Rust reales); en el
 * navegador (`npm run dev`, tests) usa el backend simulado en memoria de
 * `./mock.ts` para que el prototipo sea navegable sin que el árbol compile.
 */

async function ejecutar<T>(comando: string, args: Record<string, unknown> = {}): Promise<T> {
	if (isTauri()) {
		return invoke<T>(comando, args);
	}
	return ejecutarMock<T>(comando, args);
}

// ── Productos ─────────────────────────────────────────────────────────────

export function buscarProductoPorBarcode(barcode: string): Promise<Producto | null> {
	return ejecutar<Producto | null>('buscar_producto_por_barcode', { barcode });
}

export function listarProductos(
	filtro: string,
	categoriaId: number | null = null,
	limite = 50,
	offset = 0
): Promise<Producto[]> {
	return ejecutar<Producto[]>('listar_productos', {
		filtro,
		categoria_id: categoriaId,
		limite,
		offset
	});
}

export function crearProducto(nuevo: NuevoProducto): Promise<Producto> {
	return ejecutar<Producto>('crear_producto', { nuevo });
}

export function actualizarProducto(producto: Producto): Promise<void> {
	return ejecutar<void>('actualizar_producto', { producto });
}

export function eliminarProducto(id: number): Promise<void> {
	return ejecutar<void>('eliminar_producto', { id });
}

export function ajustarStock(
	id: number,
	stockNuevo: number,
	motivo: string,
	operador: string | null = null
): Promise<void> {
	return ejecutar<void>('ajustar_stock', { id, stock_nuevo: stockNuevo, motivo, operador });
}

// ── Categorías ────────────────────────────────────────────────────────────

export function listarCategorias(): Promise<Categoria[]> {
	return ejecutar<Categoria[]>('listar_categorias', {});
}

export function crearCategoria(nueva: CategoriaNueva): Promise<Categoria> {
	return ejecutar<Categoria>('crear_categoria', { nueva });
}

// ── Ventas y devoluciones ───────────────────────────────────────────────────

export function registrarVenta(nueva: NuevaVenta): Promise<Venta> {
	return ejecutar<Venta>('registrar_venta', { nueva });
}

/** Obtiene una venta completa (con ítems) por id. */
export function obtenerVenta(id: number): Promise<Venta | null> {
	return ejecutar<Venta | null>('obtener_venta', { id });
}

/** Lista las ventas del día (`YYYY-MM-DD`), más recientes primero. */
export function listarVentasDia(fecha: string): Promise<Venta[]> {
	return ejecutar<Venta[]>('listar_ventas_dia', { fecha });
}

/** Registra una devolución y repone el stock de los ítems devueltos. */
export function registrarDevolucion(
	ventaId: number,
	items: ItemVenta[],
	motivo: string | null = null
): Promise<void> {
	return ejecutar<void>('registrar_devolucion', { venta_id: ventaId, items, motivo });
}

/** Devuelve las devoluciones recientes, más nuevas primero. */
export function listarDevoluciones(): Promise<DevolucionRegistrada[]> {
	return ejecutar<DevolucionRegistrada[]>('listar_devoluciones', {});
}

// ── Cierre de caja ─────────────────────────────────────────────────────────

/** Resumen del día sin persistir (previsualización del cierre). */
export function resumenDiario(fecha: string): Promise<ResumenCierre> {
	return ejecutar<ResumenCierre>('resumen_diario', { fecha });
}

/** Consolida y persiste el cierre de caja de `fecha` (YYYY-MM-DD). */
export function cerrarCaja(fecha: string): Promise<CierrePersistido> {
	return ejecutar<CierrePersistido>('cerrar_caja', { fecha });
}

/** Historial de cierres, más reciente primero. */
export function listarCierres(): Promise<CierrePersistido[]> {
	return ejecutar<CierrePersistido[]>('listar_cierres', {});
}

/** Exporta los cierres del rango `[desde, hasta]` a CSV (montos en centavos). */
export function exportarCierresCsv(desde: string, hasta: string): Promise<string> {
	return ejecutar<string>('exportar_cierres_csv', { desde, hasta });
}

// ── Configuración del sistema ─────────────────────────────────────────────

/** Rutas resueltas por el backend (datos, backups, logs y BD). */
export function obtenerRutas(): Promise<RutasApp> {
	return ejecutar<RutasApp>('obtener_rutas', {});
}

/** Crea un backup manual (VACUUM INTO) y devuelve la ruta generada. */
export function crearBackupManual(): Promise<string> {
	return ejecutar<string>('crear_backup_manual', {});
}

/** Lee las preferencias guardadas (defaults para claves ausentes). */
export function obtenerConfiguracion(): Promise<ConfiguracionApp> {
	return ejecutar<ConfiguracionApp>('obtener_configuracion', {});
}

/** Persiste las preferencias y devuelve el estado normalizado. */
export function guardarConfiguracion(
	configuracion: ConfiguracionApp
): Promise<ConfiguracionApp> {
	return ejecutar<ConfiguracionApp>('guardar_configuracion', { configuracion });
}