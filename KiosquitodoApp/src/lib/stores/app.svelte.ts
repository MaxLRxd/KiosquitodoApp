export type TipoNotificacion = 'info' | 'success' | 'warn' | 'error';

export interface Notificacion {
	id: number;
	mensaje: string;
	tipo: TipoNotificacion;
	critico: boolean;
}

export const appState = $state({
	toasts: [] as Notificacion[],
	conectado: true,
	sincronizando: false,
	ultimoSyncExitoso: null as boolean | null
});

let proximoId = 1;

export function notificar(mensaje: string, tipo: TipoNotificacion = 'info', critico = false): void {
	const id = proximoId++;
	appState.toasts = [...appState.toasts, { id, mensaje, tipo, critico }];
	if (!critico) {
		setTimeout(() => cerrar(id), 3500);
	}
}

export function cerrar(id: number): void {
	appState.toasts = appState.toasts.filter((t) => t.id !== id);
}