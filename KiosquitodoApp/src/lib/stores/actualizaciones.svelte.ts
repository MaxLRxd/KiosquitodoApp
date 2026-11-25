import { isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
	buscarActualizacion,
	estadoActualizacion,
	instalarActualizacion
} from '$lib/api';
import { notificar } from '$lib/stores/app.svelte';
import type {
	EstadoActualizacion,
	InfoActualizacion,
	ProgresoActualizacion
} from '$lib/types';

export const actualizaciones = $state({
	estado: null as EstadoActualizacion | null,
	disponible: null as InfoActualizacion | null,
	buscando: false,
	instalando: false,
	descargado: 0,
	total: null as number | null,
	error: null as string | null,
	resultado: null as string | null,
	modalVisible: false
});

let iniciado = false;
let unlisten: UnlistenFn | null = null;

/** Lee la versión instalada y la configuración del endpoint. */
export async function cargarEstadoActualizaciones(): Promise<void> {
	try {
		actualizaciones.estado = await estadoActualizacion();
	} catch (e) {
		// En el navegador el mock responde; un fallo acá no debe romper la app.
		console.warn('No se pudo leer el estado de actualizaciones:', e);
	}
}

/** Verifica si hay una versión nueva. `manual` controla avisos y modal. */
export async function verificarActualizaciones(manual = false): Promise<void> {
	if (actualizaciones.buscando || actualizaciones.instalando) return;
	actualizaciones.buscando = true;
	actualizaciones.error = null;
	actualizaciones.resultado = null;
	if (manual) actualizaciones.modalVisible = false;

	try {
		const info = await buscarActualizacion();
		if (info) {
			actualizaciones.disponible = info;
			if (!manual) actualizaciones.modalVisible = true;
			else actualizaciones.resultado = `Nueva versión disponible: ${info.version_nueva}`;
		} else {
			actualizaciones.disponible = null;
			actualizaciones.resultado = 'Ya tenés la última versión';
			if (manual) notificar('La app está actualizada', 'success');
		}
	} catch (e) {
		const mensaje = e instanceof Error ? e.message : String(e);
		actualizaciones.error = mensaje;
		if (manual) notificar(mensaje, 'error', true);
		else console.warn('No se pudo verificar actualizaciones:', mensaje);
	} finally {
		actualizaciones.buscando = false;
	}
}

/** Descarga e instala la versión disponible. En Windows la app se reinicia. */
export async function instalarActualizacionDisponible(): Promise<void> {
	if (!actualizaciones.disponible || actualizaciones.instalando) return;
	actualizaciones.instalando = true;
	actualizaciones.error = null;
	actualizaciones.descargado = 0;
	actualizaciones.total = null;

	try {
		const version = await instalarActualizacion();
		actualizaciones.disponible = null;
		actualizaciones.modalVisible = false;
		actualizaciones.resultado = `Actualización ${version} instalada. Reiniciando…`;
		notificar(`Actualización ${version} instalada`, 'success');
	} catch (e) {
		const mensaje = e instanceof Error ? e.message : String(e);
		actualizaciones.error = mensaje;
		notificar(mensaje, 'error', true);
	} finally {
		actualizaciones.instalando = false;
	}
}

/** Cierra el modal sin instalar; la versión queda disponible para más tarde. */
export function descartarActualizacion(): void {
	actualizaciones.modalVisible = false;
}

/** Arranque único: escucha progreso y, si corresponde, verifica en segundo plano. */
export async function iniciarActualizaciones(): Promise<void> {
	if (iniciado) return;
	iniciado = true;

	await cargarEstadoActualizaciones();

	if (!isTauri()) return;

	if (!unlisten) {
		unlisten = await listen<ProgresoActualizacion>('actualizacion:progreso', (evento) => {
			actualizaciones.descargado = evento.payload.descargado;
			actualizaciones.total = evento.payload.total;
		});
	}

	const estado = actualizaciones.estado;
	if (estado?.auto_verificar && estado.endpoint_configurado) {
		// Espera a que la UI termine de montar para no competir con el arranque.
		setTimeout(() => void verificarActualizaciones(false), 1500);
	}
}
