import { describe, expect, it, beforeEach } from 'vitest';
import { crearCarrito } from './carrito.svelte';
import type { Producto } from '$lib/types';

function producto(id: number, nombre: string, precioVenta = 1000, stock = 10): Producto {
	return {
		id,
		nombre,
		barcode: null,
		precio_venta: precioVenta,
		precio_costo: 700,
		stock,
		stock_minimo: 2,
		activo: true,
		categoria_id: null
	};
}

describe('carrito', () => {
	let caja: ReturnType<typeof crearCarrito>;

	beforeEach(() => {
		caja = crearCarrito();
	});

	it('arranca vacío', () => {
		expect(caja.items).toEqual([]);
		expect(caja.cantidadItems).toBe(0);
		expect(caja.total).toBe(0);
	});

	it('agrega un ítem y calcula el total en centavos', () => {
		caja.agregar(producto(1, 'Alfajor', 150000));
		expect(caja.cantidadItems).toBe(1);
		expect(caja.total).toBe(150000);
	});

	it('acumula cantidad si el ítem ya está', () => {
		const p = producto(1, 'Gaseosa', 200000);
		caja.agregar(p, 2);
		caja.agregar(p, 3);
		expect(caja.items).toHaveLength(1);
		expect(caja.items[0].cantidad).toBe(5);
		expect(caja.cantidadItems).toBe(5);
		expect(caja.total).toBe(5 * 200000);
	});

	it('quita un ítem', () => {
		caja.agregar(producto(1, 'A'), 2);
		caja.agregar(producto(2, 'B'), 1);
		caja.quitar(1);
		expect(caja.items.map((it) => it.producto.id)).toEqual([2]);
	});

	it('asignarCantidad con 0 elimina el ítem', () => {
		caja.agregar(producto(1, 'A'));
		caja.asignarCantidad(1, 0);
		expect(caja.items).toHaveLength(0);
	});

	it('asignarCantidad negativa también elimina', () => {
		caja.agregar(producto(1, 'A'));
		caja.asignarCantidad(1, -3);
		expect(caja.items).toHaveLength(0);
	});

	it('vacíar limpia todo', () => {
		caja.agregar(producto(1, 'A'));
		caja.agregar(producto(2, 'B'), 3);
		caja.vaciar();
		expect(caja.items).toHaveLength(0);
		expect(caja.total).toBe(0);
	});
});