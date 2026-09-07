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

export interface ItemVenta {
	producto_id: number;
	cantidad: number;
	precio_unitario: number; // centavos
}

export type MedioPago = 'efectivo' | 'mercado_pago' | 'mixto';

export interface Venta {
	id: number;
	total: number; // centavos
	medio_pago: MedioPago;
	monto_efectivo: number; // centavos
	monto_mp: number; // centavos
	descripcion_mp: string | null;
	hora: string;
}

export interface MpPago {
	id: number;
	mp_id: string;
	fecha_aprobacion: string | null;
	monto: number; // centavos
	descripcion: string | null;
	venta_id: number | null;
}