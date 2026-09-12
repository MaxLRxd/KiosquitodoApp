<script lang="ts">
	import { onMount } from 'svelte';
	import { carrito } from '$lib/stores/carrito.svelte';
	import { buscarProductoPorBarcode, listarProductos, registrarVenta } from '$lib/api';
	import { notificar } from '$lib/stores/app.svelte';
	import { crearListenerEscaneo } from '$lib/utils/scanner';
	import { reproducirSonido } from '$lib/utils/sonido';
	import { centavosDesdePesos, formatearPesos } from '$lib/utils/moneda';
	import type { MedioPago, Producto } from '$lib/types';

	let busqueda = $state('');
	let resultados = $state<Producto[]>([]);
	let buscando = $state(false);
	let medioPago = $state<MedioPago>('efectivo');
	let efectivoIngresado = $state('');
	let descripcionMp = $state('');
	let cobrando = $state(false);
	let inputBusqueda: HTMLInputElement;

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
		resultados = [];
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
			busqueda = '';
			enfocarBusqueda();
		}
	}

	async function manejarEnterBusqueda(): Promise<void> {
		const valor = busqueda.trim();
		if (valor === '') return;
		if (valor.length >= 8) {
			await manejarEscaneo(valor);
			return;
		}
		buscando = true;
		resultados = await listarProductos(valor, null, 20, 0);
		buscando = false;
		if (resultados.length === 0) {
			notificar(`Sin resultados para «${valor}»`, 'info');
			reproducirSonido('scan_error');
		}
		busqueda = '';
		enfocarBusqueda();
	}

	function limpiarPago(): void {
		medioPago = 'efectivo';
		efectivoIngresado = '';
		descripcionMp = '';
		resultados = [];
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
		if (evento.key === 'Escape') {
			busqueda = '';
			resultados = [];
			enfocarBusqueda();
			return;
		}
		if (evento.key === 'Enter' && document.activeElement !== inputBusqueda) {
			evento.preventDefault();
			cobrar();
		}
	}

	onMount(() => {
		const off = crearListenerEscaneo((codigo) => {
			void manejarEscaneo(codigo);
		});
		window.addEventListener('keydown', manejarTeclasGlobal);
		enfocarBusqueda();
		return () => {
			off();
			window.removeEventListener('keydown', manejarTeclasGlobal);
		};
	});
</script>

<div class="flex h-full">
	<section class="flex w-1/2 min-w-0 flex-col border-r border-slate-200 p-4">
		<input
			bind:this={inputBusqueda}
			bind:value={busqueda}
			onkeydown={(e) => {
				if (e.key === 'Enter') {
					e.preventDefault();
					void manejarEnterBusqueda();
				}
			}}
			type="text"
			placeholder="Escaneá un código o buscá por nombre… (Enter para buscar)"
			class="mb-3 w-full rounded-lg border border-slate-300 px-3 py-2 text-lg"
		/>

		{#if buscando}
			<p class="text-sm text-slate-500">Buscando…</p>
		{:else if resultados.length > 0}
			<ul class="space-y-1 overflow-auto">
				{#each resultados as producto (producto.id)}
					<li>
						<button
							onclick={() => agregarAlCarrito(producto)}
							disabled={stockDisponible(producto) <= 0}
							class="flex w-full items-center justify-between rounded-lg border border-slate-200 px-3 py-2 text-left hover:bg-blue-50 disabled:opacity-50"
						>
							<span>
								<span class="block font-medium">{producto.nombre}</span>
								<span class="block text-xs text-slate-500">Stock: {producto.stock}</span>
							</span>
							<span class="font-semibold">{formatearPesos(producto.precio_venta)}</span>
						</button>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="text-sm text-slate-400">
				Consejo: escaneá el código de barras con el lector USB y el ítem se agrega solo al
				carrito.
			</p>
		{/if}
	</section>

	<section class="flex w-1/2 min-w-0 flex-col p-4">
		<div class="flex items-center justify-between">
			<h2 class="text-lg font-bold">Venta actual</h2>
			<button
				onclick={cancelarVenta}
				class="rounded-lg border border-slate-300 px-3 py-1 text-sm text-slate-600 hover:bg-slate-100"
			>
				Cancelar venta
			</button>
		</div>

		{#if carrito.items.length === 0}
			<p class="py-8 text-center text-sm text-slate-400">Todavía no hay ítems en la venta.</p>
		{:else}
			<ul class="mt-2 flex-1 space-y-1 overflow-auto">
				{#each carrito.items as item (item.producto.id)}
					<li class="flex items-center justify-between rounded-lg border border-slate-200 px-3 py-2">
						<div class="min-w-0">
							<p class="truncate font-medium">
								{item.producto.nombre}
								<span class="text-xs text-slate-500">
									× {formatearPesos(item.producto.precio_venta)}
								</span>
							</p>
							<p class="text-xs text-slate-500">
								Subtotal: {formatearPesos(item.producto.precio_venta * item.cantidad)}
							</p>
						</div>
						<div class="flex items-center gap-1">
							<button
								onclick={() => carrito.asignarCantidad(item.producto.id, item.cantidad - 1)}
								class="h-8 w-8 rounded-md border border-slate-300"
							>
								−
							</button>
							<span class="w-8 text-center font-semibold">{item.cantidad}</span>
							<button
								onclick={() => {
									if (stockDisponible(item.producto) > 0) {
										carrito.asignarCantidad(item.producto.id, item.cantidad + 1);
									} else {
										notificar(`Stock insuficiente para ${item.producto.nombre}`, 'warn', true);
									}
								}}
								class="h-8 w-8 rounded-md border border-slate-300"
							>
								+
							</button>
							<button
								onclick={() => carrito.quitar(item.producto.id)}
								class="ml-2 h-8 w-8 rounded-md border border-red-200 text-red-600 hover:bg-red-50"
								aria-label="Quitar ítem"
							>
								✕
							</button>
						</div>
					</li>
				{/each}
			</ul>
		{/if}

		<div class="mt-4 space-y-3 border-t border-slate-200 pt-4">
			<div class="flex items-center justify-between text-lg font-bold">
				<span>Total</span>
				<span>{formatearPesos(carrito.total)}</span>
			</div>

			<div class="flex gap-2">
				{#each ['efectivo', 'mercado_pago', 'mixto'] as medio (medio)}
					<button
						onclick={() => (medioPago = medio as MedioPago)}
						class="flex-1 rounded-lg border px-3 py-2 text-sm font-medium {medioPago ===
						medio
							? 'border-blue-600 bg-blue-600 text-white'
							: 'border-slate-300 text-slate-700 hover:bg-slate-100'}"
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
					class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm"
				/>
			{/if}

			{#if medioPago === 'mixto'}
				<label class="block">
					<span class="text-xs text-slate-600">Cantidad recibida en efectivo</span>
					<input
						bind:value={efectivoIngresado}
						type="text"
						inputmode="decimal"
						placeholder="0"
						class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
					/>
				</label>
				<p class="text-sm text-slate-600">
					Resto en Mercado Pago: {formatearPesos(Math.max(0, restoMp()))}
				</p>
			{/if}

			<button
				onclick={cobrar}
				disabled={carrito.cantidadItems === 0 || cobrando}
				class="w-full rounded-lg bg-emerald-600 py-3 text-lg font-bold text-white hover:bg-emerald-700 disabled:opacity-40"
			>
				{#if cobrando}Registrando…{:else}Cobrar (Enter){/if}
			</button>
		</div>
	</section>
</div>