/**
 * Normaliza un texto para comparaciones tolerantes: minúsculas, sin acentos
 * y con cualquier signo de puntuación convertido en espacio.
 */
export function normalizarTexto(valor: string): string {
	return valor
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '')
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, ' ')
		.trim();
}

/**
 * Coincidencia por tokens: «coca cola» encuentra «Coca-Cola 500ml» y «cafe»
 * encuentra «Café instantáneo». Todos los términos deben aparecer.
 */
export function coincideBusqueda(nombre: string, consulta: string): boolean {
	const terminos = normalizarTexto(consulta).split(' ').filter(Boolean);
	if (terminos.length === 0) return true;
	const objetivo = normalizarTexto(nombre);
	return terminos.every((termino) => objetivo.includes(termino));
}
