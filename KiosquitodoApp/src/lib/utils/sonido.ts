export type TipoSonido = 'scan_ok' | 'scan_error' | 'venta_ok' | 'sistema_error';

// TODO fase UI: reproducir los .wav de /static/sounds vía HTML5 Audio (no-op temporal).
export function reproducirSonido(_tipo: TipoSonido): void {
	// sin implementación en esta fase
}