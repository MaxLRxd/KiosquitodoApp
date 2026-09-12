import type { Producto } from '$lib/types';

export interface ItemCarrito {
	producto: Producto;
	cantidad: number;
}

export function crearCarrito() {
	let items = $state<ItemCarrito[]>([]);

	return {
		get items(): ItemCarrito[] {
			return items;
		},
		get cantidadItems(): number {
			return items.reduce((acc, it) => acc + it.cantidad, 0);
		},
		get total(): number {
			return items.reduce((acc, it) => acc + it.producto.precio_venta * it.cantidad, 0);
		},
		agregar(producto: Producto, cantidad = 1): void {
			const existente = items.find((it) => it.producto.id === producto.id);
			if (existente) existente.cantidad += cantidad;
			else items = [...items, { producto, cantidad }];
		},
		quitar(productoId: number): void {
			items = items.filter((it) => it.producto.id !== productoId);
		},
		asignarCantidad(productoId: number, cantidad: number): void {
			const item = items.find((it) => it.producto.id === productoId);
			if (!item) return;
			if (cantidad <= 0) items = items.filter((it) => it.producto.id !== productoId);
			else item.cantidad = cantidad;
		},
		vaciar(): void {
			items = [];
		}
	};
}

/** Carrito del POS (singleton de la app). */
export const carrito = crearCarrito();