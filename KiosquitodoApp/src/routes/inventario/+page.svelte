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
			notificar(e, 'error', true);
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
			notificar(e, 'error', true);
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
			notificar(e, 'error', true);
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
			notificar(e, 'error', true);
		}
	}

	onMount(() => {
		void cargar();
		void listarCategorias().then((c) => (categorias = c));
	});
</script>

<div class="page flex h-full flex-col">
	<div class="page-header">
		<div>
			<p class="eyebrow">Catálogo</p>
			<h1 class="page-title mt-1">Inventario</h1>
			<p class="page-subtitle">
				Productos, precios y stock. El stock se modifica únicamente con el ajuste auditado.
			</p>
		</div>
		<div class="flex items-center gap-2">
			<span class="badge-neutral">{productos.length} producto(s)</span>
			<button onclick={abrirNuevo} class="btn-primary">+ Nuevo producto</button>
		</div>
	</div>

	<div class="card mt-5 flex flex-wrap items-center gap-3 p-3.5">
		<div class="relative min-w-64 flex-1">
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
				bind:value={filtro}
				oninput={cargar}
				type="text"
				placeholder="Buscar por nombre o código de barras…"
				class="input !pl-11"
			/>
		</div>
		<select bind:value={categoriaId} onchange={cargar} class="select w-full sm:w-56">
			<option value={null}>Todas las categorías</option>
			{#each categorias as categoria (categoria.id)}
				<option value={categoria.id}>{categoria.nombre}</option>
			{/each}
		</select>
	</div>

	<div class="table-shell mt-5 flex min-h-0 flex-1 flex-col">
		<div class="min-h-0 flex-1 overflow-auto">
			<table class="table">
				<thead class="sticky top-0 z-10 backdrop-blur">
					<tr>
						<th>Producto</th>
						<th>Código</th>
						<th>Categoría</th>
						<th class="text-right">P. venta</th>
						<th class="text-right">Costo</th>
						<th class="text-right">Stock</th>
						<th class="text-right">Mínimo</th>
						<th>Acciones</th>
					</tr>
				</thead>
				<tbody>
					{#if cargando}
						<tr>
							<td colspan="8" class="py-12 text-center text-slate-400">Cargando…</td>
						</tr>
					{:else if paginados.length === 0}
						<tr>
							<td colspan="8" class="py-12 text-center text-slate-400">
								No hay productos con ese filtro.
							</td>
						</tr>
					{:else}
						{#each paginados as p (p.id)}
							<tr>
								<td class="font-semibold text-slate-800">
									<div class="flex items-center gap-2">
										<span>{p.nombre}</span>
										{#if estadoStock(p) === 'agotado'}
											<span class="badge-danger">Agotado</span>
										{:else if estadoStock(p) === 'bajo'}
											<span class="badge-warn">Stock bajo</span>
										{/if}
									</div>
								</td>
								<td class="font-mono text-xs text-slate-500">{p.barcode ?? '—'}</td>
								<td class="text-slate-600">{nombreCategoria(p.categoria_id)}</td>
								<td class="text-right font-medium tabular-nums">
									{formatearPesos(p.precio_venta)}
								</td>
								<td class="text-right tabular-nums text-slate-600">
									{p.precio_costo !== null ? formatearPesos(p.precio_costo) : '—'}
								</td>
								<td class="text-right font-bold tabular-nums text-slate-900">{p.stock}</td>
								<td class="text-right tabular-nums text-slate-500">{p.stock_minimo}</td>
								<td>
									<div class="flex gap-1.5">
										<button onclick={() => abrirEditar(p)} class="btn-secondary btn-sm">
											Editar
										</button>
										<button onclick={() => abrirAjuste(p)} class="btn-secondary btn-sm">
											Stock
										</button>
										<button
											onclick={() => eliminar(p)}
											class="btn-sm rounded-lg border border-rose-200 bg-white text-rose-600 transition hover:bg-rose-50"
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
	</div>

	<div class="mt-3 flex flex-wrap items-center justify-between gap-3 text-sm text-slate-600">
		<span>{productos.length} producto(s) en total</span>
		<div class="flex items-center gap-2">
			<button
				onclick={() => (pagina = Math.max(1, pagina - 1))}
				disabled={pagina <= 1}
				class="btn-secondary btn-sm"
			>
				‹ Anterior
			</button>
			<span class="px-2 text-xs font-semibold text-slate-500">
				Página {pagina} de {paginas}
			</span>
			<button
				onclick={() => (pagina = Math.min(paginas, pagina + 1))}
				disabled={pagina >= paginas}
				class="btn-secondary btn-sm"
			>
				Siguiente ›
			</button>
		</div>
	</div>
</div>

{#if modalProducto}
	<div class="modal-overlay">
		<button
			type="button"
			aria-label="Cerrar"
			class="absolute inset-0"
			onclick={() => (modalProducto = false)}
		></button>
		<div class="modal-panel max-w-md">
			<div class="mb-5">
				<p class="eyebrow">Inventario</p>
				<h2 class="modal-title mt-1">
					{editandoId === null ? 'Nuevo producto' : 'Editar producto'}
				</h2>
			</div>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					void guardarProducto();
				}}
				class="space-y-4"
			>
				<label class="label">
					Nombre *
					<input bind:value={fNombre} type="text" class="input" />
				</label>
				<label class="label">
					Código de barras (8, 12 o 13 dígitos) *
					<input bind:value={fBarcode} type="text" inputmode="numeric" class="input" />
				</label>
				<div class="grid grid-cols-2 gap-3">
					<label class="label">
						Precio de venta ($) *
						<input bind:value={fPrecioVenta} type="text" inputmode="decimal" class="input" />
					</label>
					<label class="label">
						Precio de costo ($)
						<input bind:value={fPrecioCosto} type="text" inputmode="decimal" class="input" />
					</label>
				</div>
				<div class="grid grid-cols-2 gap-3">
					{#if editandoId === null}
						<label class="label">
							Stock inicial
							<input bind:value={fStock} type="number" min="0" class="input" />
						</label>
					{/if}
					<label class="label">
						Stock mínimo de alerta
						<input bind:value={fStockMinimo} type="number" min="0" class="input" />
					</label>
				</div>
				{#if editandoId !== null}
					<p class="rounded-xl bg-slate-50 px-3.5 py-2.5 text-xs leading-relaxed text-slate-500">
						El stock se modifica con la acción «Stock» (ajuste auditado con motivo).
					</p>
				{/if}
				<label class="label">
					Categoría
					<select bind:value={fCategoriaId} class="select">
						<option value={null}>Sin categoría</option>
						{#each categorias as categoria (categoria.id)}
							<option value={categoria.id}>{categoria.nombre}</option>
						{/each}
					</select>
				</label>

				{#if mostrarNuevaCategoria}
					<div class="rounded-xl border border-slate-200 bg-slate-50/60 p-3">
						<p class="text-xs font-bold uppercase tracking-wide text-slate-500">
							Nueva categoría
						</p>
						<div class="mt-2 flex gap-2">
							<input
								bind:value={fCatNombre}
								type="text"
								placeholder="Nombre"
								class="input flex-1"
							/>
							<input
								bind:value={fCatColor}
								type="color"
								class="h-10 w-12 rounded-lg border border-slate-200 bg-white p-1"
							/>
							<button onclick={() => void guardarCategoria()} type="button" class="btn-primary btn-sm">
								Guardar
							</button>
						</div>
					</div>
				{:else}
					<button
						type="button"
						onclick={() => (mostrarNuevaCategoria = true)}
						class="text-xs font-semibold text-indigo-600 hover:text-indigo-700 hover:underline"
					>
						+ Nueva categoría
					</button>
				{/if}

				<div class="flex justify-end gap-2 border-t border-slate-100 pt-4">
					<button type="button" onclick={() => (modalProducto = false)} class="btn-secondary">
						Cancelar
					</button>
					<button type="submit" disabled={guardando} class="btn-primary">
						{guardando ? 'Guardando…' : 'Guardar'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

{#if modalStock}
	<div class="modal-overlay">
		<button
			type="button"
			aria-label="Cerrar"
			class="absolute inset-0"
			onclick={() => (modalStock = false)}
		></button>
		<div class="modal-panel max-w-sm">
			<div class="mb-4">
				<p class="eyebrow">Auditoría</p>
				<h2 class="modal-title mt-1">Ajustar stock</h2>
				{#if stockProducto}
					<p class="mt-1 text-sm text-slate-500">
						{stockProducto.nombre} · actual:
						<span class="font-bold text-slate-800">{stockProducto.stock}</span>
					</p>
				{/if}
			</div>
			<form
				onsubmit={(e) => {
					e.preventDefault();
					void guardarAjuste();
				}}
				class="space-y-4"
			>
				<label class="label">
					Stock nuevo *
					<input bind:value={sNuevo} type="number" min="0" class="input" />
				</label>
				<label class="label">
					Motivo *
					<input
						bind:value={sMotivo}
						type="text"
						list="motivosComunes"
						placeholder="recuento, rotura, vencimiento…"
						class="input"
					/>
					<datalist id="motivosComunes">
						<option value="recuento"></option>
						<option value="rotura"></option>
						<option value="vencimiento"></option>
						<option value="error de carga"></option>
						<option value="devolución"></option>
					</datalist>
				</label>
				<label class="label">
					Operador
					<input bind:value={sOperador} type="text" placeholder="Opcional" class="input" />
				</label>
				<div class="flex justify-end gap-2 border-t border-slate-100 pt-4">
					<button type="button" onclick={() => (modalStock = false)} class="btn-secondary">
						Cancelar
					</button>
					<button type="submit" disabled={guardando} class="btn-success">
						{guardando ? 'Guardando…' : 'Confirmar'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}