import { afterEach, describe, expect, it, vi } from 'vitest';
import { crearListenerEscaneo, esCodigoBarras } from './scanner';

const CODIGO = '7790000000016';

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

const limpiezas: Array<() => void> = [];

function escuchar(onScan: (codigo: string) => void): void {
	limpiezas.push(crearListenerEscaneo(onScan));
}

afterEach(() => {
	while (limpiezas.length > 0) limpiezas.pop()?.();
	vi.useRealTimers();
	document.body.innerHTML = '';
});

describe('esCodigoBarras', () => {
	it('acepta EAN-8, UPC-A y EAN-13', () => {
		expect(esCodigoBarras('12345678')).toBe(true);
		expect(esCodigoBarras('123456789012')).toBe(true);
		expect(esCodigoBarras('1234567890123')).toBe(true);
	});

	it('tolera espacios y rechaza largos o letras', () => {
		expect(esCodigoBarras(`  ${CODIGO}  `)).toBe(true);
		expect(esCodigoBarras('1234567')).toBe(false);
		expect(esCodigoBarras('123456789')).toBe(false);
		expect(esCodigoBarras('12345678901234')).toBe(false);
		expect(esCodigoBarras('coca cola')).toBe(false);
		expect(esCodigoBarras('77900000A0016')).toBe(false);
	});
});

describe('crearListenerEscaneo', () => {
	it('detecta un escaneo rápido terminado en Enter', () => {
		const onScan = vi.fn();
		escuchar(onScan);

		escanear(CODIGO);

		expect(onScan).toHaveBeenCalledTimes(1);
		expect(onScan).toHaveBeenCalledWith(CODIGO);
	});

	it('no confunde el tipeo humano lento con un escaneo', () => {
		vi.useFakeTimers();
		const onScan = vi.fn();
		escuchar(onScan);

		for (const caracter of CODIGO) {
			document.dispatchEvent(new KeyboardEvent('keydown', { key: caracter, bubbles: true }));
			vi.advanceTimersByTime(150);
		}
		document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));

		expect(onScan).not.toHaveBeenCalled();
	});

	it('restaura el valor de un campo que no es el buscador', () => {
		const campo = document.createElement('input');
		campo.value = '1500';
		document.body.appendChild(campo);
		campo.focus();

		const onScan = vi.fn();
		escuchar(onScan);

		for (const caracter of CODIGO) {
			document.dispatchEvent(new KeyboardEvent('keydown', { key: caracter, bubbles: true }));
			campo.value += caracter;
		}
		document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));

		expect(onScan).toHaveBeenCalledWith(CODIGO);
		expect(campo.value).toBe('1500');
	});

	it('ignora la tecla si otro handler ya la procesó', () => {
		const onScan = vi.fn();
		escuchar(onScan);

		const evento = new KeyboardEvent('keydown', {
			key: '7',
			bubbles: true,
			cancelable: true
		});
		evento.preventDefault();
		document.dispatchEvent(evento);
		document.dispatchEvent(
			new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })
		);

		expect(onScan).not.toHaveBeenCalled();
	});
});
