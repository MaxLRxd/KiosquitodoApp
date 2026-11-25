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
		tokenMpConfigurado,
		verificarBackupDiario
	} from '$lib/api';
	import { appState, notificar } from '$lib/stores/app.svelte';
	import {
		actualizaciones,
		cargarEstadoActualizaciones,
		instalarActualizacionDisponible,
		verificarActualizaciones
	} from '$lib/stores/actualizaciones.svelte';
	import type { ConfiguracionApp, RutasApp } from '$lib/types';

	const enTauri = isTauri();

	let rutas = $state<RutasApp | null>(null);
	let configuracion = $state<ConfiguracionApp | null>(null);
	let ultimoBackup = $state<string | null>(null);
	let creando = $state(false);
	let verificandoBackupDiario = $state(false);
	let resultadoBackupDiario = $state<string | null>(null);
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

	async function probarBackupDiario(): Promise<void> {
		if (verificandoBackupDiario) return;
		verificandoBackupDiario = true;
		try {
			const ruta = await verificarBackupDiario();
			resultadoBackupDiario = ruta
				? `Creado ahora: ${ruta}`
				: 'Ya existe el backup de hoy (el arranque lo verificó)';
			notificar('Backup diario verificado', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			verificandoBackupDiario = false;
		}
	}
</script>

<div class="page">
	<div class="page-header">
		<div>
			<p class="eyebrow">Administración</p>
			<h1 class="page-title mt-1">Configuración</h1>
			<p class="page-subtitle">
				Sistema, backups y preferencias de operación. Los registros y backups se comprueban acá.
			</p>
		</div>
		<span class="badge {enTauri ? 'badge-success' : 'badge-warn'}">
			{enTauri ? 'App de escritorio' : 'Modo navegador'}
		</span>
	</div>

	<section class="card card-pad mt-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="section-title">Sistema y registros</h2>
				<p class="section-subtitle">
					Rutas resueltas por el backend y creación manual de backup. Cada backup manual queda
					registrado en el log del día.
				</p>
			</div>
			<span class="badge-info">Diagnóstico</span>
		</div>

		{#if !enTauri}
			<div class="alert-warn mt-4">
				Modo navegador: las rutas y el backup son simulados por el mock; en la app Tauri se usan
				rutas y archivos reales.
			</div>
		{/if}

		{#if rutas}
			<dl class="mt-4 grid gap-3 text-sm sm:grid-cols-2">
				<div class="rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3">
					<dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500">
						Directorio de logs
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.log_dir}</dd>
				</div>
				<div class="rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3">
					<dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500">
						Directorio de backups
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.backup_dir}</dd>
				</div>
				<div class="rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3">
					<dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500">
						Directorio de datos
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.data_dir}</dd>
				</div>
				<div class="rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3">
					<dt class="text-[11px] font-bold uppercase tracking-wider text-slate-500">
						Base de datos
					</dt>
					<dd class="mt-1 break-all font-mono text-xs text-slate-800">{rutas.db_path}</dd>
				</div>
			</dl>
		{:else}
			<div class="mt-4 flex items-center gap-2 text-sm text-slate-500">
				<span class="h-2 w-2 animate-pulse-soft rounded-full bg-indigo-500"></span>
				Cargando rutas…
			</div>
		{/if}

		<div class="mt-5 flex flex-wrap items-center gap-3">
			<button onclick={crearBackup} disabled={creando || !rutas} class="btn-primary">
				{creando ? 'Creando…' : 'Crear backup manual'}
			</button>
			{#if ultimoBackup}
				<span class="text-xs text-slate-500">
					Último backup: <span class="font-mono text-slate-700">{ultimoBackup}</span>
				</span>
			{/if}
		</div>

		<div class="mt-4 flex flex-wrap items-center gap-3 border-t border-slate-100 pt-4">
			<button
				onclick={() => void probarBackupDiario()}
				disabled={verificandoBackupDiario || !rutas}
				class="btn-secondary"
			>
				{verificandoBackupDiario ? 'Verificando…' : 'Probar backup automático'}
			</button>
			{#if resultadoBackupDiario}
				<span class="text-xs text-slate-600">
					Resultado: <span class="font-mono text-slate-800">{resultadoBackupDiario}</span>
				</span>
			{/if}
		</div>

		<p class="mt-3 text-xs leading-relaxed text-slate-500">
			Los eventos se escriben en <span class="font-mono">kiosco_YYYY-MM-DD.log</span> dentro del
			directorio de logs. El estado y la configuración del auto-update están en la sección
			Actualizaciones.
		</p>
	</section>

	<section class="card card-pad mt-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="section-title">Preferencias</h2>
				<p class="section-subtitle">
					Ajustes guardados en la base de datos. Las claves ausentes toman su valor por defecto.
				</p>
			</div>
			<span class="badge-neutral">Operación</span>
		</div>

		{#if configuracion}
			<div class="mt-4 grid gap-4 sm:grid-cols-2">
				<label class="label">
					Intervalo de sync (minutos)
					<input
						type="number"
						min="1"
						bind:value={configuracion.sync_intervalo_minutos}
						class="input"
					/>
				</label>
				<label class="label">
					Inactividad (minutos)
					<input
						type="number"
						min="0"
						bind:value={configuracion.inactividad_minutos}
						class="input"
					/>
					<span class="text-[11px] font-medium normal-case tracking-normal text-slate-400">
						0 = bloqueo desactivado
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
				<label class="label sm:col-span-2">
					Carpeta de backups
					<input
						type="text"
						bind:value={configuracion.backup_dir}
						class="input font-mono text-xs"
					/>
					<span class="text-[11px] font-medium normal-case tracking-normal text-slate-400">
						La usan el backup manual, el backup diario y la limpieza de copias viejas.
					</span>
				</label>
			</div>

			<div class="mt-5 flex flex-wrap items-center gap-3 border-t border-slate-100 pt-4">
				<button
					onclick={() => void seleccionarCarpeta()}
					disabled={guardando}
					class="btn-secondary"
				>
					Seleccionar carpeta…
				</button>
				<button onclick={guardarPreferencias} disabled={guardando} class="btn-primary">
					{guardando ? 'Guardando…' : 'Guardar preferencias'}
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
					El PIN se guarda como hash SHA-256 con salt, nunca en claro. Sin PIN configurado no hay
					bloqueo por inactividad.
				</p>
			</div>
			<span class="badge {tienePin ? 'badge-success' : 'badge-neutral'}">
				{tienePin ? 'PIN configurado' : 'Sin PIN'}
			</span>
		</div>

		<div class="mt-4 grid gap-4 sm:grid-cols-2">
			<label class="label">
				Nuevo PIN (4-6 dígitos)
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
			El bloqueo salta después de {configuracion?.inactividad_minutos ?? 5} minutos sin interacción.
			Ajustá ese valor en Preferencias (0 = desactivado).
		</p>
	</section>

	<section class="card card-pad mt-5">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="section-title">Actualizaciones</h2>
				<p class="section-subtitle">
					La app consulta un manifiesto <span class="font-mono">latest.json</span> y valida la firma
					digital del instalador antes de aplicarlo. El endpoint se puede cambiar sin recompilar.
				</p>
			</div>
			<span class="badge-info">
				{actualizaciones.estado ? `v${actualizaciones.estado.version_actual}` : 'Cargando…'}
			</span>
		</div>

		{#if !enTauri}
			<div class="alert-warn mt-4">
				Modo navegador: la verificación y la instalación reales solo funcionan en la app de
				escritorio.
			</div>
		{/if}

		{#if configuracion}
			<div class="mt-4 grid gap-4 sm:grid-cols-2">
				<label class="label sm:col-span-2">
					URL del manifiesto de actualizaciones
					<input
						type="url"
						bind:value={configuracion.actualizaciones_url}
						placeholder="https://…/latest.json"
						class="input font-mono text-xs"
					/>
					<span class="text-[11px] font-medium normal-case tracking-normal text-slate-400">
						Vacío = auto-update deshabilitado. Por defecto apunta a GitHub Releases.
					</span>
				</label>
				<label
					class="flex items-center gap-2 rounded-xl border border-slate-200 bg-slate-50/60 px-4 py-3 text-sm font-semibold text-slate-700 sm:col-span-2"
				>
					<input
						type="checkbox"
						bind:checked={configuracion.actualizaciones_auto}
						class="h-4 w-4 rounded border-slate-300 accent-indigo-600"
					/>
					Verificar actualizaciones al iniciar
				</label>
			</div>

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

			<p class="mt-3 text-xs leading-relaxed text-slate-500">
				Los releases se firman con la clave privada local (fuera del repositorio). Si el manifiesto
				o la firma no coinciden, la actualización se rechaza y la app sigue funcionando.
			</p>
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
					Access Token guardado en el keyring cifrado de Windows. El backend solo informa si existe;
					nunca lo devuelve ni lo escribe en el log. La sincronización de pagos sigue diferida.
				</p>
			</div>
			<span class="badge {tieneToken ? 'badge-success' : 'badge-neutral'}">
				{tieneToken ? 'Token configurado' : 'Sin token'}
			</span>
		</div>

		{#if !enTauri}
			<div class="alert-warn mt-4">Modo navegador: el token se guarda solo en memoria (mock).</div>
		{/if}

		<label class="label mt-4">
			Access Token
			<input
				type="password"
				autocomplete="off"
				bind:value={tokenMp}
				placeholder="APP_USR-… o TEST-…"
				class="input font-mono text-xs"
			/>
		</label>

		<div class="mt-5 flex flex-wrap items-center gap-3 border-t border-slate-100 pt-4">
			<button
				onclick={() => void guardarToken()}
				disabled={guardandoToken || !tokenMp.trim()}
				class="btn-primary"
			>
				{guardandoToken ? 'Guardando…' : tieneToken ? 'Reemplazar token' : 'Guardar token'}
			</button>
			{#if tieneToken}
				<button onclick={() => void quitarToken()} disabled={guardandoToken} class="btn-secondary">
					Quitar token
				</button>
			{/if}
		</div>
	</section>
</div>
