<script lang="ts">
	import { onMount } from 'svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import { save } from '@tauri-apps/plugin-dialog';
	import { writeTextFile } from '@tauri-apps/plugin-fs';
	import { cerrarCaja, exportarCierresCsv, listarCierres, resumenDiario } from '$lib/api';
	import { notificar } from '$lib/stores/app.svelte';
	import { reproducirSonido } from '$lib/utils/sonido';
	import { formatearPesos } from '$lib/utils/moneda';
	import type { CierrePersistido, ResumenCierre } from '$lib/types';

	function hoy(): string {
		const ahora = new Date();
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${ahora.getFullYear()}-${pad(ahora.getMonth() + 1)}-${pad(ahora.getDate())}`;
	}

	let fecha = $state(hoy());
	let resumen = $state<ResumenCierre | null>(null);
	let cierres = $state<CierrePersistido[]>([]);
	let cargandoResumen = $state(false);
	let cerrando = $state(false);
	let rangoDesde = $state('');
	let rangoHasta = $state('');

	async function cargarResumen(): Promise<void> {
		cargandoResumen = true;
		try {
			resumen = await resumenDiario(fecha);
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			cargandoResumen = false;
		}
	}

	async function cargarCierres(): Promise<void> {
		try {
			cierres = await listarCierres();
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		}
	}

	async function cerrarCajaDia(): Promise<void> {
		if (!resumen || cerrando) return;
		if (!window.confirm(`¿Confirmas el cierre de caja del ${fecha}?\n\nVentas: ${resumen.cantidad_ventas}\nTotal: ${formatearPesos(resumen.total_ventas)}`)) {
			return;
		}
		cerrando = true;
		try {
			await cerrarCaja(fecha);
			reproducirSonido('venta_ok');
			notificar(`Cierre del ${fecha} registrado (${resumen.cantidad_ventas} ventas)`, 'success');
			await Promise.all([cargarResumen(), cargarCierres()]);
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			cerrando = false;
		}
	}

	async function exportarCsv(): Promise<void> {
		try {
			const csv = await exportarCierresCsv(rangoDesde.trim(), rangoHasta.trim());
			const nombre = `cierres_${rangoDesde || 'inicio'}_${rangoHasta || 'hoy'}.csv`;

			if (isTauri()) {
				const ruta = await save({
					defaultPath: nombre,
					filters: [{ name: 'CSV', extensions: ['csv'] }]
				});
				if (!ruta) return; // usuario canceló el diálogo
				await writeTextFile(ruta, '\uFEFF' + csv);
			} else {
				const blob = new Blob(['\uFEFF' + csv], { type: 'text/csv;charset=utf-8' });
				const url = URL.createObjectURL(blob);
				const enlace = document.createElement('a');
				enlace.href = url;
				enlace.download = nombre;
				enlace.click();
				URL.revokeObjectURL(url);
			}
			notificar('Historial de cierres exportado a CSV', 'success');
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		}
	}

	function seleccionarCierre(c: CierrePersistido): void {
		fecha = c.fecha;
	}

	$effect(() => {
		// Recarga el resumen al montar y cada vez que cambia la fecha.
		fecha;
		void cargarResumen();
	});

	onMount(() => {
		void cargarCierres();
	});
</script>

<div class="page">
	<div class="page-header">
		<div>
			<p class="eyebrow">Operación diaria</p>
			<h1 class="page-title mt-1">Cierre de caja</h1>
			<p class="page-subtitle">
				Resumen diario con totales, margen bruto estimado y delta de conciliación MP. El cierre no
				bloquea el sistema: puede ejecutarse en cualquier momento.
			</p>
		</div>
		<label class="flex items-center gap-2 text-xs font-bold uppercase tracking-wide text-slate-500">
			Fecha
			<input type="date" bind:value={fecha} class="input-compact" />
		</label>
	</div>

	{#if cargandoResumen}
		<div class="card mt-5 flex items-center gap-2 p-6 text-sm text-slate-500">
			<span class="h-2 w-2 animate-pulse-soft rounded-full bg-indigo-500"></span>
			Calculando resumen…
		</div>
	{:else if resumen}
		<div class="mt-5 grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-6">
			<div class="stat-card">
				<p class="stat-label">Ventas</p>
				<p class="stat-value">{resumen.cantidad_ventas}</p>
				<p class="stat-hint">Operaciones del día</p>
			</div>
			<div class="stat-card">
				<p class="stat-label">Total</p>
				<p class="stat-value">{formatearPesos(resumen.total_ventas)}</p>
				<p class="stat-hint">Facturación bruta</p>
			</div>
			<div class="stat-card">
				<p class="stat-label">Efectivo</p>
				<p class="stat-value text-emerald-700">{formatearPesos(resumen.total_efectivo)}</p>
				<p class="stat-hint">Cobrado en caja</p>
			</div>
			<div class="stat-card">
				<p class="stat-label">Mercado Pago</p>
				<p class="stat-value text-sky-700">{formatearPesos(resumen.total_mp)}</p>
				<p class="stat-hint">Cobrado por MP</p>
			</div>
			<div class="stat-card">
				<p class="stat-label">Margen bruto</p>
				<p class="stat-value">{formatearPesos(resumen.margen_bruto)}</p>
				<p class="stat-hint">Estimado del día</p>
			</div>
			<div class="stat-card">
				<p class="stat-label">Δ Mercado Pago</p>
				<p
					class="stat-value {resumen.delta_mp > 0
						? 'text-emerald-700'
						: resumen.delta_mp < 0
							? 'text-rose-600'
							: 'text-slate-900'}"
				>
					{resumen.delta_mp > 0 ? '+' : ''}{formatearPesos(resumen.delta_mp)}
				</p>
				<p class="stat-hint">API vs registrado</p>
			</div>
		</div>

		<div class="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-slate-200 bg-white px-4 py-3">
			<p class="max-w-3xl text-xs leading-relaxed text-slate-500">
				Δ MP = pagos acreditados por la API − ventas MP registradas. Con la integración de Mercado
				Pago diferida, todo lo cobrado por MP figura aún como no acreditado.
			</p>
			<div class="flex items-center gap-3">
				{#if resumen.cantidad_ventas === 0}
					<span class="text-xs text-slate-400">Sin ventas en la fecha seleccionada.</span>
				{/if}
				<button
					onclick={cerrarCajaDia}
					disabled={cerrando || resumen.cantidad_ventas === 0}
					class="btn-primary"
				>
					{cerrando ? 'Registrando…' : 'Cerrar caja del día'}
				</button>
			</div>
		</div>
	{/if}

	<section class="mt-8">
		<div class="flex flex-wrap items-end justify-between gap-3">
			<div>
				<h2 class="section-title">Historial de cierres</h2>
				<p class="section-subtitle">Hacé clic en una fila para ver el resumen de esa fecha.</p>
			</div>
			<span class="badge-neutral">{cierres.length} cierre(s)</span>
		</div>

		{#if cierres.length === 0}
			<div class="empty-state mt-4">
				<p class="text-sm font-semibold text-slate-700">Todavía no hay cierres registrados</p>
				<p class="mt-1 text-xs text-slate-500">
					Cuando cierres la caja del día, el resumen queda guardado en este historial.
				</p>
			</div>
		{:else}
			<div class="table-shell mt-4 overflow-x-auto">
				<table class="table">
					<thead>
						<tr>
							<th>Fecha</th>
							<th class="text-right">Ventas</th>
							<th class="text-right">Efectivo</th>
							<th class="text-right">Mercado Pago</th>
							<th class="text-right">Total</th>
							<th class="text-right">Margen</th>
							<th class="text-right">Δ MP</th>
						</tr>
					</thead>
					<tbody>
						{#each cierres as c (c.id)}
							<tr
								class="cursor-pointer {fecha === c.fecha ? 'bg-indigo-50/70' : ''}"
								onclick={() => seleccionarCierre(c)}
							>
								<td class="font-semibold text-slate-800">{c.fecha}</td>
								<td class="text-right tabular-nums">{c.resumen.cantidad_ventas}</td>
								<td class="text-right tabular-nums text-emerald-700">
									{formatearPesos(c.resumen.total_efectivo)}
								</td>
								<td class="text-right tabular-nums text-sky-700">
									{formatearPesos(c.resumen.total_mp)}
								</td>
								<td class="text-right font-semibold tabular-nums">
									{formatearPesos(c.resumen.total_ventas)}
								</td>
								<td class="text-right tabular-nums">{formatearPesos(c.resumen.margen_bruto)}</td>
								<td
									class="text-right tabular-nums {c.resumen.delta_mp > 0
										? 'text-emerald-700'
										: c.resumen.delta_mp < 0
											? 'text-rose-600'
											: 'text-slate-600'}"
								>
									{c.resumen.delta_mp > 0 ? '+' : ''}{formatearPesos(c.resumen.delta_mp)}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>

	<section class="card card-pad mt-8">
		<h2 class="section-title">Exportar histórico</h2>
		<p class="section-subtitle">
			Rango de fechas (vacío = sin límite). Montos en centavos, primera fila con encabezado.
		</p>
		<div class="mt-4 flex flex-wrap items-end gap-3">
			<label class="label">
				Desde
				<input type="date" bind:value={rangoDesde} class="input-compact" />
			</label>
			<label class="label">
				Hasta
				<input type="date" bind:value={rangoHasta} class="input-compact" />
			</label>
			<button onclick={exportarCsv} class="btn-secondary">Exportar CSV</button>
		</div>
	</section>
</div>