export type TipoNotificacion = 'info' | 'success' | 'warn' | 'error';
export type ZonaNotificacion = 'abajo-derecha' | 'arriba-izquierda';

export interface Notificacion {
	id: number;
	mensaje: string;
	tipo: TipoNotificacion;
	critico: boolean;
	zona: ZonaNotificacion;
}

/** Normaliza cualquier rechazo (Error, string, objeto con `message`) a texto visible. */
export function mensajeError(error: unknown): string {
	if (error instanceof Error && error.message.trim()) return error.message;
	if (typeof error === 'string' && error.trim()) return error;
	if (error && typeof error === 'object' && 'message' in error) {
		const mensaje = (error as { message: unknown }).message;
		if (typeof mensaje === 'string' && mensaje.trim()) return mensaje;
	}
	return 'Ocurrió un problema. Intentá de nuevo.';
}

export const appState = $state({
	toasts: [] as Notificacion[],
	conectado: true,
	sincronizando: false,
	ultimoSyncExitoso: null as boolean | null,
	// Estado de bloqueo: lo carga y lo gestiona +layout.svelte.
	pinConfigurado: false,
	inactividadMinutos: 5,
	bloqueado: false
});

let proximoId = 1;

export function notificar(
	mensaje: unknown,
	tipo: TipoNotificacion = 'info',
	critico = false,
	zona: ZonaNotificacion = 'abajo-derecha'
): void {
	const id = proximoId++;
	appState.toasts = [...appState.toasts, { id, mensaje: mensajeError(mensaje), tipo, critico, zona }];
	if (!critico) {
		setTimeout(() => cerrar(id), zona === 'arriba-izquierda' ? 1800 : 3500);
	}
}

export function cerrar(id: number): void {
	appState.toasts = appState.toasts.filter((t) => t.id !== id);
}