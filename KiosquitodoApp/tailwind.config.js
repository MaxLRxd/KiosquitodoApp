/** @type {import('tailwindcss').Config} */
export default {
	// Solo se escanean plantillas: algunos `.ts` (por ejemplo las regex de mock.ts)
	// contienen secuencias como `[-:T]` que Tailwind interpretaría como clase e
	// invalidarían el CSS generado.
	content: ['./src/**/*.{html,svelte}'],
	theme: {
		extend: {
			boxShadow: {
				card: '0 1px 2px 0 rgb(15 23 42 / 0.04), 0 18px 40px -28px rgb(15 23 42 / 0.28)',
				float: '0 30px 80px -30px rgb(15 23 42 / 0.38)'
			},
			keyframes: {
				'fade-up': {
					from: { opacity: '0', transform: 'translateY(6px)' },
					to: { opacity: '1', transform: 'translateY(0)' }
				},
				'pulse-soft': {
					'0%, 100%': { opacity: '1' },
					'50%': { opacity: '0.55' }
				}
			},
			animation: {
				'fade-up': 'fade-up 0.25s ease-out both',
				'pulse-soft': 'pulse-soft 2.4s ease-in-out infinite'
			}
		}
	},
	plugins: []
};
