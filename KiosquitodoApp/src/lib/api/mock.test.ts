import { describe, expect, it, beforeEach } from 'vitest';
import {
	ejecutarMock,
	mockActualizarProducto,
	mockAjustarStock,
	mockBuscarProductoPorBarcode,
	mockCerrarCaja,
	mockCrearCategoria,
	mockCrearProducto,
	mockEliminarProducto,
	mockExportarCierresCsv,
	mockListarCategorias,
	mockListarCierres,
	mockListarProductos,
	mockRegistrarVenta,
	mockResumenDiario,
	reiniciarTiendaMock,
	tiendaMock,
	validarBarcode
} from './mock';
import type { NuevoProducto, NuevaVenta } from '$lib/types';

beforeEach(() => {
	reiniciarTiendaMock();
});

describe('mock: buscar producto por barcode', () => {
	it('encuentra un producto activo por su código', () => {
		const p = mockBuscarProductoPorBarcode('7790000000016');
		expect(p?.nombre).toBe('Coca-Cola 500ml');
	});

	it('limita en blancos y devuelve null si no existe o está inactivo', () => {
		expect(mockBuscarProductoPorBarcode(' 7790000000016 ')).not.toBeNull();
		expect(mockBuscarProductoPorBarcode('9999999999999')).toBeNull();
		tiendaMock.productos[0].activo = false;
		expect(mockBuscarProductoPorBarcode('7790000000016')).toBeNull();
	});
});

describe('mock: listar productos', () => {
	it('filtra por nombre parcial sin distinguir mayúsculas', () => {
		const resultados = mockListarProductos('agua', null, 50, 0);
		expect(resultados.map((p) => p.nombre)).toEqual(['Agua sin gas 500ml']);
	});

	it('filtra por categoría', () => {
		const resultados = mockListarProductos('', 2, 50, 0);
		expect(resultados.map((p) => p.nombre)).toEqual(['Alfajor Jorgito']);
	});

	it('respeta límite y offset (paginación)', () => {
		expect(mockListarProductos('', null, 2, 0)).toHaveLength(2);
		expect(mockListarProductos('', null, 2, 2)).toHaveLength(2);
	});

	it('ordena por nombre ascendente', () => {
		const nombres = mockListarProductos('', null, 50, 0).map((p) => p.nombre);
		expect(nombres).toEqual([...nombres].sort((a, b) => a.localeCompare(b, 'es')));
	});

	it('no lista productos inactivos', () => {
		tiendaMock.productos[1].activo = false;
		const resultados = mockListarProductos('', null, 50, 0);
		expect(resultados.some((p) => p.id === 2)).toBe(false);
	});
});

describe('mock: registrar venta', () => {
	function pedido(overrides: Partial<NuevaVenta> = {}): NuevaVenta {
		return {
			items: [{ producto_id: 1, cantidad: 2, precio_unitario: 180000 }],
			medio_pago: 'efectivo',
			monto_efectivo: 360000,
			monto_mp: 0,
			descripcion_mp: null,
			...overrides
		};
	}

	it('registra una venta en efectivo, asigna id/hora y descuenta stock', () => {
		const stockPrevio = tiendaMock.productos[0].stock;
		const venta = mockRegistrarVenta(pedido());
		expect(venta.id).toBe(1);
		expect(venta.total).toBe(360000);
		expect(venta.medio_pago).toBe('efectivo');
		expect(venta.items).toEqual([{ producto_id: 1, cantidad: 2, precio_unitario: 180000 }]);
		expect(venta.hora).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$/);
		expect(tiendaMock.productos[0].stock).toBe(stockPrevio - 2);
		expect(tiendaMock.ventas).toHaveLength(1);
	});

	it('rechaza una venta sin ítems', () => {
		expect(() => mockRegistrarVenta(pedido({ items: [] }))).toThrow(/al menos un ítem/);
	});

	it('rechaza monto en efectivo distinto del total', () => {
		expect(() => mockRegistrarVenta(pedido({ monto_efectivo: 100000 }))).toThrow(
			/El monto en efectivo debe coincidir/
		);
	});

	it('rechaza una venta en efectivo con monto de Mercado Pago', () => {
		expect(() =>
			mockRegistrarVenta(pedido({ monto_efectivo: 360000, monto_mp: 100 }))
		).toThrow(/no puede tener monto de Mercado Pago/);
	});

	it('rechaza una venta en MP con monto en efectivo', () => {
		expect(() =>
			mockRegistrarVenta(pedido({ medio_pago: 'mercado_pago', monto_mp: 360000, monto_efectivo: 100 }))
		).toThrow(/no puede tener monto en efectivo/);
	});

	it('acepta una venta mixta cuyos montos suman el total', () => {
		const venta = mockRegistrarVenta(
			pedido({
				medio_pago: 'mixto',
				monto_efectivo: 200000,
				monto_mp: 160000
			})
		);
		expect(venta.medio_pago).toBe('mixto');
		expect(venta.monto_efectivo).toBe(200000);
		expect(venta.monto_mp).toBe(160000);
	});

	it('rechaza una venta mixta cuya suma no coincide con el total', () => {
		expect(() =>
			mockRegistrarVenta(
				pedido({ medio_pago: 'mixto', monto_efectivo: 200000, monto_mp: 100000 })
			)
		).toThrow(/debe coincidir con el total/);
	});

	it('rechaza stock insuficiente y no persiste nada', () => {
		const stockPrevio = tiendaMock.productos[0].stock;
		const cantidad = stockPrevio + 5;
		const precioUnitario = 180000;
		expect(() =>
			mockRegistrarVenta(
				pedido({
					items: [{ producto_id: 1, cantidad, precio_unitario: precioUnitario }],
					monto_efectivo: cantidad * precioUnitario
				})
			)
		).toThrow(/Stock insuficiente/);
		expect(tiendaMock.productos[0].stock).toBe(stockPrevio);
		expect(tiendaMock.ventas).toHaveLength(0);
	});

	it('despacha por nombre de command vía ejecutarMock', async () => {
		const producto = await ejecutarMock<{ nombre: string } | null>('buscar_producto_por_barcode', {
			barcode: '7790000000023'
		});
		expect(producto?.nombre).toBe('Agua sin gas 500ml');

		const venta = await ejecutarMock<{ id: number }>('registrar_venta', { nueva: pedido() });
		expect(venta.id).toBeGreaterThan(0);

		await expect(ejecutarMock('comando_inexistente', {})).rejects.toThrow(/sin implementación/);
	});
});

describe('mock: validar barcode', () => {
	it('acepta EAN-8 (8), UPC-A (12) y EAN-13 (13) dígitos', () => {
		expect(validarBarcode('12345678')).toBe(true);
		expect(validarBarcode('123456789012')).toBe(true);
		expect(validarBarcode('7790001234567')).toBe(true);
	});

	it('rechaza vacío, longitud incorrecta o no numérico', () => {
		expect(validarBarcode('')).toBe(false);
		expect(validarBarcode('123')).toBe(false);
		expect(validarBarcode('123456789012345')).toBe(false);
		expect(validarBarcode('abcdefghijk')).toBe(false);
		expect(validarBarcode(null)).toBe(false);
	});
});

describe('mock: CRUD de productos', () => {
	function nuevo(overrides: Partial<NuevoProducto> = {}): NuevoProducto {
		return {
			nombre: 'Chocolate 70g',
			barcode: '7791112223334',
			precio_venta: 150000,
			precio_costo: 105000,
			stock_inicial: 8,
			stock_minimo: 3,
			categoria_id: 2,
			...overrides
		};
	}

	it('crea un producto y asigna id/secuencia', () => {
		const creado = mockCrearProducto(nuevo());
		expect(creado.id).toBe(5);
		expect(creado.stock).toBe(8);
		expect(creado.activo).toBe(true);
		expect(tiendaMock.productos).toHaveLength(5);
	});

	it('rechaza datos inválidos por cada regla', () => {
		expect(() => mockCrearProducto(nuevo({ nombre: '  ' }))).toThrow(/nombre/);
		expect(() => mockCrearProducto(nuevo({ barcode: '123' }))).toThrow(/barras/);
		expect(() => mockCrearProducto(nuevo({ precio_venta: 0 }))).toThrow(/precio de venta/);
		expect(() => mockCrearProducto(nuevo({ precio_costo: -1 }))).toThrow(/precio de costo/);
		expect(() => mockCrearProducto(nuevo({ stock_inicial: -2 }))).toThrow(/stock inicial/);
		expect(() => mockCrearProducto(nuevo({ stock_minimo: -1 }))).toThrow(/stock mínimo/);
	});

	it('rechaza barcode duplicado', () => {
		mockCrearProducto(nuevo());
		expect(() => mockCrearProducto(nuevo({ nombre: 'Otro' }))).toThrow(/código de barras/);
	});

	it('actualiza campos editables conservando el id', () => {
		const creado = mockCrearProducto(nuevo());
		mockActualizarProducto({ ...creado, precio_venta: 175000, stock_minimo: 5 });
		const p = tiendaMock.productos.find((x) => x.id === creado.id);
		expect(p?.precio_venta).toBe(175000);
		expect(p?.stock_minimo).toBe(5);
	});

	it('rechaza actualizar un producto inexistente o inválido', () => {
		expect(() =>
			mockActualizarProducto({ ...tiendaMock.productos[0], nombre: '' })
		).toThrow(/nombre/);
		expect(() => mockActualizarProducto({ ...tiendaMock.productos[0], id: 999 })).toThrow(
			/no existe/
		);
	});

	it('eliminar es baja lógica (no aparece al listar)', () => {
		const creado = mockCrearProducto(nuevo());
		mockEliminarProducto(creado.id);
		expect(tiendaMock.productos.find((p) => p.id === creado.id)?.activo).toBe(false);
		expect(mockListarProductos('', null, 50, 0).some((p) => p.id === creado.id)).toBe(false);
	});

	it('ajustar stock valida y aplica', () => {
		const creado = mockCrearProducto(nuevo());
		mockAjustarStock(creado.id, 45, 'recuento', 'ana');
		expect(tiendaMock.productos.find((p) => p.id === creado.id)?.stock).toBe(45);
		expect(() => mockAjustarStock(creado.id, -1, 'recuento', null)).toThrow(/negativo/);
		expect(() => mockAjustarStock(creado.id, 10, '   ', null)).toThrow(/motivo/);
		expect(() => mockAjustarStock(999, 10, 'recuento', null)).toThrow(/no existe/);
	});
});

describe('mock: categorías', () => {
	it('lista categorías ordenadas por nombre', () => {
		const nombres = mockListarCategorias().map((c) => c.nombre);
		expect(nombres).toEqual(['Alfajores', 'Bebidas', 'Snacks']);
	});

	it('crea una categoría con color por defecto', () => {
		const creada = mockCrearCategoria({ nombre: 'Lácteos', color: '#ef4444' });
		expect(creada.id).toBe(4);
		expect(creada.color).toBe('#ef4444');
		expect(mockListarCategorias()).toHaveLength(4);
	});

	it('rechaza nombre vacío o duplicado (sin importar mayúsculas)', () => {
		expect(() => mockCrearCategoria({ nombre: '  ' })).toThrow(/vacío/);
		expect(() => mockCrearCategoria({ nombre: 'bebidas' })).toThrow(/nombre/);
	});
});

describe('mock: cierre de caja', () => {
	function hoy(): string {
		const ahora = new Date();
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${ahora.getFullYear()}-${pad(ahora.getMonth() + 1)}-${pad(ahora.getDate())}`;
	}

	function venta(overrides: Partial<NuevaVenta> = {}): NuevaVenta {
		return {
			items: [{ producto_id: 1, cantidad: 2, precio_unitario: 180000 }],
			medio_pago: 'efectivo',
			monto_efectivo: 360000,
			monto_mp: 0,
			descripcion_mp: null,
			...overrides
		};
	}

	it('resumen del día suma totales, margen y delta MP como el backend', () => {
		// Efectivo: Coca-Cola x2 @180000 (costo 140000) → margen 2·40000 = 80000
		mockRegistrarVenta(venta());
		// MP: Alfajor Jorgito x1 @140000 (costo 100000) → margen 40000
		mockRegistrarVenta(
			venta({
				items: [{ producto_id: 3, cantidad: 1, precio_unitario: 140000 }],
				medio_pago: 'mercado_pago',
				monto_efectivo: 0,
				monto_mp: 140000
			})
		);
		// Mixto: Café x2 @260000 (costo 210000) → margen 2·50000 = 100000
		mockRegistrarVenta(
			venta({
				items: [{ producto_id: 4, cantidad: 2, precio_unitario: 260000 }],
				medio_pago: 'mixto',
				monto_efectivo: 300000,
				monto_mp: 220000
			})
		);

		const r = mockResumenDiario(hoy());
		expect(r.total_efectivo).toBe(660000);
		expect(r.total_mp).toBe(360000);
		expect(r.total_ventas).toBe(1020000);
		expect(r.cantidad_ventas).toBe(3);
		expect(r.margen_bruto).toBe(220000);
		// Sin pagos importados desde la API: delta = 0 − (140000 + 220000)
		expect(r.delta_mp).toBe(-360000);
	});

	it('resumen sin ventas devuelve todo en cero', () => {
		expect(mockResumenDiario('2020-01-01')).toEqual({
			total_efectivo: 0,
			total_mp: 0,
			total_ventas: 0,
			cantidad_ventas: 0,
			margen_bruto: 0,
			delta_mp: 0
		});
	});

	it('resumen valida el formato de la fecha', () => {
		expect(() => mockResumenDiario('10-09-2026')).toThrow(/YYYY-MM-DD/);
		expect(() => mockResumenDiario('2026-9-1')).toThrow(/YYYY-MM-DD/);
		expect(() => mockResumenDiario('')).toThrow(/YYYY-MM-DD/);
	});

	it('cerrar caja persiste el resumen y lo lista', () => {
		mockRegistrarVenta(venta());
		const cierre = mockCerrarCaja(hoy());
		expect(cierre.id).toBe(1);
		expect(cierre.fecha).toBe(hoy());
		expect(cierre.resumen.cantidad_ventas).toBe(1);
		expect(cierre.resumen.total_ventas).toBe(360000);
		expect(mockListarCierres()).toHaveLength(1);
	});

	it('listar cierres ordena por fecha descendente', () => {
		mockCerrarCaja('2026-09-10');
		mockCerrarCaja('2026-09-11');
		mockCerrarCaja('2026-09-09');
		expect(mockListarCierres().map((c) => c.fecha)).toEqual([
			'2026-09-11',
			'2026-09-10',
			'2026-09-09'
		]);
	});

	it('exporta CSV con encabezado, montos en centavos y filtro por rango', () => {
		mockCerrarCaja('2026-09-09');
		mockCerrarCaja('2026-09-10');
		const csv = mockExportarCierresCsv('2026-09-10', '2026-09-10');
		const lineas = csv.split('\n');
		expect(lineas[0]).toBe(
			'fecha,total_efectivo,total_mp,total_ventas,cantidad_ventas,margen_bruto,delta_mp'
		);
		expect(lineas).toHaveLength(2);
		expect(lineas[1]).toBe('2026-09-10,0,0,0,0,0,0');
		expect(mockExportarCierresCsv('', '').split('\n')).toHaveLength(3);
	});

	it('despacha los commands de cierre vía ejecutarMock', async () => {
		mockRegistrarVenta(venta());
		const resumen = await ejecutarMock<{ cantidad_ventas: number }>('resumen_diario', {
			fecha: hoy()
		});
		expect(resumen.cantidad_ventas).toBe(1);
		const cierre = await ejecutarMock<{ id: number }>('cerrar_caja', { fecha: hoy() });
		expect(cierre.id).toBe(1);
		const csv = await ejecutarMock<string>('exportar_cierres_csv', { desde: '', hasta: '' });
		expect(csv.startsWith('fecha,total_efectivo')).toBe(true);
	});
});