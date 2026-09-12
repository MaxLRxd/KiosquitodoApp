/**
 * Tipos compartidos con el backend Rust.
 * IMPORTANTE: los montos viajan SIEMPRE en centavos (i64) y los campos
 * usan snake_case (serde de Rust). La conversión a decimales ocurre solo
 * en la capa de presentación (lib/utils/moneda.ts).
 */

export interface Producto {
	id: number;
	nombre: string;
	barcode: string | null;
	precio_venta: number; // centavos
	precio_costo: number | null; // centavos
	stock: number;
	stock_minimo: number;
	activo: boolean;
	categoria_id: number | null;
}

export interface Categoria {
	id: number;
	nombre: string;
	color: string | null;
}

export interface CategoriaNueva {
	nombre: string;
	color?: string | null;
}

/** Comando de alta de producto (espejo de `NuevoProducto` en Rust). */
export interface NuevoProducto {
	nombre: string;
	barcode: string | null;
	precio_venta: number; // centavos
	precio_costo: number | null; // centavos
	stock_inicial: number;
	stock_minimo: number;
	categoria_id: number | null;
}

export interface ItemVenta {
	producto_id: number;
	cantidad: number;
	precio_unitario: number; // centavos
}

export type MedioPago = 'efectivo' | 'mercado_pago' | 'mixto';

export interface Venta {
	id: number;
	items: ItemVenta[];
	total: number; // centavos
	medio_pago: MedioPago;
	monto_efectivo: number; // centavos
	monto_mp: number; // centavos
	descripcion_mp: string | null;
	hora: string;
}

/** Resumen diario del cierre de caja (espejo de `ResumenCierre` en Rust). */
export interface ResumenCierre {
	total_efectivo: number; // centavos
	total_mp: number; // centavos
	total_ventas: number; // centavos
	cantidad_ventas: number;
	margen_bruto: number; // centavos
	/** Σ pagos MP acreditados − Σ ventas MP registradas. >0 = la API muestra más. */
	delta_mp: number;
}

/** Cierre persista (espejo de `CierrePersistido` en Rust). */
export interface CierrePersistido {
	id: number;
	fecha: string; // YYYY-MM-DD
	resumen: ResumenCierre;
}

export interface MpPago {
	id: number;
	mp_id: string;
	fecha_aprobacion: string | null;
	monto: number; // centavos
	descripcion: string | null;
	venta_id: number | null;
}

/** Comando de entrada para registrar una venta (espejo de `NuevaVenta` en Rust). */
export interface NuevaVenta {
	items: ItemVenta[];
	medio_pago: MedioPago;
	monto_efectivo: number; // centavos
	monto_mp: number; // centavos
	descripcion_mp?: string | null;
}