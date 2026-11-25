import { beforeEach, describe, expect, it } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';
import { carrito } from '$lib/stores/carrito.svelte';
import { reiniciarTiendaMock } from '$lib/api/mock';

function escanear(codigo: string): void {
	for (const caracter of codigo) {
		document.dispatchEvent(
			new KeyboardEvent('keydown', { key: caracter, bubbles: true, cancelable: true })
		);
	}
	document.dispatchEvent(
		new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })
	);
}

beforeEach(() => {
	reiniciarTiendaMock();
	carrito.vaciar();
});

describe('POS', () => {
	it('muestra el catálogo con stock al abrir', async () => {
		render(Page);

		expect(await screen.findByText('Agua sin gas 500ml')).toBeInTheDocument();
		expect(screen.getByText(/Catálogo · \d+ con stock/)).toBeInTheDocument();
	});

	it('busca por nombre parcial con espacios', async () => {
		render(Page);

		const buscador = screen.getByPlaceholderText(/Escaneá un código/);
		await fireEvent.input(buscador, { target: { value: 'coca cola' } });

		expect(await screen.findByText('Coca-Cola 500ml')).toBeInTheDocument();
	});

	it('agrega al carrito el producto escaneado', async () => {
		render(Page);

		escanear('7790000000016');

		await waitFor(() => expect(carrito.cantidadItems).toBe(1));
		expect(carrito.items[0].producto.nombre).toBe('Coca-Cola 500ml');
	});

	it('no agrega nada si el código escaneado no existe', async () => {
		render(Page);

		escanear('9999999999999');
		await new Promise((resolve) => setTimeout(resolve, 80));

		expect(carrito.cantidadItems).toBe(0);
	});
});
