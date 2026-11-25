<script lang="ts">
	import { onMount } from 'svelte';
	import { carrito } from '$lib/stores/carrito.svelte';
	import { buscarProductoPorBarcode, listarProductos, registrarVenta } from '$lib/api';
	import { notificar } from '$lib/stores/app.svelte';
	import { crearListenerEscaneo, esCodigoBarras } from '$lib/utils/scanner';
	import { reproducirSonido } from '$lib/utils/sonido';
	import { centavosDesdePesos, formatearPesos } from '$lib/utils/moneda';
	import { coincideBusqueda } from '$lib/utils/busqueda';
	import type { MedioPago, Producto } from '$lib/types';

	const CATALOGO_INICIAL = 24;

	let busqueda = $state('');
	let resultados = $state<Producto[]>([]);
	let buscando = $state(false);
	let sinResultados = $state(false);
	let catalogo = $state<Producto[]>([]);
	let cargandoCatalogo = $state(false);
	let mostrarTodo = $state(false);
	let medioPago = $state<MedioPago>('efectivo');
	let efectivoIngresado = $state('');
	let descripcionMp = $state('');
	let cobrando = $state(false);
	let inputBusqueda: HTMLInputElement;
	let temporizadorBusqueda: ReturnType<typeof setTimeout> | null = null;
	let idBusqueda = 0;

	const hayBusqueda = $derived(busqueda.trim() !== '');
	const catalogoVisible = $derived(
		mostrarTodo ? catalogo : catalogo.slice(0, CATALOGO_INICIAL)
	);

	function enfocarBusqueda(): void {
		inputBusqueda?.focus();
	}

	function stockDisponible(producto: Producto): number {
		const enCarrito = carrito.items.find((it) => it.producto.id === producto.id)?.cantidad ?? 0;
		return producto.stock - enCarrito;
	}

	function agregarAlCarrito(producto: Producto): void {
		if (stockDisponible(producto) <= 0) {
			notificar(`Stock insuficiente para ${producto.nombre}`, 'warn', true);
			reproducirSonido('scan_error');
			return;
		}
		carrito.agregar(producto);
		notificar(`${producto.nombre} — ${formatearPesos(producto.precio_venta)}`, 'success');
		reproducirSonido('scan_ok');
		idBusqueda++;
		resultados = [];
		sinResultados = false;
		busqueda = '';
		enfocarBusqueda();
	}

	async function manejarEscaneo(codigo: string): Promise<void> {
		const producto = await buscarProductoPorBarcode(codigo);
		if (producto) {
			agregarAlCarrito(producto);
		} else {
			notificar(`Producto no encontrado: ${codigo}`, 'warn', true);
			reproducirSonido('scan_error');
			idBusqueda++;
			resultados = [];
			sinResultados = false;
			busqueda = '';
			enfocarBusqueda();
		}
	}

	async function cargarCatalogo(): Promise<void> {
		cargandoCatalogo = true;
		try {
			const productos = await listarProductos('', null, 100, 0);
			catalogo = productos.filter((producto) => producto.stock > 0);
		} catch (e) {
			const mensaje = e instanceof Error ? e.message : String(e);
			notificar(mensaje, 'error', true);
			catalogo = [];
		} finally {
			cargandoCatalogo = false;
		}
	}

	async function ejecutarBusqueda(valor: string): Promise<void> {
		const id = ++idBusqueda;
		buscando = true;
		try {
			let encontrados = await listarProductos(valor, null, 20, 0);
			if (id !== idBusqueda) return;
			// El LIKE del backend no perdona guiones ni acentos («coca cola» no
			// encuentra «Coca-Cola»). Si no hubo coincidencias se reintenta con
			// una comparación normalizada sobre el listado completo.
			if (encontrados.length === 0) {
				const todos = await listarProductos('', null, 200, 0);
				if (id !== idBusqueda) return;
				encontrados = todos.filter(
					(producto) =>
						coincideBusqueda(producto.nombre, valor) ||
						(producto.barcode ?? '').includes(valor)
				);
			}
			resultados = encontrados;
			sinResultados = encontrados.length === 0;
			if (sinResultados) reproducirSonido('scan_error');
		} catch (e) {
			if (id !== idBusqueda) return;
			const mensaje = e instanceof Error ? e.message : String(e);
			notificar(mensaje, 'error', true);
			resultados = [];
			sinResultados = true;
		} finally {
			if (id === idBusqueda) buscando = false;
		}
	}

	function programarBusqueda(): void {
		if (temporizadorBusqueda) clearTimeout(temporizadorBusqueda);
		const valor = busqueda.trim();
		sinResultados = false;
		if (valor === '') {
			idBusqueda++;
			resultados = [];
			buscando = false;
			return;
		}
		temporizadorBusqueda = setTimeout(() => {
			void ejecutarBusqueda(valor);
		}, 250);
	}

	async function manejarEnterBusqueda(): Promise<void> {
		const valor = busqueda.trim();
		if (valor === '') {
			enfocarBusqueda();
			return;
		}
		if (temporizadorBusqueda) clearTimeout(temporizadorBusqueda);
		if (esCodigoBarras(valor)) {
			await manejarEscaneo(valor);
			return;
		}
		await ejecutarBusqueda(valor);
	}

	function limpiarPago(): void {
		if (temporizadorBusqueda) clearTimeout(temporizadorBusqueda);
		idBusqueda++;
		medioPago = 'efectivo';
		efectivoIngresado = '';
		descripcionMp = '';
		resultados = [];
		sinResultados = false;
		buscando = false;
		busqueda = '';
	}

	function cancelarVenta(): void {
		if (carrito.cantidadItems === 0) return;
		const ok = window.confirm('¿Descartar la venta actual? Se pierden los ítems cargados.');
		if (ok) {
			carrito.vaciar();
			limpiarPago();
			notificar('Venta cancelada', 'info');
		}
		enfocarBusqueda();
	}

	function restoMp(): number {
		return carrito.total - centavosDesdePesos(efectivoIngresado);
	}

	async function cobrar(): Promise<void> {
		if (carrito.cantidadItems === 0 || cobrando) return;

		const total = carrito.total;
		let montoEfectivo = 0;
		let montoMp = 0;
		let descripcion = null as string | null;

		if (medioPago === 'efectivo') {
			montoEfectivo = total;
		} else if (medioPago === 'mercado_pago') {
			montoMp = total;
			descripcion = descripcionMp.trim() || null;
		} else {
			const efectivo = centavosDesdePesos(efectivoIngresado);
			if (efectivo <= 0 || efectivo >= total) {
				notificar('Ingresá un monto en efectivo entre $0 y el total', 'warn', true);
				reproducirSonido('sistema_error');
				return;
			}
			montoEfectivo = efectivo;
			montoMp = total - efectivo;
			descripcion = descripcionMp.trim() || null;
		}

		const pedido = {
			items: carrito.items.map((it) => ({
				producto_id: it.producto.id,
				cantidad: it.cantidad,
				precio_unitario: it.producto.precio_venta
			})),
			medio_pago: medioPago,
			monto_efectivo: montoEfectivo,
			monto_mp: montoMp,
			descripcion_mp: descripcion
		};

		cobrando = true;
		try {
			const venta = await registrarVenta(pedido);
			reproducirSonido('venta_ok');
			notificar(`Venta N° ${venta.id} registrada — ${formatearPesos(venta.total)}`, 'success');
			carrito.vaciar();
			limpiarPago();
			void cargarCatalogo();
		} catch (e) {
			const mensaje = e instanceof Error ? e.message : String(e);
			notificar(mensaje, 'error', true);
			reproducirSonido('sistema_error');
		} finally {
			cobrando = false;
			enfocarBusqueda();
		}
	}

	function manejarTeclasGlobal(evento: KeyboardEvent): void {
		if (evento.defaultPrevented) return;
		if (evento.key === 'Escape') {
			busqueda = '';
			resultados = [];
			sinResultados = false;
			enfocarBusqueda();
			return;
		}
		if (evento.key === 'Enter' && document.activeElement !== inputBusqueda) {
			evento.preventDefault();
			void cobrar();
		}
	}

	onMount(() => {
		void cargarCatalogo();
		const off = crearListenerEscaneo((codigo) => {
			void manejarEscaneo(codigo);
		});
		window.addEventListener('keydown', manejarTeclasGlobal);
		enfocarBusqueda();
		return () => {
			off();
			window.removeEventListener('keydown', manejarTeclasGlobal);
			if (temporizadorBusqueda) clearTimeout(temporizadorBusqueda);
		};
	});
</script>

{#snippet filaProducto(producto: Producto)}
	<li>
		<button
			onclick={() => agregarAlCarrito(producto)}
			disabled={stockDisponible(producto) <= 0}
			class="group flex w-full items-center justify-between gap-4 rounded-xl border border-slate-200 bg-white px-4 py-3 text-left transition hover:border-indigo-200 hover:bg-indigo-50/50 disabled:cursor-not-allowed disabled:opacity-50"
		>
			<span class="min-w-0">
				<span class="block truncate font-semibold text-slate-800">{producto.nombre}</span>
				<span class="mt-0.5 block text-xs text-slate-500">
					Stock disponible: {stockDisponible(producto)}
				</span>
			</span>
			<span class="flex shrink-0 items-center gap-3">
				<span class="font-bold tabular-nums text-slate-900">
					{formatearPesos(producto.precio_venta)}
				</span>
				<span
					class="grid h-8 w-8 place-items-center rounded-lg bg-slate-100 text-slate-500 transition group-hover:bg-indigo-600 group-hover:text-white"
				>
					+
				</span>
			</span>
		</button>
	</li>
{/snippet}

<div class="flex flex-col gap-5 p-5 lg:p-6 xl:h-full xl:min-h-0 xl:flex-row xl:overflow-hidden">
	<section class="card flex flex-col overflow-hidden xl:min-h-0 xl:flex-1">
		<div class="border-b border-slate-100 p-5 sm:p-6">
			<div class="flex items-center gap-2">
				<span class="badge-success">
					<span class="mr-1 inline-block h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
					Escáner listo
				</span>
				<span class="badge-neutral">F1</span>
			</div>

			<div class="relative mt-4">
				<svg
					class="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.8"
					stroke="currentColor"
					aria-hidden="true"
				>
					<circle cx="11" cy="11" r="7" />
					<path d="m16.5 16.5 4 4" stroke-linecap="round" />
				</svg>
				<input
					bind:this={inputBusqueda}
					bind:value={busqueda}
					oninput={programarBusqueda}
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							e.preventDefault();
							void manejarEnterBusqueda();
						}
					}}
					type="text"
					placeholder="Escaneá un código o buscá por nombre…"
					class="input !pl-11 text-base"
				/>
			</div>
			<p class="mt-2 text-xs text-slate-400">
				El lector USB agrega el producto al ticket · Enter para buscar · Esc para limpiar.
			</p>
		</div>

		<div class="min-h-0 flex-1 overflow-auto p-4 sm:p-5">
			{#if hayBusqueda}
				{#if buscando}
					<div class="flex items-center gap-2 text-sm text-slate-500">
						<span class="h-2 w-2 animate-pulse-soft rounded-full bg-indigo-500"></span>
						Buscando productos…
					</div>
				{:else if resultados.length > 0}
					<p class="mb-3 text-xs font-bold uppercase tracking-wider text-slate-400">
						{resultados.length} resultado(s)
					</p>
					<ul class="space-y-2">
						{#each resultados as producto (producto.id)}
							{@render filaProducto(producto)}
						{/each}
					</ul>
				{:else if sinResultados}
					<div class="empty-state h-full">
						<div class="grid h-12 w-12 place-items-center rounded-2xl bg-slate-100 text-slate-400">
							<svg
								class="h-6 w-6"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.6"
								stroke="currentColor"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<circle cx="11" cy="11" r="7" />
								<path d="m16.5 16.5 4 4" />
							</svg>
						</div>
						<p class="mt-3 text-sm font-semibold text-slate-700">Sin resultados</p>
						<p class="mt-1 max-w-xs text-xs leading-relaxed text-slate-500">
							No encontramos productos para «{busqueda.trim()}». Probá con otro nombre o
							escaneá el código de barras.
						</p>
					</div>
				{/if}
			{:else}
				<div class="mb-3 flex items-center justify-between gap-3">
					<p class="text-xs font-bold uppercase tracking-wider text-slate-400">
						Catálogo · {catalogo.length} con stock
					</p>
					{#if catalogo.length > CATALOGO_INICIAL}
						<button class="btn-ghost btn-sm" onclick={() => (mostrarTodo = !mostrarTodo)}>
							{mostrarTodo ? 'Ver menos' : `Ver todos (${catalogo.length})`}
						</button>
					{/if}
				</div>
				{#if cargandoCatalogo}
					<div class="flex items-center gap-2 text-sm text-slate-500">
						<span class="h-2 w-2 animate-pulse-soft rounded-full bg-indigo-500"></span>
						Cargando catálogo…
					</div>
				{:else if catalogoVisible.length === 0}
					<div class="empty-state h-full">
						<div class="grid h-12 w-12 place-items-center rounded-2xl bg-slate-100 text-slate-400">
							<svg
								class="h-6 w-6"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.6"
								stroke="currentColor"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<path d="M4 7V5.5A1.5 1.5 0 0 1 5.5 4h13A1.5 1.5 0 0 1 20 5.5V7" />
								<path d="M4 7h16v11.5A1.5 1.5 0 0 1 18.5 20h-13A1.5 1.5 0 0 1 4 18.5V7Z" />
								<path d="M9 11h6" />
							</svg>
						</div>
						<p class="mt-3 text-sm font-semibold text-slate-700">Sin productos con stock</p>
						<p class="mt-1 max-w-xs text-xs leading-relaxed text-slate-500">
							Reponé stock desde Inventario o escaneá un producto para agregarlo al ticket.
						</p>
					</div>
				{:else}
					<ul class="space-y-2">
						{#each catalogoVisible as producto (producto.id)}
							{@render filaProducto(producto)}
						{/each}
					</ul>
				{/if}
			{/if}
		</div>
	</section>

	<section class="card flex min-h-0 w-full flex-col overflow-hidden xl:w-[46%]">
		<div class="flex items-center justify-between gap-3 border-b border-slate-100 p-5 sm:p-6">
			<div>
				<p class="eyebrow">Ticket</p>
				<h2 class="mt-1 text-lg font-bold tracking-tight text-slate-900">Venta actual</h2>
			</div>
			<button onclick={cancelarVenta} class="btn-ghost btn-sm">Cancelar venta</button>
		</div>

		<div class="min-h-0 flex-1 overflow-auto p-4 sm:p-5">
			{#if carrito.items.length === 0}
				<div class="empty-state h-full">
					<div class="grid h-12 w-12 place-items-center rounded-2xl bg-slate-100 text-slate-400">
						<svg
							class="h-6 w-6"
							fill="none"
							viewBox="0 0 24 24"
							stroke-width="1.6"
							stroke="currentColor"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<path d="M3 4h2l2.2 11.2a2 2 0 0 0 2 1.6h7.9a2 2 0 0 0 2-1.6L20.5 8H6" />
							<circle cx="10" cy="20" r="1" />
							<circle cx="17" cy="20" r="1" />
						</svg>
					</div>
					<p class="mt-3 text-sm font-semibold text-slate-700">Todavía no hay ítems</p>
					<p class="mt-1 text-xs text-slate-500">Los productos que agregues se listan acá.</p>
				</div>
			{:else}
				<ul class="space-y-2">
					{#each carrito.items as item (item.producto.id)}
						<li
							class="flex items-center justify-between gap-3 rounded-xl border border-slate-200 bg-white px-3.5 py-3"
						>
							<div class="min-w-0">
								<p class="truncate font-semibold text-slate-800">{item.producto.nombre}</p>
								<p class="mt-0.5 text-xs text-slate-500">
									{formatearPesos(item.producto.precio_venta)} c/u · subtotal
									<span class="font-semibold text-slate-700">
										{formatearPesos(item.producto.precio_venta * item.cantidad)}
									</span>
								</p>
							</div>
							<div class="flex shrink-0 items-center gap-1">
								<button
									onclick={() => carrito.asignarCantidad(item.producto.id, item.cantidad - 1)}
									class="grid h-8 w-8 place-items-center rounded-lg border border-slate-200 text-slate-600 transition hover:bg-slate-50"
									aria-label="Restar uno"
								>
									−
								</button>
								<span class="w-7 text-center text-sm font-bold tabular-nums">
									{item.cantidad}
								</span>
								<button
									onclick={() => {
										if (stockDisponible(item.producto) > 0) {
											carrito.asignarCantidad(item.producto.id, item.cantidad + 1);
										} else {
											notificar(
												`Stock insuficiente para ${item.producto.nombre}`,
												'warn',
												true
											);
										}
									}}
									class="grid h-8 w-8 place-items-center rounded-lg border border-slate-200 text-slate-600 transition hover:bg-slate-50"
									aria-label="Sumar uno"
								>
									+
								</button>
								<button
									onclick={() => carrito.quitar(item.producto.id)}
									class="ml-1 grid h-8 w-8 place-items-center rounded-lg text-slate-400 transition hover:bg-rose-50 hover:text-rose-600"
									aria-label="Quitar ítem"
								>
									✕
								</button>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		</div>

		<div class="space-y-4 border-t border-slate-100 bg-slate-50/70 p-5 sm:p-6">
			<div class="flex items-end justify-between">
				<div>
					<p class="text-[11px] font-bold uppercase tracking-wider text-slate-500">Total</p>
					<p class="mt-0.5 text-xs text-slate-400">
						{carrito.cantidadItems} ítem(s) en el ticket
					</p>
				</div>
				<p class="text-3xl font-black tabular-nums tracking-tight text-slate-900">
					{formatearPesos(carrito.total)}
				</p>
			</div>

			<div class="grid grid-cols-3 gap-2">
				{#each ['efectivo', 'mercado_pago', 'mixto'] as medio (medio)}
					<button
						onclick={() => (medioPago = medio as MedioPago)}
						class="rounded-xl border px-2 py-2.5 text-xs font-bold transition sm:text-sm {medioPago ===
						medio
							? 'border-indigo-600 bg-indigo-600 text-white shadow-sm'
							: 'border-slate-200 bg-white text-slate-600 hover:border-slate-300 hover:bg-slate-50'}"
					>
						{medio === 'efectivo'
							? 'Efectivo'
							: medio === 'mercado_pago'
								? 'Mercado Pago'
								: 'Mixto'}
					</button>
				{/each}
			</div>

			{#if medioPago === 'mercado_pago' || medioPago === 'mixto'}
				<input
					bind:value={descripcionMp}
					type="text"
					placeholder="Descripción de la operación (opcional)"
					class="input text-sm"
				/>
			{/if}

			{#if medioPago === 'mixto'}
				<label class="block">
					<span class="text-xs font-semibold text-slate-600">Cantidad recibida en efectivo</span>
					<input
						bind:value={efectivoIngresado}
						type="text"
						inputmode="decimal"
						placeholder="0"
						class="input mt-1.5"
					/>
				</label>
				<p class="flex items-center justify-between text-sm text-slate-600">
					<span>Resto en Mercado Pago</span>
					<span class="font-bold tabular-nums text-slate-900">
						{formatearPesos(Math.max(0, restoMp()))}
					</span>
				</p>
			{/if}

			<button
				onclick={cobrar}
				disabled={carrito.cantidadItems === 0 || cobrando}
				class="btn-success w-full py-3.5 text-base"
			>
				{#if cobrando}
					Registrando…
				{:else}
					Cobrar
					<span class="rounded-md bg-white/15 px-1.5 py-0.5 text-[10px] font-bold">Enter</span>
				{/if}
			</button>
		</div>
	</section>
</div>