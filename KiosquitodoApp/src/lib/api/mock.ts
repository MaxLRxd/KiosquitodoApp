import type {
	Categoria,
	CategoriaNueva,
	ConfiguracionApp,
	CierrePersistido,
	DevolucionRegistrada,
	EstadoActualizacion,
	InfoActualizacion,
	ItemVenta,
	MedioPago,
	MpPago,
	NuevoProducto,
	NuevaVenta,
	Producto,
	ResumenCierre,
	RutasApp,
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

export function crearVentaMock(
	partial: Partial<Venta> & { id: number; items: ItemVenta[]; total: number; hora: string }
): Venta {
	return {
		medio_pago: 'efectivo' as MedioPago,
		monto_efectivo: 0,
		monto_mp: 0,
		descripcion_mp: null,
		...partial
	};
}

export function crearCierre(id: number, fecha: string, resumen: ResumenCierre): CierrePersistido {
	return { id, fecha, resumen };
}

export interface TiendaMock {
	productos: Producto[];
	categorias: Categoria[];
	ventas: Venta[];
	/** Pagos MP importados desde la API (integración diferida: queda vacío en el prototipo). */
	pagosMp: MpPago[];
	cierres: CierrePersistido[];
	devoluciones: DevolucionRegistrada[];
	proximoIdProducto: number;
	proximoIdCategoria: number;
	proximoIdVenta: number;
	proximoIdCierre: number;
	proximoIdDevolucion: number;
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
		pagosMp: [],
		cierres: [],
		devoluciones: [],
		proximoIdProducto: 5,
		proximoIdCategoria: 4,
		proximoIdVenta: 1,
		proximoIdCierre: 1,
		proximoIdDevolucion: 1
	};
}

const configuracionPorDefecto = (): ConfiguracionApp => ({
	sync_intervalo_minutos: 15,
	inactividad_minutos: 5,
	sonidos_activos: true,
	backup_dir: 'mock://KioscoBackups',
	actualizaciones_url:
		'https://github.com/MaxLRxd/KiosquitodoApp/releases/latest/download/latest.json',
	actualizaciones_auto: true
});

let configuracionMock: ConfiguracionApp = configuracionPorDefecto();

/** `obtener_configuracion()`: copia para que el caller no mute el estado mock. */
export function mockObtenerConfiguracion(): ConfiguracionApp {
	return { ...configuracionMock };
}

/** `guardar_configuracion()`: valida y reemplaza las preferencias del mock. */
export function mockGuardarConfiguracion(configuracion: ConfiguracionApp): ConfiguracionApp {
	if (configuracion.sync_intervalo_minutos < 1) {
		throw new Error('El intervalo de sync debe ser al menos 1 minuto');
	}
	if (!configuracion.backup_dir.trim()) {
		throw new Error('La carpeta de backups no puede estar vacía');
	}
	const actualizacionesUrl = configuracion.actualizaciones_url.trim();
	if (
		actualizacionesUrl &&
		!actualizacionesUrl.startsWith('https://') &&
		!actualizacionesUrl.startsWith('http://')
	) {
		throw new Error('La URL de actualizaciones debe empezar con https:// o http://');
	}
	configuracionMock = {
		sync_intervalo_minutos: configuracion.sync_intervalo_minutos,
		inactividad_minutos: configuracion.inactividad_minutos,
		sonidos_activos: configuracion.sonidos_activos,
		backup_dir: configuracion.backup_dir.trim(),
		actualizaciones_url: actualizacionesUrl,
		actualizaciones_auto: configuracion.actualizaciones_auto
	};
	return { ...configuracionMock };
}

// El mock guarda el PIN en claro a propósito: solo reproduce el contrato IPC.
// El backend real guarda `hash(salt || pin)` y nunca devuelve el valor.
let pinMock: string | null = null;

function validarPinMock(pin: string): void {
	if (!/^\d{4,6}$/.test(pin)) {
		throw new Error('El PIN debe tener entre 4 y 6 dígitos');
	}
}

/** `pin_configurado()`: solo booleano, como el command real. */
export function mockPinConfigurado(): boolean {
	return pinMock !== null;
}

/** `establecer_pin()`: valida y devuelve el estado ya configurado. */
export function mockEstablecerPin(pin: string): boolean {
	validarPinMock(pin);
	pinMock = pin;
	return true;
}

/** `verificar_pin()`: compara con el PIN del mock; sin PIN devuelve false. */
export function mockVerificarPin(pin: string): boolean {
	return pinMock !== null && pinMock === pin;
}

/** `eliminar_pin()`: desactiva el bloqueo. */
export function mockEliminarPin(): void {
	pinMock = null;
}

// Igual que el PIN: el mock guarda el token en memoria a propósito. El backend
// real usa el keyring cifrado y nunca lo devuelve al frontend.
let tokenMpMock: string | null = null;

/** `token_mp_configurado()`: solo booleano. */
export function mockTokenMpConfigurado(): boolean {
	return tokenMpMock !== null;
}

/** `guardar_token_mp()`: valida no vacío y devuelve el estado. */
export function mockGuardarTokenMp(token: string): boolean {
	const limpio = token.trim();
	if (!limpio) {
		throw new Error('El token de Mercado Pago no puede estar vacío');
	}
	tokenMpMock = limpio;
	return true;
}

/** `eliminar_token_mp()`: idempotente. */
export function mockEliminarTokenMp(): void {
	tokenMpMock = null;
}

/** `seleccionar_carpeta_backups()`: carpeta simulada distinta de la default. */
export function mockSeleccionarCarpetaBackups(): string {
	return 'mock://BackupsElegidos';
}

/** `estado_actualizacion()`: versión y endpoint del mock. */
export function mockEstadoActualizacion(): EstadoActualizacion {
	const endpoint = configuracionMock.actualizaciones_url.trim();
	return {
		version_actual: '0.1.0',
		endpoint,
		endpoint_configurado: endpoint !== '',
		auto_verificar: configuracionMock.actualizaciones_auto
	};
}

/** `buscar_actualizacion()`: el navegador siempre está "al día". */
export function mockBuscarActualizacion(): InfoActualizacion | null {
	return null;
}

/** `instalar_actualizacion()`: solo tiene sentido con el runtime Tauri. */
export function mockInstalarActualizacion(): string {
	throw new Error('Las actualizaciones solo están disponibles en la app de escritorio');
}

// El flag simula el chequeo del arranque: primero crea el backup del día y
// las siguientes llamadas devuelven null hasta que se reinicia el mock.
let backupDiarioMockCreado = false;

/** `verificar_backup_diario()`: ruta creada la primera vez, luego `null`. */
export function mockVerificarBackupDiario(): string | null {
	if (backupDiarioMockCreado) return null;
	backupDiarioMockCreado = true;
	const dia = horaLocalIso().replace(/[-:T]/g, '').slice(0, 8);
	return `${configuracionMock.backup_dir}/kiosco_${dia}_000000.db`;
}

export const tiendaMock: TiendaMock = crearTienda();

export function reiniciarTiendaMock(): void {
	const fresca = crearTienda();
	tiendaMock.productos = fresca.productos;
	tiendaMock.categorias = fresca.categorias;
	tiendaMock.ventas = fresca.ventas;
	tiendaMock.pagosMp = fresca.pagosMp;
	tiendaMock.cierres = fresca.cierres;
	tiendaMock.devoluciones = fresca.devoluciones;
	tiendaMock.proximoIdProducto = fresca.proximoIdProducto;
	tiendaMock.proximoIdCategoria = fresca.proximoIdCategoria;
	tiendaMock.proximoIdVenta = fresca.proximoIdVenta;
	tiendaMock.proximoIdCierre = fresca.proximoIdCierre;
	tiendaMock.proximoIdDevolucion = fresca.proximoIdDevolucion;
	configuracionMock = configuracionPorDefecto();
	pinMock = null;
	tokenMpMock = null;
	backupDiarioMockCreado = false;
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
 * El stock NO se edita aquí (solo por «Ajustar stock», flujo auditado en
 * `movimientos_stock`); se conserva el valor actual, igual que el repo Rust
 * cuyo UPDATE no toca la columna `stock`.
 */
export function mockActualizarProducto(producto: Producto): void {
	const error = validarProductoActualizado(producto);
	if (error) throw new Error(error);
	const indice = tiendaMock.productos.findIndex((p) => p.id === producto.id);
	if (indice === -1) throw new Error('El producto no existe');
	const actual = tiendaMock.productos[indice];
	tiendaMock.productos[indice] = { ...actual, ...producto, stock: actual.stock };
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
		items: nueva.items.map((it) => ({ ...it })),
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

// ── Cierre de caja (espejo de application/cierre_service.rs) ───────────────

/** Valida fecha `YYYY-MM-DD`; devuelve el mensaje de error o `null`. */
export function validarFecha(fecha: string): string | null {
	const f = fecha.trim();
	const valida =
		f.length === 10 &&
		f[4] === '-' &&
		f[7] === '-' &&
		/^\d{4}$/.test(f.slice(0, 4)) &&
		/^\d{2}$/.test(f.slice(5, 7)) &&
		/^\d{2}$/.test(f.slice(8, 10));
	return valida ? null : 'La fecha debe tener formato YYYY-MM-DD';
}

/**
 * `resumen_diario(fecha)`: totales del día idénticos a la query de
 * `CierreSqlite::resumen_dia` (Σ ventas por `date(hora)`, margen por ítem
 * con costo ACTUAL del producto, delta MP = Σ pagos API − Σ ventas MP).
 */
export function mockResumenDiario(fecha: string): ResumenCierre {
	const error = validarFecha(fecha);
	if (error) throw new Error(error);
	const dia = fecha.trim();

	const ventasDelDia = tiendaMock.ventas.filter((v) => v.hora.startsWith(dia));

	const total_efectivo = ventasDelDia.reduce((acc, v) => acc + v.monto_efectivo, 0);
	const total_mp = ventasDelDia.reduce((acc, v) => acc + v.monto_mp, 0);
	const total_ventas = ventasDelDia.reduce((acc, v) => acc + v.total, 0);
	const cantidad_ventas = ventasDelDia.length;

	// Margen bruto: (precio_unitario − COALESCE(precio_costo, 0)) × cantidad,
	// con el costo ACTUAL del producto (mismo JOIN del SQL).
	let margen_bruto = 0;
	for (const v of ventasDelDia) {
		for (const item of v.items) {
			const costo = tiendaMock.productos.find((p) => p.id === item.producto_id)?.precio_costo ?? null;
			margen_bruto += (item.precio_unitario - (costo ?? 0)) * item.cantidad;
		}
	}

	// Delta MP: Σ pagos acreditados según API − Σ ventas MP registradas,
	// solo ventas con medio de pago MP o mixto (misma query del backend).
	const montoAcreditado = tiendaMock.pagosMp
		.filter((p) => p.fecha_aprobacion?.startsWith(dia))
		.reduce((acc, p) => acc + p.monto, 0);
	const montoRegistrado = ventasDelDia
		.filter((v) => v.medio_pago === 'mercado_pago' || v.medio_pago === 'mixto')
		.reduce((acc, v) => acc + v.monto_mp, 0);
	const delta_mp = montoAcreditado - montoRegistrado;

	return {
		total_efectivo,
		total_mp,
		total_ventas,
		cantidad_ventas,
		margen_bruto,
		delta_mp
	};
}

/**
 * `cerrar_caja(fecha)`: consolida el resumen, lo persiste y devuelve el cierre.
 * No borra ventas ni bloquea el día (puede repetirse, como el backend).
 */
export function mockCerrarCaja(fecha: string): CierrePersistido {
	const dia = fecha.trim();
	const resumen = mockResumenDiario(dia);
	const cierre = crearCierre(tiendaMock.proximoIdCierre++, dia, resumen);
	tiendaMock.cierres = [...tiendaMock.cierres, cierre];
	return cierre;
}

/** `listar_cierres()`: histórico ordenado por fecha descendente (`ORDER BY fecha DESC`). */
export function mockListarCierres(): CierrePersistido[] {
	return [...tiendaMock.cierres].sort((a, b) => b.fecha.localeCompare(a.fecha));
}

/**
 * `exportar_cierres_csv(desde, hasta)`: CSV con encabezado y montos en
 * centavos; bordes vacíos = sin filtro por ese lado (mismo rango del backend).
 */
export function mockExportarCierresCsv(desde: string, hasta: string): string {
	const lineas = ['fecha,total_efectivo,total_mp,total_ventas,cantidad_ventas,margen_bruto,delta_mp'];
	for (const c of mockListarCierres()) {
		if (desde.trim() !== '' && c.fecha < desde.trim()) continue;
		if (hasta.trim() !== '' && c.fecha > hasta.trim()) continue;
		const r = c.resumen;
		lineas.push(
			`${c.fecha},${r.total_efectivo},${r.total_mp},${r.total_ventas},${r.cantidad_ventas},${r.margen_bruto},${r.delta_mp}`
		);
	}
	return lineas.join('\n');
}

// ── Lectura de ventas y devoluciones (espejo de application/venta_service.rs) ─

/** `obtener_venta(id)`: devuelve la venta con sus ítems, o `null`. */
export function mockObtenerVenta(id: number): Venta | null {
	return tiendaMock.ventas.find((v) => v.id === id) ?? null;
}

/** `listar_ventas_dia(fecha)`: ventas del día (`date(hora) = ?1`), más recientes primero. */
export function mockListarVentasDia(fecha: string): Venta[] {
	return tiendaMock.ventas
		.filter((v) => v.hora.startsWith(fecha))
		.sort((a, b) => b.hora.localeCompare(a.hora));
}

/**
 * `registrar_devolucion(venta_id, items, motivo)` con las validaciones del
 * servicio (venta existente, al menos un ítem, cantidades positivas); repone
 * stock por ítem y registra la devolución. El repo Rust no valida que los
 * ítems pertenezcan a la venta original ni topes de cantidad: el mock lo
 * espeja igual.
 */
export function mockRegistrarDevolucion(
	ventaId: number,
	items: ItemVenta[],
	motivo: string | null = null
): DevolucionRegistrada {
	if (!tiendaMock.ventas.some((v) => v.id === ventaId)) throw new Error('La venta no existe');
	if (items.length === 0) throw new Error('La devolución debe tener al menos un ítem');
	if (items.some((i) => i.cantidad <= 0))
		throw new Error('La cantidad devuelta de cada ítem debe ser mayor a cero');

	for (const item of items) {
		const producto = tiendaMock.productos.find((p) => p.id === item.producto_id);
		if (producto) producto.stock += item.cantidad;
	}

	const devolucion: DevolucionRegistrada = {
		id: tiendaMock.proximoIdDevolucion++,
		venta_id: ventaId,
		fecha: horaLocalIso(),
		motivo: motivo?.trim() || null,
		items: items.map((it) => ({ ...it }))
	};
	tiendaMock.devoluciones = [...tiendaMock.devoluciones, devolucion];
	return devolucion;
}

/**
 * `listar_devoluciones()`: devoluciones recientes, más nuevas primero.
 * NOTA: el backend no expone este command aún (solo `registrar_devolucion`);
 * es mock-only hasta definir el command de listado.
 */
export function mockListarDevoluciones(): DevolucionRegistrada[] {
	return [...tiendaMock.devoluciones].sort((a, b) => b.fecha.localeCompare(a.fecha));
}

/** `obtener_rutas()`: rutas de demostración cuando no hay runtime Tauri. */
export function mockObtenerRutas(): RutasApp {
	return {
		data_dir: 'mock://datos-pruebas',
		backup_dir: configuracionMock.backup_dir,
		log_dir: 'mock://KiosquitodoApp/logs',
		db_path: 'mock://datos-pruebas/kiosco.db'
	};
}

/** `crear_backup_manual()`: ruta simulada con sello de fecha/hora local. */
export function mockCrearBackupManual(): string {
	const sello = horaLocalIso().replace(/[-:T]/g, '');
	return `${configuracionMock.backup_dir}/kiosco_${sello}.db`;
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
		case 'resumen_diario':
			return Promise.resolve(mockResumenDiario(args.fecha as string) as T);
		case 'cerrar_caja':
			return Promise.resolve(mockCerrarCaja(args.fecha as string) as T);
		case 'listar_cierres':
			return Promise.resolve(mockListarCierres() as T);
		case 'exportar_cierres_csv':
			return Promise.resolve(
				mockExportarCierresCsv(args.desde as string, args.hasta as string) as T
			);
		case 'obtener_venta':
			return Promise.resolve(mockObtenerVenta(args.id as number) as T);
		case 'listar_ventas_dia':
			return Promise.resolve(mockListarVentasDia(args.fecha as string) as T);
		case 'registrar_devolucion':
			mockRegistrarDevolucion(
				args.venta_id as number,
				args.items as ItemVenta[],
				(args.motivo as string | null | undefined) ?? null
			);
			return Promise.resolve(undefined as T);
		case 'listar_devoluciones':
			return Promise.resolve(mockListarDevoluciones() as T);
		case 'obtener_rutas':
			return Promise.resolve(mockObtenerRutas() as T);
		case 'crear_backup_manual':
			return Promise.resolve(mockCrearBackupManual() as T);
		case 'verificar_backup_diario':
			return Promise.resolve(mockVerificarBackupDiario() as T);
		case 'obtener_configuracion':
			return Promise.resolve(mockObtenerConfiguracion() as T);
		case 'guardar_configuracion':
			return Promise.resolve(
				mockGuardarConfiguracion(args.configuracion as ConfiguracionApp) as T
			);
		case 'pin_configurado':
			return Promise.resolve(mockPinConfigurado() as T);
		case 'establecer_pin':
			return Promise.resolve(mockEstablecerPin(args.pin as string) as T);
		case 'verificar_pin':
			return Promise.resolve(mockVerificarPin(args.pin as string) as T);
		case 'eliminar_pin':
			mockEliminarPin();
			return Promise.resolve(undefined as T);
		case 'token_mp_configurado':
			return Promise.resolve(mockTokenMpConfigurado() as T);
		case 'guardar_token_mp':
			return Promise.resolve(mockGuardarTokenMp(args.token as string) as T);
		case 'eliminar_token_mp':
			mockEliminarTokenMp();
			return Promise.resolve(undefined as T);
		case 'seleccionar_carpeta_backups':
			return Promise.resolve(mockSeleccionarCarpetaBackups() as T);
		case 'estado_actualizacion':
			return Promise.resolve(mockEstadoActualizacion() as T);
		case 'buscar_actualizacion':
			return Promise.resolve(mockBuscarActualizacion() as T);
		case 'instalar_actualizacion':
			return Promise.resolve(mockInstalarActualizacion() as T);
		default:
			return Promise.reject(new Error(`Comando sin implementación en el mock: ${comando}`));
	}
}