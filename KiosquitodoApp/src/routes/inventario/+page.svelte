<script lang="ts">
	import { onMount } from 'svelte';
	import {
		actualizarProducto,
		ajustarStock,
		crearCategoria,
		crearProducto,
		eliminarProducto,
		listarCategorias,
		listarProductos
	} from '$lib/api';
	import { notificar } from '$lib/stores/app.svelte';
	import { reproducirSonido } from '$lib/utils/sonido';
	import { centavosDesdePesos, formatearPesos } from '$lib/utils/moneda';
	import type { Categoria, Producto } from '$lib/types';

	const TAM_PAGINA = 20;
	const TAM_FETCH = 1000;

	let filtro = $state('');
	let categoriaId = $state<number | null>(null);
	let categorias = $state<Categoria[]>([]);
	let productos = $state<Producto[]>([]);
	let cargando = $state(false);
	let pagina = $state(1);

	let modalProducto = $state(false);
	let editandoId = $state<number | null>(null);
	let fNombre = $state('');
	let fBarcode = $state('');
	let fPrecioVenta = $state('');
	let fPrecioCosto = $state('');
	let fStock = $state('0');
	let fStockMinimo = $state('0');
	let fCategoriaId = $state<number | null>(null);
	let mostrarNuevaCategoria = $state(false);
	let fCatNombre = $state('');
	let fCatColor = $state('#3b82f6');
	let guardando = $state(false);

	let modalStock = $state(false);
	let stockProducto = $state<Producto | null>(null);
	let sNuevo = $state('0');
	let sMotivo = $state('');
	let sOperador = $state('');

	let paginas = $derived(Math.max(1, Math.ceil(productos.length / TAM_PAGINA)));
	let paginados = $derived(productos.slice((pagina - 1) * TAM_PAGINA, pagina * TAM_PAGINA));

	function aPesos(centavos: number): string {
		return String(centavos / 100).replace('.', ',');
	}

	function estadoStock(producto: Producto): 'ok' | 'bajo' | 'agotado' {
		if (producto.stock === 0) return 'agotado';
		if (producto.stock <= producto.stock_minimo) return 'bajo';
		return 'ok';
	}

	function nombreCategoria(id: number | null): string {
		if (id === null) return '—';
		return categorias.find((c) => c.id === id)?.nombre ?? `#${id}`;
	}

	function barcodeValido(codigo: string): boolean {
		const limpio = codigo.trim();
		return (limpio.length === 8 || limpio.length === 12 || limpio.length === 13) &&
			/^\d+$/.test(limpio);
	}

	async function cargar(): Promise<void> {
		pagina = 1;
		cargando = true;
		try {
			productos = await listarProductos(filtro, categoriaId, TAM_FETCH, 0);
		} finally {
			cargando = false;
		}
	}

	function abrirNuevo(): void {
		editandoId = null;
		fNombre = '';
		fBarcode = '';
		fPrecioVenta = '';
		fPrecioCosto = '';
		fStock = '0';
		fStockMinimo = '0';
		fCategoriaId = null;
		mostrarNuevaCategoria = false;
		modalProducto = true;
	}

	function abrirEditar(p: Producto): void {
		editandoId = p.id;
		fNombre = p.nombre;
		fBarcode = p.barcode ?? '';
		fPrecioVenta = aPesos(p.precio_venta);
		fPrecioCosto = p.precio_costo !== null ? aPesos(p.precio_costo) : '';
		fStock = String(p.stock);
		fStockMinimo = String(p.stock_minimo);
		fCategoriaId = p.categoria_id;
		mostrarNuevaCategoria = false;
		modalProducto = true;
	}

	async function guardarProducto(): Promise<void> {
		if (guardando) return;
		if (fNombre.trim() === '') {
			notificar('El nombre del producto no puede estar vacío', 'warn', true);
			return;
		}
		if (!barcodeValido(fBarcode)) {
			notificar('El código de barras debe tener 8, 12 o 13 dígitos', 'warn', true);
			return;
		}
		const precioVenta = centavosDesdePesos(fPrecioVenta);
		if (precioVenta <= 0) {
			notificar('El precio de venta debe ser mayor a cero', 'warn', true);
			return;
		}
		const precioCosto = fPrecioCosto.trim() !== '' ? centavosDesdePesos(fPrecioCosto) : null;
		if (precioCosto !== null && precioCosto < 0) {
			notificar('El precio de costo no puede ser negativo', 'warn', true);
			return;
		}

		guardando = true;
		try {
			if (editandoId === null) {
				await crearProducto({
					nombre: fNombre.trim(),
					barcode: fBarcode.trim(),
					precio_venta: precioVenta,
					precio_costo: precioCosto,
					stock_inicial: Number(fStock) || 0,
					stock_minimo: Number(fStockMinimo) || 0,
					categoria_id: fCategoriaId
				});
				notificar(`Producto «${fNombre.trim()}» creado`, 'success');
			} else {
				const actual = productos.find((p) => p.id === editandoId);
				if (!actual) throw new Error('El producto no existe');
				await actualizarProducto({
					...actual,
					nombre: fNombre.trim(),
					barcode: fBarcode.trim(),
					precio_venta: precioVenta,
					precio_costo: precioCosto,
					stock: Number(fStock) || 0,
					stock_minimo: Number(fStockMinimo) || 0,
					categoria_id: fCategoriaId,
					activo: true
				});
				notificar('Producto actualizado', 'success');
			}
			reproducirSonido('venta_ok');
			modalProducto = false;
			await cargar();
		} catch (e) {
			const mensaje = e instanceof Error ? e.message : String(e);
			notificar(mensaje, 'error', true);
			reproducirSonido('sistema_error');
		} finally {
			guardando = false;
		}
	}

	async function guardarCategoria(): Promise<void> {
		try {
			const creada = await crearCategoria({ nombre: fCatNombre, color: fCatColor });
			categorias = await listarCategorias();
			fCategoriaId = creada.id;
			fCatNombre = '';
			mostrarNuevaCategoria = false;
			notificar(`Categoría «${creada.nombre}» creada`, 'success');
		} catch (e) {
			notificar(e instanceof Error ? e.message : String(e), 'error', true);
		}
	}

	function abrirAjuste(p: Producto): void {
		stockProducto = p;
		sNuevo = String(p.stock);
		sMotivo = '';
		sOperador = '';
		modalStock = true;
	}

	async function guardarAjuste(): Promise<void> {
		if (!stockProducto || guardando) return;
		if (sMotivo.trim() === '') {
			notificar('Debe indicar un motivo para el ajuste de stock', 'warn', true);
			return;
		}
		guardando = true;
		try {
			await ajustarStock(
				stockProducto.id,
				Number(sNuevo) || 0,
				sMotivo.trim(),
				sOperador.trim() || null
			);
			notificar('Stock ajustado', 'success');
			modalStock = false;
			await cargar();
		} catch (e) {
			notificar(e instanceof Error ? e.message : String(e), 'error', true);
		} finally {
			guardando = false;
		}
	}

	async function eliminar(p: Producto): Promise<void> {
		const ok = window.confirm(`¿Dar de baja «${p.nombre}»? La venta conserva su historial.`);
		if (!ok) return;
		try {
			await eliminarProducto(p.id);
			notificar(`Producto «${p.nombre}» dado de baja`, 'info');
			await cargar();
		} catch (e) {
			notificar(e instanceof Error ? e.message : String(e), 'error', true);
		}
	}

	onMount(() => {
		void cargar();
		void listarCategorias().then((c) => (categorias = c));
	});
</script>

<div class="flex h-full flex-col p-6">
	<div class="flex flex-wrap items-center gap-3">
		<h1 class="text-2xl font-bold">Inventario</h1>
		<input
			bind:value={filtro}
			oninput={cargar}
			type="text"
			placeholder="Buscar por nombre o código de barras…"
			class="min-w-64 flex-1 rounded-lg border border-slate-300 px-3 py-2"
		/>
		<select
			bind:value={categoriaId}
			onchange={cargar}
			class="rounded-lg border border-slate-300 px-3 py-2"
		>
			<option value={null}>Todas las categorías</option>
			{#each categorias as categoria (categoria.id)}
				<option value={categoria.id}>{categoria.nombre}</option>
			{/each}
		</select>
		<button
			onclick={abrirNuevo}
			class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700"
		>
			+ Nuevo producto
		</button>
	</div>

	<div class="mt-4 flex-1 overflow-auto rounded-lg border border-slate-200">
		<table class="w-full text-sm">
			<thead class="sticky top-0 bg-white text-left text-xs uppercase tracking-wide text-slate-500">
				<tr>
					<th class="px-3 py-2">Producto</th>
					<th class="px-3 py-2">Código</th>
					<th class="px-3 py-2">Categoría</th>
					<th class="px-3 py-2 text-right">P. venta</th>
					<th class="px-3 py-2 text-right">Costo</th>
					<th class="px-3 py-2 text-right">Stock</th>
					<th class="px-3 py-2 text-right">Mínimo</th>
					<th class="px-3 py-2">Acciones</th>
				</tr>
			</thead>
			<tbody>
				{#if cargando}
					<tr>
						<td colspan="8" class="px-3 py-6 text-center text-slate-400">Cargando…</td>
					</tr>
				{:else if paginados.length === 0}
					<tr>
						<td colspan="8" class="px-3 py-6 text-center text-slate-400">Sin productos.</td>
					</tr>
				{:else}
					{#each paginados as p (p.id)}
						<tr class="border-t border-slate-100">
							<td class="px-3 py-2 font-medium">
								{p.nombre}
								{#if estadoStock(p) === 'agotado'}
									<span class="ml-2 rounded bg-red-100 px-2 py-0.5 text-xs text-red-700">Agotado</span>
								{:else if estadoStock(p) === 'bajo'}
									<span class="ml-2 rounded bg-amber-100 px-2 py-0.5 text-xs text-amber-700">
										Stock bajo
									</span>
								{/if}
							</td>
							<td class="px-3 py-2 font-mono text-xs text-slate-500">{p.barcode ?? '—'}</td>
							<td class="px-3 py-2 text-slate-600">{nombreCategoria(p.categoria_id)}</td>
							<td class="px-3 py-2 text-right">{formatearPesos(p.precio_venta)}</td>
							<td class="px-3 py-2 text-right text-slate-600">
								{p.precio_costo !== null ? formatearPesos(p.precio_costo) : '—'}
							</td>
							<td class="px-3 py-2 text-right font-semibold">{p.stock}</td>
							<td class="px-3 py-2 text-right text-slate-500">{p.stock_minimo}</td>
							<td class="px-3 py-2">
								<div class="flex gap-1">
									<button
										onclick={() => abrirEditar(p)}
										class="rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100"
									>
										Editar
									</button>
									<button
										onclick={() => abrirAjuste(p)}
										class="rounded border border-slate-300 px-2 py-1 text-xs hover:bg-slate-100"
									>
										Stock
									</button>
									<button
										onclick={() => eliminar(p)}
										class="rounded border border-red-200 px-2 py-1 text-xs text-red-600 hover:bg-red-50"
									>
										Baja
									</button>
								</div>
							</td>
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>

	<div class="mt-3 flex items-center justify-between text-sm text-slate-600">
		<span>{productos.length} producto(s)</span>
		<div class="flex items-center gap-2">
			<button
				onclick={() => (pagina = Math.max(1, pagina - 1))}
				disabled={pagina <= 1}
				class="rounded border border-slate-300 px-3 py-1 disabled:opacity-40"
			>
				‹ Anterior
			</button>
			<span>Página {pagina} de {paginas}</span>
			<button
				onclick={() => (pagina = Math.min(paginas, pagina + 1))}
				disabled={pagina >= paginas}
				class="rounded border border-slate-300 px-3 py-1 disabled:opacity-40"
			>
				Siguiente ›
			</button>
		</div>
	</div>
</div>

{#if modalProducto}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<button
			type="button"
			aria-label="Cerrar"
			class="absolute inset-0 bg-black/40"
			onclick={() => (modalProducto = false)}
		></button>
		<div class="relative z-10 w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
			<h2 class="mb-4 text-lg font-bold">{editandoId === null ? 'Nuevo producto' : 'Editar producto'}</h2>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					void guardarProducto();
				}}
				class="space-y-3"
			>
				<label class="block">
					<span class="text-xs text-slate-600">Nombre *</span>
					<input
						bind:value={fNombre}
						type="text"
						class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
					/>
				</label>
				<label class="block">
					<span class="text-xs text-slate-600">Código de barras (8, 12 o 13 dígitos) *</span>
					<input
						bind:value={fBarcode}
						type="text"
						inputmode="numeric"
						class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
					/>
				</label>
				<div class="grid grid-cols-2 gap-3">
					<label class="block">
						<span class="text-xs text-slate-600">Precio de venta ($) *</span>
						<input
							bind:value={fPrecioVenta}
							type="text"
							inputmode="decimal"
							class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
						/>
					</label>
					<label class="block">
						<span class="text-xs text-slate-600">Precio de costo ($)</span>
						<input
							bind:value={fPrecioCosto}
							type="text"
							inputmode="decimal"
							class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
						/>
					</label>
				</div>
				<div class="grid grid-cols-2 gap-3">
					<label class="block">
						<span class="text-xs text-slate-600">
							{editandoId === null ? 'Stock inicial' : 'Stock actual'}
						</span>
						<input
							bind:value={fStock}
							type="number"
							min="0"
							class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
						/>
					</label>
					<label class="block">
						<span class="text-xs text-slate-600">Stock mínimo de alerta</span>
						<input
							bind:value={fStockMinimo}
							type="number"
							min="0"
							class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
						/>
					</label>
				</div>
				<label class="block">
					<span class="text-xs text-slate-600">Categoría</span>
					<select
						bind:value={fCategoriaId}
						class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
					>
						<option value={null}>Sin categoría</option>
						{#each categorias as categoria (categoria.id)}
							<option value={categoria.id}>{categoria.nombre}</option>
						{/each}
					</select>
				</label>

				{#if mostrarNuevaCategoria}
					<div class="rounded-lg border border-slate-200 p-3">
						<p class="text-xs font-medium text-slate-600">Nueva categoría</p>
						<div class="mt-2 flex gap-2">
							<input
								bind:value={fCatNombre}
								type="text"
								placeholder="Nombre"
								class="flex-1 rounded-lg border border-slate-300 px-2 py-1"
							/>
							<input
								bind:value={fCatColor}
								type="color"
								class="h-9 w-10 rounded border border-slate-300"
							/>
							<button
								onclick={() => void guardarCategoria()}
								type="button"
								class="rounded-lg bg-blue-600 px-3 py-1 text-white"
							>
								Guardar
							</button>
						</div>
					</div>
				{:else}
					<button
						type="button"
						onclick={() => (mostrarNuevaCategoria = true)}
						class="text-xs text-blue-600 hover:underline"
					>
						+ Nueva categoría
					</button>
				{/if}

				<div class="flex justify-end gap-2 pt-2">
					<button
						type="button"
						onclick={() => (modalProducto = false)}
						class="rounded-lg border border-slate-300 px-4 py-2 text-sm"
					>
						Cancelar
					</button>
					<button
						type="submit"
						disabled={guardando}
						class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
					>
						{guardando ? 'Guardando…' : 'Guardar'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

{#if modalStock}
	<div class="fixed inset-0 z-50 flex items-center justify-center">
		<button
			type="button"
			aria-label="Cerrar"
			class="absolute inset-0 bg-black/40"
			onclick={() => (modalStock = false)}
		></button>
		<div class="relative z-10 w-full max-w-sm rounded-xl bg-white p-6 shadow-xl">
			<h2 class="mb-1 text-lg font-bold">Ajustar stock</h2>
			{#if stockProducto}
				<p class="mb-4 text-sm text-slate-600">
					{stockProducto.nombre} · actual: <span class="font-semibold">{stockProducto.stock}</span>
				</p>
			{/if}
			<form
				onsubmit={(e) => {
					e.preventDefault();
					void guardarAjuste();
				}}
				class="space-y-3"
			>
				<label class="block">
					<span class="text-xs text-slate-600">Stock nuevo *</span>
					<input
						bind:value={sNuevo}
						type="number"
						min="0"
						class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
					/>
				</label>
				<label class="block">
					<span class="text-xs text-slate-600">Motivo *</span>
					<input
						bind:value={sMotivo}
						type="text"
						list="motivosComunes"
						placeholder="recuento, rotura, vencimiento…"
						class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
					/>
					<datalist id="motivosComunes">
						<option value="recuento"></option>
						<option value="rotura"></option>
						<option value="vencimiento"></option>
						<option value="error de carga"></option>
						<option value="devolución"></option>
					</datalist>
				</label>
				<label class="block">
					<span class="text-xs text-slate-600">Operador</span>
					<input
						bind:value={sOperador}
						type="text"
						placeholder="Opcional"
						class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
					/>
				</label>
				<div class="flex justify-end gap-2 pt-2">
					<button
						type="button"
						onclick={() => (modalStock = false)}
						class="rounded-lg border border-slate-300 px-4 py-2 text-sm"
					>
						Cancelar
					</button>
					<button
						type="submit"
						disabled={guardando}
						class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-700 disabled:opacity-50"
					>
						{guardando ? 'Guardando…' : 'Confirmar'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}