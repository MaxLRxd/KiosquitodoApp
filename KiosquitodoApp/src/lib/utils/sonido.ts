export type TipoSonido = 'scan_ok' | 'scan_error' | 'venta_ok' | 'sistema_error';

const RUTA: Record<TipoSonido, string> = {
	scan_ok: '/sounds/scan_ok.wav',
	scan_error: '/sounds/scan_error.wav',
	venta_ok: '/sounds/venta_ok.wav',
	sistema_error: '/sounds/sistema_error.wav'
};

export function reproducirSonido(tipo: TipoSonido): void {
	try {
		void new Audio(RUTA[tipo]).play();
	} catch {
		// sin audio disponible no debe romper el flujo
	}
}