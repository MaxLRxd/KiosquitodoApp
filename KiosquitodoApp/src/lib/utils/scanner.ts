export type CallbackEscaneo = (codigo: string) => void;

const TIMEOUT_MS = 100;
const LONGITUD_MINIMA = 8;
const TECLAS_CONTROL = ['Shift', 'Control', 'Alt', 'Tab', 'CapsLock', 'Meta'];

/** EAN-8, UPC-A (12) y EAN-13: los formatos que soporta el lector USB 1D. */
const PATRON_CODIGO = /^(?:\d{8}|\d{12}|\d{13})$/;

export function esCodigoBarras(valor: string): boolean {
	return PATRON_CODIGO.test(valor.trim());
}

type CampoTexto = HTMLInputElement | HTMLTextAreaElement;

const TIPOS_NO_TEXTO = [
	'button',
	'checkbox',
	'color',
	'file',
	'hidden',
	'image',
	'radio',
	'range',
	'reset',
	'submit'
];

function esCampoTexto(elemento: Element | null): elemento is CampoTexto {
	if (elemento instanceof HTMLTextAreaElement) return true;
	if (!(elemento instanceof HTMLInputElement)) return false;
	return !TIPOS_NO_TEXTO.includes(elemento.type);
}

/**
 * Discrimina input del escáner HID (emula teclado) del tipeo manual.
 * El escáner entrega el código completo en <50 ms terminado en Enter;
 * un humano nunca alcanza esa velocidad, por eso el timer de 100 ms.
 *
 * Si el escaneo cae sobre un campo de texto distinto del buscador, el valor
 * previo se restaura al terminar: el código nunca queda pegado en el input.
 */
export function crearListenerEscaneo(onScan: CallbackEscaneo): () => void {
	let buffer = '';
	let timer: ReturnType<typeof setTimeout> | null = null;
	let origen: { campo: CampoTexto; valor: string } | null = null;

	function limpiar(): void {
		if (timer) clearTimeout(timer);
		timer = null;
		buffer = '';
		origen = null;
	}

	function restaurarOrigen(): void {
		if (!origen) return;
		const { campo, valor } = origen;
		origen = null;
		if (!campo.isConnected || campo.value === valor) return;
		campo.value = valor;
		campo.dispatchEvent(new Event('input', { bubbles: true }));
	}

	function onKeyDown(evento: KeyboardEvent) {
		// Otro handler (por ejemplo el Enter del buscador) ya procesó la tecla.
		if (evento.defaultPrevented) {
			limpiar();
			return;
		}

		if (TECLAS_CONTROL.includes(evento.key)) return;
		if (evento.ctrlKey || evento.metaKey || evento.altKey) return;

		if (evento.key === 'Enter') {
			if (buffer.length >= LONGITUD_MINIMA) {
				const codigo = buffer.trim();
				restaurarOrigen();
				limpiar();
				onScan(codigo);
				evento.preventDefault();
			} else {
				limpiar();
			}
			return;
		}

		if (evento.key.length !== 1) return;

		const activo = document.activeElement;
		if (buffer === '' && esCampoTexto(activo)) {
			origen = { campo: activo, valor: activo.value };
		}

		// Sin un campo enfocado no hay dónde escribir: se evita el quick-find
		// del navegador y el scroll por barra espaciadora.
		if (!esCampoTexto(activo) && /[a-z0-9]/i.test(evento.key)) {
			evento.preventDefault();
		}

		buffer += evento.key;
		if (timer) clearTimeout(timer);
		timer = setTimeout(() => {
			buffer = '';
			origen = null;
			timer = null;
		}, TIMEOUT_MS);
	}

	document.addEventListener('keydown', onKeyDown);
	return () => {
		limpiar();
		document.removeEventListener('keydown', onKeyDown);
	};
}
