export interface Modulo {
	tecla: string;
	ruta: string;
	nombre: string;
}

export const MODULOS: Modulo[] = [
	{ tecla: 'F1', ruta: '/', nombre: 'POS' },
	{ tecla: 'F2', ruta: '/inventario', nombre: 'Inventario' },
	{ tecla: 'F3', ruta: '/conciliacion', nombre: 'Conciliación' },
	{ tecla: 'F4', ruta: '/cierre', nombre: 'Cierre de Caja' },
	{ tecla: 'F5', ruta: '/configuracion', nombre: 'Configuración' },
	{ tecla: 'F6', ruta: '/devoluciones', nombre: 'Devoluciones' }
];