<script lang="ts">
	import { onMount } from 'svelte';
	import {
		listarDevoluciones,
		listarProductos,
		listarVentasDia,
		obtenerVenta,
		registrarDevolucion
	} from '$lib/api';
	import { notificar } from '$lib/stores/app.svelte';
	import { reproducirSonido } from '$lib/utils/sonido';
	import { formatearPesos } from '$lib/utils/moneda';
	import type { DevolucionRegistrada, ItemVenta, MedioPago, Venta } from '$lib/types';

	function hoy(): string {
		const ahora = new Date();
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${ahora.getFullYear()}-${pad(ahora.getMonth() + 1)}-${pad(ahora.getDate())}`;
	}

	function etiquetaMedio(m: MedioPago): string {
		if (m === 'efectivo') return 'Efectivo';
		if (m === 'mercado_pago') return 'Mercado Pago';
		return 'Mixto';
	}

	function colorMedio(m: MedioPago): string {
		if (m === 'efectivo') return 'bg-emerald-100 text-emerald-800';
		if (m === 'mercado_pago') return 'bg-sky-100 text-sky-800';
		return 'bg-violet-100 text-violet-800';
	}

	function montoDevuelto(d: DevolucionRegistrada): number {
		return d.items.reduce((acc, it) => acc + it.cantidad * it.precio_unitario, 0);
	}

	function horaLegible(hora: string): string {
		return hora.length >= 16 ? hora.slice(11, 16) : hora;
	}

	let fecha = $state(hoy());
	let ventas = $state<Venta[]>([]);
	let devoluciones = $state<DevolucionRegistrada[]>([]);
	let nombres = $state<Record<number, string>>({});
	let cargandoVentas = $state(false);

	let busquedaId = $state('');
	let ventaBusqueda = $state<Venta | null>(null);

	let modalDevolucion = $state(false);
	let ventaSel = $state<Venta | null>(null);
	let cantidades = $state<Record<number, string>>({});
	let motivo = $state('');
	let devolviendo = $state(false);

	const totalDevolver = $derived(
		ventaSel
			? ventaSel.items.reduce(
					(acc, it) => acc + (Number(cantidades[it.producto_id]) || 0) * it.precio_unitario,
					0
				)
			: 0
	);

	function nombreProducto(id: number): string {
		return nombres[id] ?? `#${id}`;
	}

	async function cargarVentas(): Promise<void> {
		cargandoVentas = true;
		try {
			ventas = await listarVentasDia(fecha);
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			cargandoVentas = false;
		}
	}

	async function cargarDevoluciones(): Promise<void> {
		try {
			devoluciones = await listarDevoluciones();
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		}
	}

	async function buscarVenta(): Promise<void> {
		const id = Number(busquedaId.trim());
		if (!id) {
			notificar('Ingresá un número de venta', 'warn', true);
			return;
		}
		try {
			ventaBusqueda = await obtenerVenta(id);
			if (!ventaBusqueda) notificar(`No existe la venta #${id}`, 'warn', true);
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		}
	}

	function abrirDevolucion(v: Venta): void {
		ventaSel = v;
		cantidades = {};
		for (const it of v.items) cantidades[it.producto_id] = String(it.cantidad);
		motivo = '';
		modalDevolucion = true;
	}

	async function confirmarDevolucion(): Promise<void> {
		if (!ventaSel || devolviendo) return;
		const items: ItemVenta[] = [];
		for (const it of ventaSel.items) {
			const q = Math.floor(Number(cantidades[it.producto_id]));
			if (q > 0) {
				items.push({
					producto_id: it.producto_id,
					cantidad: q,
					precio_unitario: it.precio_unitario
				});
			}
		}
		if (items.length === 0) {
			notificar('Seleccioná al menos un ítem con cantidad para devolver', 'warn', true);
			return;
		}
		devolviendo = true;
		try {
			await registrarDevolucion(ventaSel.id, items, motivo.trim() === '' ? null : motivo.trim());
			reproducirSonido('venta_ok');
			notificar(`Devolución registrada sobre la venta #${ventaSel.id} (stock repuesto)`, 'success');
			modalDevolucion = false;
			ventaSel = null;
			await Promise.all([cargarVentas(), cargarDevoluciones()]);
			if (ventaBusqueda && items.length > 0) ventaBusqueda = await obtenerVenta(ventaBusqueda.id);
		} catch (e) {
			notificar((e as Error).message, 'error', true);
		} finally {
			devolviendo = false;
		}
	}

	$effect(() => {
		fecha;
		void cargarVentas();
	});

	onMount(() => {
		void cargarDevoluciones();
		void listarProductos('', null, 1000, 0)
			.then((ps) => {
				nombres = Object.fromEntries(ps.map((p) => [p.id, p.nombre]));
			})
			.catch(() => {
				/* sin nombres: se muestra el id */
			});
	});
</script>

<div class="p-6">
	<h1 class="text-2xl font-bold">Devoluciones</h1>
	<p class="text-sm text-slate-500">
		Devolución total o parcial de una venta existente. Al confirmar, el stock de los productos
		devueltos se incrementa automáticamente.
	</p>

	<section class="mt-6">
		<h2 class="text-sm font-semibold text-slate-700">Buscar venta original</h2>
		<div class="mt-2 flex items-center gap-2">
			<input
				type="number"
				bind:value={busquedaId}
				placeholder="N° de venta"
				class="w-40 rounded-md border border-slate-300 px-3 py-1.5"
			/>
			<button
				onclick={buscarVenta}
				class="rounded-lg bg-slate-800 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-900"
			>
				Buscar
			</button>
		</div>

		{#if ventaBusqueda}
			<div class="mt-3 rounded-xl border border-blue-200 bg-blue-50 p-4">
				<div class="flex items-center justify-between gap-4">
					<div class="text-sm">
						<p class="font-semibold text-slate-800">
							Venta #<span class="tabular-nums">{ventaBusqueda.id}</span>
							<span class="ml-2 text-slate-500">
								{horaLegible(ventaBusqueda.hora)} · {ventaBusqueda.items.reduce(
									(acc, it) => acc + it.cantidad,
									0
								)} ítems
							</span>
						</p>
						<ul class="mt-1 text-slate-600">
							{#each ventaBusqueda.items as it}
								<li>
									{it.cantidad} × {nombreProducto(it.producto_id)} · {formatearPesos(
										it.precio_unitario
									)}
								</li>
							{/each}
						</ul>
					</div>
					<div class="text-right">
						<span
							class="inline-block rounded-full px-2 py-0.5 text-xs font-semibold {colorMedio(
								ventaBusqueda.medio_pago
							)}"
						>
							{etiquetaMedio(ventaBusqueda.medio_pago)}
						</span>
						<p class="mt-1 font-bold">{formatearPesos(ventaBusqueda.total)}</p>
						<button
							onclick={() => abrirDevolucion(ventaBusqueda!)}
							class="mt-2 rounded-lg bg-red-600 px-3 py-1.5 text-sm font-semibold text-white hover:bg-red-700"
						>
							Devolver
						</button>
					</div>
				</div>
			</div>
		{/if}
	</section>

	<section class="mt-8">
		<div class="flex flex-wrap items-center justify-between gap-4">
			<h2 class="text-lg font-bold">Ventas del día</h2>
			<label class="flex items-center gap-2 text-sm font-medium text-slate-700">
				Fecha
				<input
					type="date"
					bind:value={fecha}
					class="rounded-md border border-slate-300 px-3 py-1.5"
				/>
			</label>
		</div>

		{#if cargandoVentas}
			<p class="mt-3 text-sm text-slate-500">Cargando ventas…</p>
		{:else if ventas.length === 0}
			<p class="mt-3 text-sm text-slate-500">No hay ventas en la fecha seleccionada.</p>
		{:else}
			<div class="mt-3 space-y-3">
				{#each ventas as v (v.id)}
					<div class="rounded-xl border border-slate-200 bg-white p-4">
						<div class="flex items-center justify-between gap-4">
							<div class="text-sm">
								<p class="font-semibold text-slate-800">
									Venta #<span class="tabular-nums">{v.id}</span>
									<span class="ml-2 font-normal text-slate-500">{horaLegible(v.hora)}</span>
								</p>
								<ul class="mt-1 text-slate-600">
									{#each v.items as it}
										<li>
											{it.cantidad} × {nombreProducto(it.producto_id)} · {formatearPesos(
												it.precio_unitario
											)}
										</li>
									{/each}
								</ul>
							</div>
							<div class="text-right">
								<span
									class="inline-block rounded-full px-2 py-0.5 text-xs font-semibold {colorMedio(
										v.medio_pago
									)}"
								>
									{etiquetaMedio(v.medio_pago)}
								</span>
								<p class="mt-1 font-bold">{formatearPesos(v.total)}</p>
								<button
									onclick={() => abrirDevolucion(v)}
									class="mt-2 rounded-lg bg-red-600 px-3 py-1.5 text-sm font-semibold text-white hover:bg-red-700"
								>
									Devolver
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</section>

	<section class="mt-8">
		<h2 class="text-lg font-bold">Devoluciones registradas</h2>
		<p class="text-sm text-slate-500">
			Recientes primero. Nota: el listado es mock-only; el backend hoy solo expone el registro.
		</p>

		{#if devoluciones.length === 0}
			<p class="mt-3 text-sm text-slate-500">Todavía no hay devoluciones registradas.</p>
		{:else}
			<div class="mt-3 overflow-x-auto rounded-xl border border-slate-200">
				<table class="w-full text-left text-sm">
					<thead class="bg-slate-50 text-xs uppercase tracking-wide text-slate-500">
						<tr>
							<th class="px-4 py-2">Fecha</th>
							<th class="px-4 py-2">Venta</th>
							<th class="px-4 py-2">Ítems</th>
							<th class="px-4 py-2">Motivo</th>
							<th class="px-4 py-2 text-right">Monto devuelto</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each devoluciones as d (d.id)}
							<tr>
								<td class="px-4 py-2 tabular-nums text-slate-600">
									{d.fecha.slice(0, 10)} {d.fecha.slice(11, 19)}
								</td>
								<td class="px-4 py-2 font-medium text-slate-800">
									#<span class="tabular-nums">{d.venta_id}</span>
								</td>
								<td class="px-4 py-2 text-slate-600">
									{d.items.map((it) => `${it.cantidad}×${nombreProducto(it.producto_id)}`).join(', ')}
								</td>
								<td class="px-4 py-2">{d.motivo ?? '—'}</td>
								<td class="px-4 py-2 text-right font-semibold">
									{formatearPesos(montoDevuelto(d))}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>

{#if modalDevolucion && ventaSel}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<button
			type="button"
			aria-label="Cerrar"
			class="absolute inset-0 bg-black/40"
			onclick={() => (modalDevolucion = false)}
		></button>
		<div class="relative z-10 w-full max-w-lg rounded-xl bg-white p-6 shadow-xl">
			<h3 class="text-lg font-bold">Devolver venta #<span class="tabular-nums">{ventaSel.id}</span></h3>
			<p class="text-sm text-slate-500">
				Marcá la cantidad a devolver de cada ítem (0 = no devolver). Esta devolución podés
				compensarla fuera del sistema; no aplica reverso automático de Mercado Pago.
			</p>

			<div class="mt-4 space-y-2">
				{#each ventaSel.items as it}
					<div class="flex items-center justify-between gap-3 rounded-lg border border-slate-200 px-3 py-2">
						<div class="text-sm">
							<p class="font-medium text-slate-800">{nombreProducto(it.producto_id)}</p>
							<p class="text-slate-500">
								{formatearPesos(it.precio_unitario)} · vendidos: {it.cantidad}
							</p>
						</div>
						<label class="flex items-center gap-2 text-sm text-slate-600">
							Devolver
							<input
								type="number"
								min="0"
								max={it.cantidad}
								bind:value={cantidades[it.producto_id]}
								class="w-20 rounded-md border border-slate-300 px-2 py-1 text-right tabular-nums"
							/>
						</label>
					</div>
				{/each}
			</div>

			<label class="mt-4 block text-sm font-medium text-slate-700">
				Motivo (opcional)
				<input
					type="text"
					bind:value={motivo}
					list="motivosDevolucion"
					placeholder="ej: no le gustó, error de cobro"
					class="mt-1 w-full rounded-md border border-slate-300 px-3 py-1.5"
				/>
			</label>
			<datalist id="motivosDevolucion">
				<option value="no le gustó"></option>
				<option value="error de cobro"></option>
				<option value="cambio de opinión"></option>
				<option value="producto defectuoso"></option>
				<option value="otro"></option>
			</datalist>

			<div class="mt-4 flex items-center justify-between">
				<p class="text-sm text-slate-600">
					Total a devolver:
					<span class="ml-1 font-bold text-slate-800">{formatearPesos(totalDevolver)}</span>
				</p>
				<div class="flex gap-2">
					<button
						onclick={() => (modalDevolucion = false)}
						class="rounded-lg px-4 py-2 text-sm font-semibold text-slate-600 hover:bg-slate-100"
					>
						Cancelar
					</button>
					<button
						onclick={confirmarDevolucion}
						disabled={devolviendo || totalDevolver <= 0}
						class="rounded-lg bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
					>
						{devolviendo ? 'Registrando…' : 'Confirmar devolución'}
					</button>
			</div>
		</div>
	</div>
	</div>
{/if}