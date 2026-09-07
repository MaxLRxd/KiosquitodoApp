export type CallbackEscaneo = (codigo: string) => void;

const TIMEOUT_MS = 100;
const LONGITUD_MINIMA = 8;
const TECLAS_CONTROL = ['Shift', 'Control', 'Alt', 'Tab', 'CapsLock', 'Meta'];

/**
 * Discrimina input del escáner HID (emula teclado) del tipeo manual.
 * El escáner entrega el código completo en <50 ms terminado en Enter;
 * un humano nunca alcanza esa velocidad, por eso el timer de 100 ms.
 */
export function crearListenerEscaneo(onScan: CallbackEscaneo): () => void {
	let buffer = '';
	let timer: ReturnType<typeof setTimeout> | null = null;

	function onKeyDown(evento: KeyboardEvent) {
		if (TECLAS_CONTROL.includes(evento.key)) return;

		if (evento.key === 'Enter') {
			if (buffer.length >= LONGITUD_MINIMA) {
				if (timer) clearTimeout(timer);
				onScan(buffer.trim());
				buffer = '';
				evento.preventDefault();
			} else {
				buffer = '';
			}
			return;
		}

		if (evento.key.length === 1) {
			buffer += evento.key;
			if (timer) clearTimeout(timer);
			timer = setTimeout(() => {
				buffer = '';
			}, TIMEOUT_MS);
		}
	}

	document.addEventListener('keydown', onKeyDown);
	return () => document.removeEventListener('keydown', onKeyDown);
}