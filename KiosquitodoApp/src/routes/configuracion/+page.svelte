<script lang="ts">
	import { onMount } from 'svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import {
		eliminarPin,
		eliminarTokenMp,
		establecerPin,
		guardarConfiguracion,
		guardarTokenMp,
		obtenerConfiguracion,
		pinConfigurado,
		seleccionarCarpetaBackups,
		tokenMpConfigurado
	} from '$lib/api';
	import { appState, mensajeError, notificar } from '$lib/stores/app.svelte';
	import {
		actualizaciones,
		cargarEstadoActualizaciones,
		instalarActualizacionDisponible,
		verificarActualizaciones
	} from '$lib/stores/actualizaciones.svelte';
	import type { ConfiguracionApp } from '$lib/types';

	const enTauri = isTauri();

	let configuracion = $state<ConfiguracionApp | null>(null);
	let guardando = $state(false);
	let tienePin = $state(false);
	let pinNuevo = $state('');
	let pinConfirmar = $state('');
	let guardandoPin = $state(false);
	let tieneToken = $state(false);
	let tokenMp = $state('');
	let guardandoToken = $state(false);

	const porcentajeActualizacion = $derived(
		actualizaciones.total && actualizaciones.total > 0
			? Math.min(100, Math.round((actualizaciones.descargado / actualizaciones.total) * 100))
			: null
	);

	function nombreCarpeta(ruta: string): string {
		return ruta.split(/[\\/]/).filter(Boolean).pop() ?? ruta;
	}

	onMount(() => {
		void cargarDatos();
	});

	async function cargarDatos(): Promise<void> {
		try {
			const [preferencias, hayPin] = await Promise.all([
				obtenerConfiguracion(),
				pinConfigurado()
			]);
			configuracion = preferencias;
			tienePin = hayPin;
			appState.inactividadMinutos = preferencias.inactividad_minutos;
			appState.pinConfigurado = hayPin;
			if (!hayPin) appState.bloqueado = false;
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		}

		try {
			tieneToken = await tokenMpConfigurado();
		} catch {
			tieneToken = false;
			notificar('No se pudo verificar el estado de Mercado Pago', 'warn');
		}

		await cargarEstadoActualizaciones();
	}

	async function guardarPreferencias(): Promise<void> {
		if (!configuracion || guardando) return;
		const datos: ConfiguracionApp = {
			sync_intervalo_minutos: Number(configuracion.sync_intervalo_minutos),
			inactividad_minutos: Number(configuracion.inactividad_minutos),
			sonidos_activos: configuracion.sonidos_activos,
			backup_dir: configuracion.backup_dir,
			actualizaciones_url: configuracion.actualizaciones_url,
			actualizaciones_auto: configuracion.actualizaciones_auto
		};
		if (!Number.isFinite(datos.sync_intervalo_minutos) || !Number.isFinite(datos.inactividad_minutos)) {
			notificar('Ingresá minutos válidos', 'error');
			return;
		}
		guardando = true;
		try {
			configuracion = await guardarConfiguracion(datos);
			appState.inactividadMinutos = configuracion.inactividad_minutos;
			await cargarEstadoActualizaciones();
			notificar('Cambios guardados', 'success');
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		} finally {
			guardando = false;
		}
	}

	async function configurarPin(): Promise<void> {
		if (guardandoPin || !pinNuevo) return;
		if (!/^\d{4,6}$/.test(pinNuevo)) {
			notificar('El PIN debe tener entre 4 y 6 números', 'warn');
			return;
		}
		if (pinNuevo !== pinConfirmar) {
			notificar('Los PIN no coinciden', 'error');
			return;
		}
		guardandoPin = true;
		try {
			tienePin = await establecerPin(pinNuevo);
			appState.pinConfigurado = tienePin;
			pinNuevo = '';
			pinConfirmar = '';
			if (tienePin) appState.bloqueado = true;
			notificar(
				tienePin ? 'PIN configurado. Se pedirá al abrir el kiosco.' : 'PIN actualizado',
				'success'
			);
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		} finally {
			guardandoPin = false;
		}
	}

	async function quitarPin(): Promise<void> {
		if (guardandoPin) return;
		guardandoPin = true;
		try {
			await eliminarPin();
			tienePin = false;
			appState.pinConfigurado = false;
			appState.bloqueado = false;
			notificar('PIN quitado. El kiosco ya no pedirá clave', 'success');
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		} finally {
			guardandoPin = false;
		}
	}

	async function seleccionarCarpeta(): Promise<void> {
		if (!configuracion) return;
		try {
			const carpeta = await seleccionarCarpetaBackups();
			if (carpeta) {
				configuracion.backup_dir = carpeta;
				notificar('Carpeta elegida. Guardá los cambios para aplicarla', 'info');
			}
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		}
	}

	async function guardarToken(): Promise<void> {
		if (guardandoToken || !tokenMp.trim()) return;
		guardandoToken = true;
		try {
			tieneToken = await guardarTokenMp(tokenMp.trim());
			tokenMp = '';
			notificar(
				tieneToken ? 'Credencial guardada de forma segura.' : 'No se pudo guardar la credencial',
				tieneToken ? 'success' : 'warn'
			);
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		} finally {
			guardandoToken = false;
		}
	}

	async function quitarToken(): Promise<void> {
		if (guardandoToken) return;
		guardandoToken = true;
		try {
			await eliminarTokenMp();
			tieneToken = false;
			notificar('Credencial de Mercado Pago quitada', 'success');
		} catch (e) {
			notificar(mensajeError(e), 'error', true);
		} finally {
			guardandoToken = false;
		}
	}
</script>

<div class="page">
	<div class="page-header">
		<div>
			<p class="eyebrow">Administración</p>
			<h1 class="page-title mt-1">Configuración</h1>
			<p class="page-subtitle">Preferencias, acceso y conexiones del kiosco.</p>
		</div>
		<span class="badge {enTauri ? 'badge-success' : 'badge-warn'}">
			{enTauri ? 'App de escritorio' : 'Modo navegador'}
		</span>
	</div>

	<section class="card card-pad mt-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="section-title">Preferencias</h2>
				<p class="section-subtitle">Ajustes de uso diario. Guardalos con el botón al final.</p>
			</div>
			<span class="badge-neutral">Operación</span>
		</div>

		{#if configuracion}
			<div class="mt-4 grid gap-4 sm:grid-cols-2">
				<label class="label">
					Actualización de pagos (minutos)
					<input
						type="number"
						min="1"
						bind:value={configuracion.sync_intervalo_minutos}
						class="input"
					/>
				</label>
				<label class="label">
					Bloquear después de (minutos)
					<input
						type="number"
						min="0"
						bind:value={configuracion.inactividad_minutos}
						class="input"
					/>
					<span class="text-[11px] font-medium normal-case tracking-normal text-slate-400">
						0 = no se bloquea solo. Si hay PIN, se pide igual al abrir.
					</span>
				</label>
				<label
					class="flex items-center gap-2 rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3 text-sm font-semibold text-slate-700"
				>
					<input
						type="checkbox"
						bind:checked={configuracion.sonidos_activos}
						class="h-4 w-4 rounded border-slate-300 accent-indigo-600"
					/>
					Sonidos activos
				</label>
				<div
					class="flex flex-col justify-between gap-2 rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3 sm:col-span-2"
				>
					<div>
						<p class="text-sm font-semibold text-slate-700">Respaldo de datos</p>
						<p class="mt-1 text-[11px] text-slate-500">
							Carpeta elegida: {nombreCarpeta(configuracion.backup_dir)}
						</p>
					</div>
					<button
						onclick={() => void seleccionarCarpeta()}
						disabled={guardando}
						class="btn-secondary self-start"
					>
						Elegir carpeta…
					</button>
				</div>
			</div>

			<div class="mt-5 flex flex-wrap items-center gap-3 border-t border-slate-100 pt-4">
				<button onclick={guardarPreferencias} disabled={guardando} class="btn-primary">
					{guardando ? 'Guardando…' : 'Guardar cambios'}
				</button>
				<button onclick={() => void cargarDatos()} disabled={guardando} class="btn-ghost">
					Recargar
				</button>
			</div>
		{:else}
			<div class="mt-4 flex items-center gap-2 text-sm text-slate-500">
				<span class="h-2 w-2 animate-pulse-soft rounded-full bg-indigo-500"></span>
				Cargando preferencias…
			</div>
		{/if}
	</section>

	<section class="card card-pad mt-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="section-title">Acceso y bloqueo</h2>
				<p class="section-subtitle">
					Poné un PIN de 4 a 6 números para que nadie más opere el kiosco. Se pedirá al abrir y
					también después del tiempo de inactividad.
				</p>
			</div>
			<span class="badge {tienePin ? 'badge-success' : 'badge-neutral'}">
				{tienePin ? 'Con PIN' : 'Sin PIN'}
			</span>
		</div>

		<div class="mt-4 grid gap-4 sm:grid-cols-2">
			<label class="label">
				Nuevo PIN
				<input
					type="password"
					inputmode="numeric"
					autocomplete="new-password"
					maxlength="6"
					bind:value={pinNuevo}
					class="input tracking-[0.4em]"
				/>
			</label>
			<label class="label">
				Confirmar PIN
				<input
					type="password"
					inputmode="numeric"
					autocomplete="new-password"
					maxlength="6"
					bind:value={pinConfirmar}
					class="input tracking-[0.4em]"
				/>
			</label>
		</div>

		<div class="mt-5 flex flex-wrap items-center gap-3">
			<button
				onclick={() => void configurarPin()}
				disabled={guardandoPin || !pinNuevo}
				class="btn-primary"
			>
				{guardandoPin ? 'Guardando…' : tienePin ? 'Cambiar PIN' : 'Configurar PIN'}
			</button>
			{#if tienePin}
				<button onclick={() => void quitarPin()} disabled={guardandoPin} class="btn-secondary">
					Quitar PIN
				</button>
				<button
					onclick={() => (appState.bloqueado = true)}
					class="btn-sm rounded-lg border border-amber-200 bg-amber-50 font-semibold text-amber-800 transition hover:bg-amber-100"
				>
					Probar bloqueo
				</button>
			{/if}
		</div>

		<p class="mt-3 text-xs leading-relaxed text-slate-500">
			El tiempo de inactividad se ajusta en Preferencias. Si lo ponés en 0, el PIN solo se pide al
			abrir el kiosco.
		</p>
	</section>

	<section class="card card-pad mt-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="section-title">Actualizaciones</h2>
				<p class="section-subtitle">Buscá mejoras y aplicá la última versión cuando quieras.</p>
			</div>
			<span class="badge-info">
				{actualizaciones.estado ? `v${actualizaciones.estado.version_actual}` : 'Cargando…'}
			</span>
		</div>

		{#if !enTauri}
			<div class="alert-warn mt-4">
				Modo navegador: la búsqueda y la instalación se simulan. Probá las actualizaciones en la app
				de escritorio.
			</div>
		{/if}

		{#if configuracion}
			<div class="mt-5 flex flex-wrap items-center gap-3 border-t border-slate-100 pt-4">
				<button
					onclick={() => void verificarActualizaciones(true)}
					disabled={actualizaciones.buscando ||
						actualizaciones.instalando ||
						!actualizaciones.estado?.endpoint_configurado}
					class="btn-secondary"
				>
					{actualizaciones.buscando ? 'Buscando…' : 'Buscar actualizaciones'}
				</button>
				{#if actualizaciones.disponible}
					<button
						onclick={() => void instalarActualizacionDisponible()}
						disabled={actualizaciones.instalando}
						class="btn-primary"
					>
						{actualizaciones.instalando
							? 'Instalando…'
							: `Instalar v${actualizaciones.disponible.version_nueva}`}
					</button>
				{/if}
				{#if actualizaciones.resultado}
					<span class="text-xs text-slate-600">{actualizaciones.resultado}</span>
				{/if}
			</div>

			{#if actualizaciones.error}
				<p class="alert-error mt-3" role="alert">{actualizaciones.error}</p>
			{/if}

			{#if actualizaciones.disponible && !actualizaciones.instalando}
				<div class="alert-info mt-3">
					Nueva versión <span class="font-semibold">{actualizaciones.disponible.version_nueva}</span>
					disponible. Instalada: {actualizaciones.disponible.version_actual}.
				</div>
			{/if}

			{#if actualizaciones.instalando}
				<div class="mt-4">
					<div class="flex items-center justify-between text-xs font-semibold text-slate-600">
						<span>Descargando e instalando…</span>
						<span>{porcentajeActualizacion !== null ? `${porcentajeActualizacion}%` : '…'}</span>
					</div>
					<div class="mt-2 h-2 overflow-hidden rounded-full bg-slate-200">
						<div
							class="h-full rounded-full bg-indigo-600 transition-all"
							style="width: {porcentajeActualizacion ?? 5}%"
						></div>
					</div>
				</div>
			{/if}
		{:else}
			<div class="mt-4 flex items-center gap-2 text-sm text-slate-500">
				<span class="h-2 w-2 animate-pulse-soft rounded-full bg-indigo-500"></span>
				Cargando preferencias…
			</div>
		{/if}
	</section>

	<section class="card card-pad mt-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="section-title">Mercado Pago</h2>
				<p class="section-subtitle">
					Ingresá la credencial para consultar los cobros. Se guarda de forma segura y nunca se
					muestra nuevamente. La sincronización de pagos estará disponible próximamente.
				</p>
			</div>
			<span class="badge {tieneToken ? 'badge-success' : 'badge-neutral'}">
				{tieneToken ? 'Credencial configurada' : 'Sin credencial'}
			</span>
		</div>

		{#if !enTauri}
			<div class="alert-warn mt-4">Modo navegador: la credencial se guarda solo en memoria.</div>
		{/if}

		<label class="label mt-4">
			Credencial de Mercado Pago
			<input
				type="password"
				autocomplete="off"
				bind:value={tokenMp}
				placeholder="Pegá acá la credencial"
				class="input text-sm"
			/>
		</label>

		<div class="mt-5 flex flex-wrap items-center gap-3 border-t border-slate-100 pt-4">
			<button
				onclick={() => void guardarToken()}
				disabled={guardandoToken || !tokenMp.trim()}
				class="btn-primary"
			>
				{guardandoToken ? 'Guardando…' : tieneToken ? 'Reemplazar credencial' : 'Guardar credencial'}
			</button>
			{#if tieneToken}
				<button onclick={() => void quitarToken()} disabled={guardandoToken} class="btn-secondary">
					Quitar credencial
				</button>
			{/if}
		</div>
	</section>
</div>
