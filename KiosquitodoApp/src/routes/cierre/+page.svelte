<script lang="ts">
	import { onMount } from 'svelte';
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
			// TODO fase integración: en Tauri usar plugin-fs/plugin-dialog para guardar.
			const blob = new Blob(['\uFEFF' + csv], { type: 'text/csv;charset=utf-8' });
			const url = URL.createObjectURL(blob);
			const enlace = document.createElement('a');
			enlace.href = url;
			enlace.download = `cierres_${rangoDesde || 'inicio'}_${rangoHasta || 'hoy'}.csv`;
			enlace.click();
			URL.revokeObjectURL(url);
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

<div class="p-6">
	<div class="flex flex-wrap items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Cierre de Caja</h1>
			<p class="text-sm text-slate-500">
				Resumen diario con totales, margen bruto estimado y delta de conciliación MP. El cierre
				no bloquea el sistema: puede ejecutarse en cualquier momento.
			</p>
		</div>
		<label class="flex items-center gap-2 text-sm font-medium text-slate-700">
			Fecha
			<input type="date" bind:value={fecha} class="rounded-md border border-slate-300 px-3 py-1.5" />
		</label>
	</div>

	{#if cargandoResumen}
		<p class="mt-6 text-sm text-slate-500">Calculando resumen…</p>
	{:else if resumen}
		<div class="mt-6 grid grid-cols-2 gap-4 md:grid-cols-4 xl:grid-cols-6">
			<div class="rounded-xl border border-slate-200 bg-white p-4">
				<p class="text-xs font-medium uppercase tracking-wide text-slate-500">Ventas</p>
				<p class="mt-1 text-2xl font-bold">{resumen.cantidad_ventas}</p>
			</div>
			<div class="rounded-xl border border-slate-200 bg-white p-4">
				<p class="text-xs font-medium uppercase tracking-wide text-slate-500">Total</p>
				<p class="mt-1 text-2xl font-bold text-slate-800">{formatearPesos(resumen.total_ventas)}</p>
			</div>
			<div class="rounded-xl border border-slate-200 bg-white p-4">
				<p class="text-xs font-medium uppercase tracking-wide text-slate-500">Efectivo</p>
				<p class="mt-1 text-2xl font-bold text-emerald-700">
					{formatearPesos(resumen.total_efectivo)}
				</p>
			</div>
			<div class="rounded-xl border border-slate-200 bg-white p-4">
				<p class="text-xs font-medium uppercase tracking-wide text-slate-500">Mercado Pago</p>
				<p class="mt-1 text-2xl font-bold text-sky-700">{formatearPesos(resumen.total_mp)}</p>
			</div>
			<div class="rounded-xl border border-slate-200 bg-white p-4">
				<p class="text-xs font-medium uppercase tracking-wide text-slate-500">Margen bruto</p>
				<p class="mt-1 text-2xl font-bold text-slate-800">
					{formatearPesos(resumen.margen_bruto)}
				</p>
			</div>
			<div class="rounded-xl border border-slate-200 bg-white p-4">
				<p class="text-xs font-medium uppercase tracking-wide text-slate-500">Δ Mercado Pago</p>
				<p class="mt-1 text-2xl font-bold {resumen.delta_mp > 0 ? 'text-emerald-700' : resumen.delta_mp < 0 ? 'text-red-600' : 'text-slate-800'}">
					{resumen.delta_mp > 0 ? '+' : ''}{formatearPesos(resumen.delta_mp)}
				</p>
			</div>
		</div>

		<p class="mt-2 text-xs text-slate-500">
			Δ MP = pagos acreditados por la API − ventas MP registradas. Con la integración de Mercado
			Pago diferida, todo lo cobrado por MP figura aún como no acreditado.
		</p>

		<div class="mt-4">
			<button
				onclick={cerrarCajaDia}
				disabled={cerrando || resumen.cantidad_ventas === 0}
				class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{cerrando ? 'Registrando…' : 'Cerrar caja del día'}
			</button>
			{#if resumen.cantidad_ventas === 0}
				<span class="ml-3 text-sm text-slate-500">Sin ventas en la fecha seleccionada.</span>
			{/if}
		</div>
	{/if}

	<section class="mt-10">
		<h2 class="text-lg font-bold">Historial de cierres</h2>
		<p class="text-sm text-slate-500">Hacé clic en una fila para ver el resumen de esa fecha.</p>

		{#if cierres.length === 0}
			<p class="mt-3 text-sm text-slate-500">Todavía no hay cierres registrados.</p>
		{:else}
			<div class="mt-3 overflow-x-auto rounded-xl border border-slate-200">
				<table class="w-full text-left text-sm">
					<thead class="bg-slate-50 text-xs uppercase tracking-wide text-slate-500">
						<tr>
							<th class="px-4 py-2">Fecha</th>
							<th class="px-4 py-2 text-right">Ventas</th>
							<th class="px-4 py-2 text-right">Efectivo</th>
							<th class="px-4 py-2 text-right">Mercado Pago</th>
							<th class="px-4 py-2 text-right">Total</th>
							<th class="px-4 py-2 text-right">Margen</th>
							<th class="px-4 py-2 text-right">Δ MP</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each cierres as c (c.id)}
							<tr
								class="cursor-pointer hover:bg-slate-50 {fecha === c.fecha ? 'bg-blue-50' : ''}"
								onclick={() => seleccionarCierre(c)}
							>
								<td class="px-4 py-2 font-medium text-slate-800">{c.fecha}</td>
								<td class="px-4 py-2 text-right">{c.resumen.cantidad_ventas}</td>
								<td class="px-4 py-2 text-right text-emerald-700">
									{formatearPesos(c.resumen.total_efectivo)}
								</td>
								<td class="px-4 py-2 text-right text-sky-700">
									{formatearPesos(c.resumen.total_mp)}
								</td>
								<td class="px-4 py-2 text-right font-semibold">
									{formatearPesos(c.resumen.total_ventas)}
								</td>
								<td class="px-4 py-2 text-right">{formatearPesos(c.resumen.margen_bruto)}</td>
								<td class="px-4 py-2 text-right {c.resumen.delta_mp > 0 ? 'text-emerald-700' : c.resumen.delta_mp < 0 ? 'text-red-600' : 'text-slate-600'}">
									{c.resumen.delta_mp > 0 ? '+' : ''}{formatearPesos(c.resumen.delta_mp)}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>

	<section class="mt-10">
		<h2 class="text-lg font-bold">Exportar histórico</h2>
		<p class="text-sm text-slate-500">
			Rango de fechas (vacío = sin límite). Montos en centavos, primera fila con encabezado.
		</p>
		<div class="mt-3 flex flex-wrap items-end gap-3">
			<label class="flex flex-col gap-1 text-sm font-medium text-slate-700">
				Desde
				<input type="date" bind:value={rangoDesde} class="rounded-md border border-slate-300 px-3 py-1.5" />
			</label>
			<label class="flex flex-col gap-1 text-sm font-medium text-slate-700">
				Hasta
				<input type="date" bind:value={rangoHasta} class="rounded-md border border-slate-300 px-3 py-1.5" />
			</label>
			<button
				onclick={exportarCsv}
				class="rounded-lg bg-slate-800 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-900"
			>
				Exportar CSV
			</button>
		</div>
	</section>
</div>