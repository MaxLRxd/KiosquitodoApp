export function formatearFechaLarga(fecha: Date): string {
	return fecha.toLocaleDateString('es-AR', {
		weekday: 'long',
		day: 'numeric',
		month: 'long',
		year: 'numeric'
	});
}