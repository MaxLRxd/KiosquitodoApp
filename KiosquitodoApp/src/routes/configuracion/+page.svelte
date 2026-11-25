<script lang="ts">
	import { onMount } from 'svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import {
		crearBackupManual,
		eliminarPin,
		eliminarTokenMp,
		establecerPin,
		guardarConfiguracion,
		guardarTokenMp,
		obtenerConfiguracion,
		obtenerRutas,
		pinConfigurado,
		seleccionarCarpetaBackups,
		tokenMpConfigurado
	} from '$lib/api';
	import { appState, notificar } from '$lib/stores/app.svelte';
	import type { ConfiguracionApp, RutasApp } from '$lib/types';

	const enTauri = isTauri();

	let rutas = $state<RutasApp | null>(null);
	let configuracion = $state<ConfiguracionApp | null>(null);
	let ultimoBackup = $state<string | null>(null);
	let creando = $state(false);
	let guardando = $state(false);
	let tienePin = $state(false);
	let pinNuevo = $state('');
	let pinConfirmar = $state('');
	let guardandoPin = $state(false);
	let tieneToken = $state(false);
	let tokenMp = $state('');
	let guardandoToken = $state(false);

	onMount(() => {
		void cargarDatos();
	});

	async function cargarDatos(): Promise<void> {
		try {
			const [rutasLeidas, preferencias, hayPin] = await Promise.all([
				obtenerRutas(),
				obtenerConfiguracion(),
				pinConfigurado()
			]);
			rutas = rutasLeidas;
			configuracion = preferencias;
			tienePin = hayPin;
			appState.inactividadMinutos = preferencias.inactividad_minutos;
			appState.pinConfigurado = hayPin;
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		}

		// El keyring puede fallar por razones del SO: no debe bloquear el resto
		// de la pantalla de configuración.
		try {
			tieneToken = await tokenMpConfigurado();
		} catch (e) {
			tieneToken = false;
			notificar(`No se pudo consultar el keyring: ${(e as Error).message}`, 'warn');
		}
	}

	async function guardarPreferencias(): Promise<void> {
		if (!configuracion || guardando) return;
		const datos: ConfiguracionApp = {
			sync_intervalo_minutos: Number(configuracion.sync_intervalo_minutos),
			inactividad_minutos: Number(configuracion.inactividad_minutos),
			sonidos_activos: configuracion.sonidos_activos,
			backup_dir: configuracion.backup_dir
		};
		if (!Number.isFinite(datos.sync_intervalo_minutos) || !Number.isFinite(datos.inactividad_minutos)) {
			notificar('Ingresá minutos válidos', 'error');
			return;
		}
		guardando = true;
		try {
			configuracion = await guardarConfiguracion(datos);
			appState.inactividadMinutos = configuracion.inactividad_minutos;
			notificar('Preferencias guardadas', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			guardando = false;
		}
	}

	async function configurarPin(): Promise<void> {
		if (guardandoPin || !pinNuevo) return;
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
			notificar(tienePin ? 'PIN configurado' : 'PIN actualizado', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
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
			notificar('PIN quitado. El bloqueo quedó desactivado', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
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
				notificar('Carpeta elegida. Guardá las preferencias para aplicarla', 'info');
			}
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		}
	}

	async function guardarToken(): Promise<void> {
		if (guardandoToken || !tokenMp.trim()) return;
		guardandoToken = true;
		try {
			tieneToken = await guardarTokenMp(tokenMp);
			tokenMp = '';
			notificar('Token guardado en el keyring de Windows', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
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
			notificar('Token eliminado del keyring', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			guardandoToken = false;
		}
	}

	async function crearBackup(): Promise<void> {
		if (creando) return;
		creando = true;
		try {
			ultimoBackup = await crearBackupManual();
			notificar('Backup manual creado', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			creando = false;
		}
	}
</script>

<div class="p-6">
	<div>
		<h1 class="text-2xl font-bold">Configuración</h1>
		<p class="text-sm text-slate-500">
			Sistema, backups y preferencias de operación. Los registros y backups se comprueban acá.
		</p>
	</div>

	<section class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
		<h2 class="text-lg font-bold">Sistema y registros</h2>
		<p class="text-sm text-slate-500">
			Rutas resueltas por el backend y creación manual de backup. Cada backup manual queda registrado
			en el log del día.
		</p>

		{#if !enTauri}
			<p class="mt-3 rounded-lg bg-amber-50 px-3 py-2 text-xs text-amber-800">
				Modo navegador: las rutas y el backup son simulados por el mock; en la app Tauri se usan
				rutas y archivos reales.
			</p>
		{/if}

		{#if rutas}
			<dl class="mt-4 grid gap-4 text-sm sm:grid-cols-2">
				<div>
					<dt class="text-xs font-medium uppercase tracking-wide text-slate-500">
						Directorio de logs
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.log_dir}</dd>
				</div>
				<div>
					<dt class="text-xs font-medium uppercase tracking-wide text-slate-500">
						Directorio de backups
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.backup_dir}</dd>
				</div>
				<div>
					<dt class="text-xs font-medium uppercase tracking-wide text-slate-500">
						Directorio de datos
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.data_dir}</dd>
				</div>
				<div>
					<dt class="text-xs font-medium uppercase tracking-wide text-slate-500">
						Base de datos
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.db_path}</dd>
				</div>
			</dl>
		{:else}
			<p class="mt-4 text-sm text-slate-500">Cargando rutas…</p>
		{/if}

		<div class="mt-5 flex flex-wrap items-center gap-3">
			<button
				onclick={crearBackup}
				disabled={creando || !rutas}
				class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{creando ? 'Creando…' : 'Crear backup manual'}
			</button>
			{#if ultimoBackup}
				<span class="text-xs text-slate-500">
					Último backup: <span class="font-mono">{ultimoBackup}</span>
				</span>
			{/if}
		</div>

		<p class="mt-3 text-xs text-slate-500">
			Los eventos se escriben en <span class="font-mono">kiosco_YYYY-MM-DD.log</span> dentro del
			directorio de logs.
		</p>
	</section>

	<section class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
		<h2 class="text-lg font-bold">Preferencias</h2>
		<p class="text-sm text-slate-500">
			Ajustes guardados en la base de datos. Las claves ausentes toman su valor por defecto.
		</p>

		{#if configuracion}
			<div class="mt-4 grid gap-4 sm:grid-cols-2">
				<label class="flex flex-col gap-1 text-sm font-medium text-slate-700">
					Intervalo de sync (minutos)
					<input
						type="number"
						min="1"
						bind:value={configuracion.sync_intervalo_minutos}
						class="rounded-md border border-slate-300 px-3 py-1.5"
					/>
				</label>
				<label class="flex flex-col gap-1 text-sm font-medium text-slate-700">
					Inactividad (minutos)
					<input
						type="number"
						min="0"
						bind:value={configuracion.inactividad_minutos}
						class="rounded-md border border-slate-300 px-3 py-1.5"
					/>
					<span class="text-xs font-normal text-slate-500">0 = bloqueo desactivado</span>
				</label>
				<label class="flex items-center gap-2 text-sm font-medium text-slate-700">
					<input
						type="checkbox"
						bind:checked={configuracion.sonidos_activos}
						class="h-4 w-4 rounded border-slate-300"
					/>
					Sonidos activos
				</label>
				<label class="flex flex-col gap-1 text-sm font-medium text-slate-700 sm:col-span-2">
					Carpeta de backups
					<input
						type="text"
						bind:value={configuracion.backup_dir}
						class="rounded-md border border-slate-300 px-3 py-1.5 font-mono text-xs"
					/>
					<span class="text-xs font-normal text-slate-500">
						La usan el backup manual y la limpieza de copias viejas.
					</span>
				</label>
			</div>

			<div class="mt-5 flex flex-wrap items-center gap-3">
				<button
					onclick={() => void seleccionarCarpeta()}
					disabled={guardando}
					class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
				>
					Seleccionar carpeta…
				</button>
				<button
					onclick={guardarPreferencias}
					disabled={guardando}
					class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{guardando ? 'Guardando…' : 'Guardar preferencias'}
				</button>
				<button
					onclick={() => void cargarDatos()}
					disabled={guardando}
					class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
				>
					Recargar
				</button>
			</div>
		{:else}
			<p class="mt-4 text-sm text-slate-500">Cargando preferencias…</p>
		{/if}
	</section>

	<section class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
		<h2 class="text-lg font-bold">Acceso y bloqueo</h2>
		<p class="text-sm text-slate-500">
			El PIN se guarda como hash SHA-256 con salt, nunca en claro. Sin PIN configurado no hay
			bloqueo por inactividad.
		</p>

		<p class="mt-4 text-sm">
			Estado:
			<span class="font-semibold {tienePin ? 'text-green-700' : 'text-slate-600'}">
				{tienePin ? 'PIN configurado' : 'Sin PIN (bloqueo desactivado)'}
			</span>
		</p>

		<div class="mt-4 grid gap-4 sm:grid-cols-2">
			<label class="flex flex-col gap-1 text-sm font-medium text-slate-700">
				Nuevo PIN (4-6 dígitos)
				<input
					type="password"
					inputmode="numeric"
					autocomplete="new-password"
					maxlength="6"
					bind:value={pinNuevo}
					class="rounded-md border border-slate-300 px-3 py-1.5 tracking-widest"
				/>
			</label>
			<label class="flex flex-col gap-1 text-sm font-medium text-slate-700">
				Confirmar PIN
				<input
					type="password"
					inputmode="numeric"
					autocomplete="new-password"
					maxlength="6"
					bind:value={pinConfirmar}
					class="rounded-md border border-slate-300 px-3 py-1.5 tracking-widest"
				/>
			</label>
		</div>

		<div class="mt-5 flex flex-wrap items-center gap-3">
			<button
				onclick={() => void configurarPin()}
				disabled={guardandoPin || !pinNuevo}
				class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{guardandoPin ? 'Guardando…' : tienePin ? 'Cambiar PIN' : 'Configurar PIN'}
			</button>
			{#if tienePin}
				<button
					onclick={() => void quitarPin()}
					disabled={guardandoPin}
					class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
				>
					Quitar PIN
				</button>
				<button
					onclick={() => (appState.bloqueado = true)}
					class="rounded-lg border border-amber-300 px-4 py-2 text-sm font-medium text-amber-800 hover:bg-amber-50"
				>
					Probar bloqueo
				</button>
			{/if}
		</div>

		<p class="mt-3 text-xs text-slate-500">
			El bloqueo salta después de {configuracion?.inactividad_minutos ?? 5} minutos sin
			interacción. Ajustá ese valor en Preferencias (0 = desactivado).
		</p>
	</section>

	<section class="mt-6 rounded-xl border border-slate-200 bg-white p-5">
		<h2 class="text-lg font-bold">Mercado Pago</h2>
		<p class="text-sm text-slate-500">
			Access Token guardado en el keyring cifrado de Windows. El backend solo informa si existe;
			nunca lo devuelve ni lo escribe en el log. La sincronización de pagos sigue diferida.
		</p>

		{#if !enTauri}
			<p class="mt-3 rounded-lg bg-amber-50 px-3 py-2 text-xs text-amber-800">
				Modo navegador: el token se guarda solo en memoria (mock).
			</p>
		{/if}

		<p class="mt-4 text-sm">
			Estado:
			<span class="font-semibold {tieneToken ? 'text-green-700' : 'text-slate-600'}">
				{tieneToken ? 'Token configurado' : 'Sin token'}
			</span>
		</p>

		<label class="mt-4 flex flex-col gap-1 text-sm font-medium text-slate-700">
			Access Token
			<input
				type="password"
				autocomplete="off"
				bind:value={tokenMp}
				placeholder="APP_USR-… o TEST-…"
				class="rounded-md border border-slate-300 px-3 py-1.5"
			/>
		</label>

		<div class="mt-5 flex flex-wrap items-center gap-3">
			<button
				onclick={() => void guardarToken()}
				disabled={guardandoToken || !tokenMp.trim()}
				class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{guardandoToken ? 'Guardando…' : tieneToken ? 'Reemplazar token' : 'Guardar token'}
			</button>
			{#if tieneToken}
				<button
					onclick={() => void quitarToken()}
					disabled={guardandoToken}
					class="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
				>
					Quitar token
				</button>
			{/if}
		</div>
	</section>
</div>
