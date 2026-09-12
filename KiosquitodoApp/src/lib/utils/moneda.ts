/** Formatea centavos (i64 del backend) como pesos argentinos para la UI. */
export function formatearPesos(centavos: number): string {
	return (centavos / 100).toLocaleString('es-AR', { style: 'currency', currency: 'ARS' });
}

/** Convierte una cadena ingresada por el operador (decimal con coma o punto) a centavos. */
export function centavosDesdePesos(cadena: string): number {
	let normalizada = cadena.trim().replace(/\s/g, '');
	if (normalizada === '') return 0;
	// Convención es-AR: "." separa miles y "," es el separador decimal.
	if (normalizada.includes(',')) {
		normalizada = normalizada.replace(/\./g, '').replace(',', '.');
	}
	const valor = Number.parseFloat(normalizada);
	if (Number.isNaN(valor) || valor < 0) return 0;
	return Math.round(valor * 100);
}