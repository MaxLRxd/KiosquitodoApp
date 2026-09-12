import type {
	Categoria,
	CategoriaNueva,
	MedioPago,
	NuevoProducto,
	NuevaVenta,
	Producto,
	Venta
} from '$lib/types';

/**
 * Backend simulado en memoria para desarrollo del frontend sin el runtime Tauri.
 * Espeja el contrato de los commands Rust (`buscar_producto_por_barcode`,
 * `listar_productos`, `registrar_venta`) incluyendo las reglas de negocio del
 * dominio (`validar_venta`: coherencia de medios de pago y stock).
 */

export function crearCategoria(id: number, nombre: string, color?: string): Categoria {
	return { id, nombre, color: color ?? null };
}

export function crearProducto(partial: Partial<Producto> & { id: number; nombre: string }): Producto {
	return {
		barcode: null,
		precio_venta: 0,
		precio_costo: null,
		stock: 0,
		stock_minimo: 0,
		activo: true,
		categoria_id: null,
		...partial
	};
}

export interface TiendaMock {
	productos: Producto[];
	categorias: Categoria[];
	ventas: Venta[];
	proximoIdProducto: number;
	proximoIdCategoria: number;
	proximoIdVenta: number;
}

export function crearTienda(): TiendaMock {
	return {
		productos: [
			crearProducto({
				id: 1,
				nombre: 'Coca-Cola 500ml',
				barcode: '7790000000016',
				precio_venta: 180000,
				precio_costo: 140000,
				stock: 6,
				stock_minimo: 4,
				categoria_id: 1
			}),
			crearProducto({
				id: 2,
				nombre: 'Agua sin gas 500ml',
				barcode: '7790000000023',
				precio_venta: 120000,
				precio_costo: 90000,
				stock: 10,
				stock_minimo: 6,
				categoria_id: 1
			}),
			crearProducto({
				id: 3,
				nombre: 'Alfajor Jorgito',
				barcode: '7790000000030',
				precio_venta: 140000,
				precio_costo: 100000,
				stock: 12,
				stock_minimo: 5,
				categoria_id: 2
			}),
			crearProducto({
				id: 4,
				nombre: 'Café instantáneo 50g',
				barcode: '7790000000047',
				precio_venta: 260000,
				precio_costo: 210000,
				stock: 3,
				stock_minimo: 4,
				categoria_id: 3
			})
		],
		categorias: [
			crearCategoria(1, 'Bebidas', '#3b82f6'),
			crearCategoria(2, 'Alfajores', '#f59e0b'),
			crearCategoria(3, 'Snacks', '#10b981')
		],
		ventas: [],
		proximoIdProducto: 5,
		proximoIdCategoria: 4,
		proximoIdVenta: 1
	};
}

export const tiendaMock: TiendaMock = crearTienda();

export function reiniciarTiendaMock(): void {
	const fresca = crearTienda();
	tiendaMock.productos = fresca.productos;
	tiendaMock.categorias = fresca.categorias;
	tiendaMock.ventas = fresca.ventas;
	tiendaMock.proximoIdProducto = fresca.proximoIdProducto;
	tiendaMock.proximoIdCategoria = fresca.proximoIdCategoria;
	tiendaMock.proximoIdVenta = fresca.proximoIdVenta;
}

/** `SELECT * FROM productos WHERE barcode = ?1 AND activo = 1`. */
export function mockBuscarProductoPorBarcode(barcode: string): Producto | null {
	const codigo = barcode?.trim() ?? '';
	return tiendaMock.productos.find((p) => p.activo && p.barcode === codigo) ?? null;
}

/**
 * `listar_productos(filtro, categoria_id, limite, offset)`:
 * filtro por nombre parcial (case-insensitive) o barcode; activos solamente.
 */
export function mockListarProductos(
	filtro: string,
	categoriaId: number | null = null,
	limite = 50,
	offset = 0
): Producto[] {
	const termino = filtro?.trim().toLowerCase() ?? '';
	const filtrados = tiendaMock.productos.filter((p) => {
		if (!p.activo) return false;
		if (categoriaId !== null && p.categoria_id !== categoriaId) return false;
		if (termino === '') return true;
		return (
			p.nombre.toLowerCase().includes(termino) ||
			(p.barcode?.toLowerCase().includes(termino) ?? false)
		);
	});
	// El repo SQLite ordena por nombre ascendente (producto_service).
	filtrados.sort((a, b) => a.nombre.localeCompare(b.nombre, 'es'));
	return filtrados.slice(offset, offset + Math.max(0, limite));
}

// ── Validaciones de dominio (espejo de domain/producto.rs) ────────────────

export function validarBarcode(codigo: string | null | undefined): boolean {
	const limpio = (codigo ?? '').trim();
	return (
		limpio.length === 8 || limpio.length === 12 || limpio.length === 13
	) && [...limpio].every((c) => c >= '0' && c <= '9');
}

/** Valida un `NuevoProducto`; devuelve el mensaje de la primera regla violada. */
export function validarNuevoProducto(nuevo: NuevoProducto): string | null {
	if (nuevo.nombre.trim() === '') return 'El nombre del producto no puede estar vacío';
	if (!validarBarcode(nuevo.barcode)) return 'El código de barras debe tener 8, 12 o 13 dígitos';
	if (nuevo.precio_venta <= 0) return 'El precio de venta debe ser mayor a cero (centavos)';
	if (nuevo.precio_costo !== null && nuevo.precio_costo < 0)
		return 'El precio de costo no puede ser negativo (centavos)';
	if (nuevo.stock_inicial < 0) return 'El stock inicial no puede ser negativo';
	if (nuevo.stock_minimo < 0) return 'El stock mínimo no puede ser negativo';
	return null;
}

/** Valida un `Producto` existente (mismas reglas que el de alta). */
export function validarProductoActualizado(producto: Producto): string | null {
	if (producto.nombre.trim() === '') return 'El nombre del producto no puede estar vacío';
	if (!validarBarcode(producto.barcode)) return 'El código de barras debe tener 8, 12 o 13 dígitos';
	if (producto.precio_venta <= 0) return 'El precio de venta debe ser mayor a cero (centavos)';
	if (producto.precio_costo !== null && producto.precio_costo < 0)
		return 'El precio de costo no puede ser negativo (centavos)';

if (producto.stock < 0) return 'El stock no puede ser negativo';
	if (producto.stock_minimo < 0) return 'El stock mínimo no puede ser negativo';
	return null;
}

// ── Operaciones de inventario (espejo de application/producto_service.rs) ──

/** `crear_producto(nuevo)`: valida reglas, rechaza barcode duplicado, persiste. */
export function mockCrearProducto(nuevo: NuevoProducto): Producto {
	const error = validarNuevoProducto(nuevo);
	if (error) throw new Error(error);

	const barcodeLimpio = (nuevo.barcode ?? '').trim();
	const duplicado = tiendaMock.productos.some((p) => p.barcode === barcodeLimpio);
	if (duplicado) throw new Error('Ya existe un producto con ese código de barras');

	const producto: Producto = {
		id: tiendaMock.proximoIdProducto++,
		nombre: nuevo.nombre.trim(),
		barcode: barcodeLimpio,
		precio_venta: nuevo.precio_venta,
		precio_costo: nuevo.precio_costo,
		stock: nuevo.stock_inicial,
		stock_minimo: nuevo.stock_minimo,
		activo: true,
		categoria_id: nuevo.categoria_id
	};
	tiendaMock.productos = [...tiendaMock.productos, producto];
	return producto;
}

/**
 * `actualizar_producto(producto)`: reemplaza los campos editables del producto.
 * Nota: el repo Rust actual tiene un bug conocido (no persiste `stock` en el
 * UPDATE). El mock sigue el contrato del servicio: el stock se conserva.
 */
export function mockActualizarProducto(producto: Producto): void {
	const error = validarProductoActualizado(producto);
	if (error) throw new Error(error);
	const indice = tiendaMock.productos.findIndex((p) => p.id === producto.id);
	if (indice === -1) throw new Error('El producto no existe');
	tiendaMock.productos[indice] = { ...tiendaMock.productos[indice], ...producto };
}

/** `eliminar_producto(id)`: baja lógica (activo = false), conserva historial. */
export function mockEliminarProducto(id: number): void {
	const producto = tiendaMock.productos.find((p) => p.id === id);
	if (!producto) throw new Error('El producto no existe');
	producto.activo = false;
}

/**
 * `ajustar_stock(id, stock_nuevo, motivo, operador)`: valida stock no negativo,
 * motivo obligatorio y producto existente; fija el stock nuevo.
 */
export function mockAjustarStock(
	id: number,
	stockNuevo: number,
	motivo: string,
	operador: string | null = null
): void {
	if (stockNuevo < 0) throw new Error('El nuevo stock no puede ser negativo');
	if (motivo.trim() === '') throw new Error('Debe indicar un motivo para el ajuste de stock');
	const producto = tiendaMock.productos.find((p) => p.id === id);
	if (!producto) throw new Error('El producto no existe');
	producto.stock = stockNuevo;
}

// ── Categorías ────────────────────────────────────────────────────────────

/** Lista todas las categorías (No existe ese command en el backend aún). */
export function mockListarCategorias(): Categoria[] {
	return [...tiendaMock.categorias].sort((a, b) => a.nombre.localeCompare(b.nombre, 'es'));
}

/** Crea una categoría validando nombre no vacío y sin duplicados. */
export function mockCrearCategoria(nueva: CategoriaNueva): Categoria {
	const nombre = nueva.nombre.trim();
	if (nombre === '') throw new Error('El nombre de la categoría no puede estar vacío');
	if (tiendaMock.categorias.some((c) => c.nombre.toLowerCase() === nombre.toLowerCase())) {
		throw new Error('Ya existe una categoría con ese nombre');
	}
	const categoria: Categoria = {
		id: tiendaMock.proximoIdCategoria++,
		nombre,
		color: nueva.color ?? null
	};
	tiendaMock.categorias = [...tiendaMock.categorias, categoria];
	return categoria;
}

function validarVenta(pedido: NuevaVenta): string | null {
	if (pedido.items.length === 0) return 'La venta debe tener al menos un ítem';
	const total = pedido.items.reduce((acc, it) => acc + it.cantidad * it.precio_unitario, 0);
	switch (pedido.medio_pago) {
		case 'efectivo':
			if (pedido.monto_mp !== 0) return 'Una venta en efectivo no puede tener monto de Mercado Pago';
			if (pedido.monto_efectivo !== total)
				return 'El monto en efectivo debe coincidir con el total de la venta';
			break;
		case 'mercado_pago':
			if (pedido.monto_efectivo !== 0)
				return 'Una venta con Mercado Pago no puede tener monto en efectivo';
			if (pedido.monto_mp !== total)
				return 'El monto de Mercado Pago debe coincidir con el total de la venta';
			break;
		case 'mixto':
			if (pedido.monto_efectivo <= 0 || pedido.monto_mp <= 0)
				return 'Efectivo y Mercado Pago deben ser mayores a cero';
			if (pedido.monto_efectivo + pedido.monto_mp !== total)
				return 'Efectivo + Mercado Pago debe coincidir con el total de la venta';
			break;
	}
	return null;
}

function horaLocalIso(): string {
	const ahora = new Date();
	const desfase = -ahora.getTimezoneOffset();
	const pad = (n: number) => String(n).padStart(2, '0');
	return `${ahora.getFullYear()}-${pad(ahora.getMonth() + 1)}-${pad(ahora.getDate())}T${pad(
		ahora.getHours()
	)}:${pad(ahora.getMinutes())}:${pad(ahora.getSeconds())}`;
}

/**
 * `registrar_venta(nueva)`: valida el pedido, verifica stock por ítem y
 * persiste la venta descontando el stock (mismo comportamiento de
 * `VentaService::registrar_venta`).
 */
export function mockRegistrarVenta(nueva: NuevaVenta): Venta {
	const error = validarVenta(nueva);
	if (error) throw new Error(error);

	for (const item of nueva.items) {
		const producto = tiendaMock.productos.find((p) => p.id === item.producto_id);
		if (!producto || !producto.activo) throw new Error(`Producto inexistente (id ${item.producto_id})`);
		if (producto.stock < item.cantidad) {
			throw new Error(
				`Stock insuficiente para ${producto.nombre} (disponible: ${producto.stock}, pedido: ${item.cantidad})`
			);
		}
	}

	for (const item of nueva.items) {
		const producto = tiendaMock.productos.find((p) => p.id === item.producto_id);
		if (producto) producto.stock -= item.cantidad;
	}

	const venta: Venta = {
		id: tiendaMock.proximoIdVenta++,
		total: nueva.items.reduce((acc, it) => acc + it.cantidad * it.precio_unitario, 0),
		medio_pago: nueva.medio_pago as MedioPago,
		monto_efectivo: nueva.monto_efectivo,
		monto_mp: nueva.monto_mp,
		descripcion_mp: nueva.descripcion_mp ?? null,
		hora: horaLocalIso()
	};
	tiendaMock.ventas = [...tiendaMock.ventas, venta];
	return venta;
}

/** Despacha un command Tauri simulado por su nombre. */
export function ejecutarMock<T>(comando: string, args: Record<string, unknown> = {}): Promise<T> {
	switch (comando) {
		case 'buscar_producto_por_barcode':
			return Promise.resolve(mockBuscarProductoPorBarcode(args.barcode as string) as T);
		case 'listar_productos':
			return Promise.resolve(
				mockListarProductos(
					args.filtro as string,
					(args.categoria_id as number | null | undefined) ?? null,
					args.limite as number,
					args.offset as number
				) as T
			);
		case 'crear_producto':
			return Promise.resolve(mockCrearProducto(args.nuevo as NuevoProducto) as T);
		case 'actualizar_producto':
			mockActualizarProducto(args.producto as Producto);
			return Promise.resolve(undefined as T);
		case 'eliminar_producto':
			mockEliminarProducto(args.id as number);
			return Promise.resolve(undefined as T);
		case 'ajustar_stock':
			mockAjustarStock(
				args.id as number,
				args.stock_nuevo as number,
				args.motivo as string,
				(args.operador as string | null | undefined) ?? null
			);
			return Promise.resolve(undefined as T);
		case 'listar_categorias':
			return Promise.resolve(mockListarCategorias() as T);
		case 'crear_categoria':
			return Promise.resolve(mockCrearCategoria(args.nueva as CategoriaNueva) as T);
		case 'registrar_venta':
			return Promise.resolve(mockRegistrarVenta(args.nueva as NuevaVenta) as T);
		default:
			return Promise.reject(new Error(`Comando sin implementación en el mock: ${comando}`));
	}
}