/** Formatea centavos (i64 del backend) como pesos argentinos para la UI. */
export function formatearPesos(centavos: number): string {
	return (centavos / 100).toLocaleString('es-AR', { style: 'currency', currency: 'ARS' });
}