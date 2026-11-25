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

<div class="page">
	<div class="page-header">
		<div>
			<p class="eyebrow">Post-venta</p>
			<h1 class="page-title mt-1">Devoluciones</h1>
			<p class="page-subtitle">
				Devolución total o parcial de una venta existente. Al confirmar, el stock de los productos
				devueltos se incrementa automáticamente.
			</p>
		</div>
	</div>

	<section class="card card-pad mt-5">
		<h2 class="card-title">Buscar venta original</h2>
		<p class="section-subtitle">Ingresá el número de ticket impreso o visible en el POS.</p>
		<div class="mt-3 flex flex-wrap items-center gap-2">
			<input
				type="number"
				bind:value={busquedaId}
				placeholder="N° de venta"
				class="input-compact w-40"
			/>
			<button onclick={buscarVenta} class="btn-secondary">Buscar</button>
		</div>

		{#if ventaBusqueda}
			<div class="mt-4 rounded-xl border border-indigo-200 bg-indigo-50/60 p-4">
				<div class="flex flex-wrap items-start justify-between gap-4">
					<div class="text-sm">
						<p class="font-bold text-slate-800">
							Venta #<span class="tabular-nums">{ventaBusqueda.id}</span>
							<span class="ml-2 font-normal text-slate-500">
								{horaLegible(ventaBusqueda.hora)} · {ventaBusqueda.items.reduce(
									(acc, it) => acc + it.cantidad,
									0
								)} ítems
							</span>
						</p>
						<ul class="mt-1 space-y-0.5 text-slate-600">
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
						<span class="badge {colorMedio(ventaBusqueda.medio_pago)}">
							{etiquetaMedio(ventaBusqueda.medio_pago)}
						</span>
						<p class="mt-1.5 text-lg font-bold tabular-nums text-slate-900">
							{formatearPesos(ventaBusqueda.total)}
						</p>
						<button onclick={() => abrirDevolucion(ventaBusqueda!)} class="btn-danger btn-sm mt-2">
							Devolver
						</button>
					</div>
				</div>
			</div>
		{/if}
	</section>

	<section class="mt-8">
		<div class="flex flex-wrap items-end justify-between gap-4">
			<div>
				<h2 class="section-title">Ventas del día</h2>
				<p class="section-subtitle">Elegí la venta a devolver desde el listado.</p>
			</div>
			<label class="flex items-center gap-2 text-xs font-bold uppercase tracking-wide text-slate-500">
				Fecha
				<input type="date" bind:value={fecha} class="input-compact" />
			</label>
		</div>

		{#if cargandoVentas}
			<div class="card mt-4 flex items-center gap-2 p-5 text-sm text-slate-500">
				<span class="h-2 w-2 animate-pulse-soft rounded-full bg-indigo-500"></span>
				Cargando ventas…
			</div>
		{:else if ventas.length === 0}
			<div class="empty-state mt-4">
				<p class="text-sm font-semibold text-slate-700">No hay ventas en la fecha seleccionada</p>
				<p class="mt-1 text-xs text-slate-500">Probá con otra fecha del calendario.</p>
			</div>
		{:else}
			<div class="mt-4 space-y-2.5">
				{#each ventas as v (v.id)}
					<div class="card flex flex-wrap items-start justify-between gap-4 p-4">
						<div class="text-sm">
							<p class="font-bold text-slate-800">
								Venta #<span class="tabular-nums">{v.id}</span>
								<span class="ml-2 font-normal text-slate-500">{horaLegible(v.hora)}</span>
							</p>
							<ul class="mt-1 space-y-0.5 text-slate-600">
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
							<span class="badge {colorMedio(v.medio_pago)}">{etiquetaMedio(v.medio_pago)}</span>
							<p class="mt-1.5 text-lg font-bold tabular-nums text-slate-900">
								{formatearPesos(v.total)}
							</p>
							<button onclick={() => abrirDevolucion(v)} class="btn-danger btn-sm mt-2">
								Devolver
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</section>

	<section class="mt-8">
		<div class="flex flex-wrap items-end justify-between gap-3">
			<div>
				<h2 class="section-title">Devoluciones registradas</h2>
				<p class="section-subtitle">
					Recientes primero. Nota: el listado es mock-only; el backend hoy solo expone el registro.
				</p>
			</div>
			<span class="badge-neutral">{devoluciones.length} registro(s)</span>
		</div>

		{#if devoluciones.length === 0}
			<div class="empty-state mt-4">
				<p class="text-sm font-semibold text-slate-700">Todavía no hay devoluciones</p>
				<p class="mt-1 text-xs text-slate-500">
					Cuando confirmes una devolución, aparecerá en este historial.
				</p>
			</div>
		{:else}
			<div class="table-shell mt-4 overflow-x-auto">
				<table class="table">
					<thead>
						<tr>
							<th>Fecha</th>
							<th>Venta</th>
							<th>Ítems</th>
							<th>Motivo</th>
							<th class="text-right">Monto devuelto</th>
						</tr>
					</thead>
					<tbody>
						{#each devoluciones as d (d.id)}
							<tr>
								<td class="tabular-nums text-slate-600">
									{d.fecha.slice(0, 10)} {d.fecha.slice(11, 19)}
								</td>
								<td class="font-semibold text-slate-800">
									#<span class="tabular-nums">{d.venta_id}</span>
								</td>
								<td class="text-slate-600">
									{d.items.map((it) => `${it.cantidad}×${nombreProducto(it.producto_id)}`).join(', ')}
								</td>
								<td>{d.motivo ?? '—'}</td>
								<td class="text-right font-bold tabular-nums text-rose-600">
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
	<div class="modal-overlay">
		<button
			type="button"
			aria-label="Cerrar"
			class="absolute inset-0"
			onclick={() => (modalDevolucion = false)}
		></button>
		<div class="modal-panel max-w-lg">
			<div class="mb-4">
				<p class="eyebrow">Devolución</p>
				<h3 class="modal-title mt-1">
					Devolver venta #<span class="tabular-nums">{ventaSel.id}</span>
				</h3>
				<p class="mt-1 text-xs leading-relaxed text-slate-500">
					Marcá la cantidad a devolver de cada ítem (0 = no devolver). Esta devolución podés
					compensarla fuera del sistema; no aplica reverso automático de Mercado Pago.
				</p>
			</div>

			<div class="space-y-2">
				{#each ventaSel.items as it}
					<div class="flex items-center justify-between gap-3 rounded-xl border border-slate-200 px-3.5 py-2.5">
						<div class="text-sm">
							<p class="font-semibold text-slate-800">{nombreProducto(it.producto_id)}</p>
							<p class="text-xs text-slate-500">
								{formatearPesos(it.precio_unitario)} · vendidos: {it.cantidad}
							</p>
						</div>
						<label class="flex items-center gap-2 text-xs font-semibold text-slate-500">
							Devolver
							<input
								type="number"
								min="0"
								max={it.cantidad}
								bind:value={cantidades[it.producto_id]}
								class="input-compact w-20 text-right tabular-nums"
							/>
						</label>
					</div>
				{/each}
			</div>

			<label class="label mt-4">
				Motivo (opcional)
				<input
					type="text"
					bind:value={motivo}
					list="motivosDevolucion"
					placeholder="ej: no le gustó, error de cobro"
					class="input"
				/>
			</label>
			<datalist id="motivosDevolucion">
				<option value="no le gustó"></option>
				<option value="error de cobro"></option>
				<option value="cambio de opinión"></option>
				<option value="producto defectuoso"></option>
				<option value="otro"></option>
			</datalist>

			<div class="mt-5 flex flex-wrap items-center justify-between gap-3 border-t border-slate-100 pt-4">
				<p class="text-sm text-slate-600">
					Total a devolver:
					<span class="ml-1 text-lg font-bold tabular-nums text-slate-900">
						{formatearPesos(totalDevolver)}
					</span>
				</p>
				<div class="flex gap-2">
					<button onclick={() => (modalDevolucion = false)} class="btn-ghost">Cancelar</button>
					<button
						onclick={confirmarDevolucion}
						disabled={devolviendo || totalDevolver <= 0}
						class="btn-danger"
					>
						{devolviendo ? 'Registrando…' : 'Confirmar devolución'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
