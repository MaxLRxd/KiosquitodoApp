import { describe, expect, it } from 'vitest';
import { coincideBusqueda, normalizarTexto } from './busqueda';

describe('normalizarTexto', () => {
	it('quita acentos, mayúsculas y puntuación', () => {
		expect(normalizarTexto('Café  instantáneo-50g')).toBe('cafe instantaneo 50g');
		expect(normalizarTexto('Coca-Cola 500ml')).toBe('coca cola 500ml');
	});
});

describe('coincideBusqueda', () => {
	it('encuentra por tokens aunque haya guiones', () => {
		expect(coincideBusqueda('Coca-Cola 500ml', 'coca cola')).toBe(true);
		expect(coincideBusqueda('Coca-Cola 500ml', 'cola')).toBe(true);
	});

	it('ignora acentos', () => {
		expect(coincideBusqueda('Café instantáneo 50g', 'cafe')).toBe(true);
	});

	it('exige que aparezcan todos los términos', () => {
		expect(coincideBusqueda('Coca-Cola 500ml', 'coca agua')).toBe(false);
	});
});
