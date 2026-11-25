<script lang="ts">
	import { onMount } from 'svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import { crearBackupManual, obtenerRutas } from '$lib/api';
	import { notificar } from '$lib/stores/app.svelte';
	import type { RutasApp } from '$lib/types';

	const enTauri = isTauri();

	let rutas = $state<RutasApp | null>(null);
	let ultimoBackup = $state<string | null>(null);
	let creando = $state(false);

	onMount(() => {
		void cargarRutas();
	});

	async function cargarRutas(): Promise<void> {
		try {
			rutas = await obtenerRutas();
		} catch (e) {
			notificar((e as Error).message, 'error', true);
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
</div>
