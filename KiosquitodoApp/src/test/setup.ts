import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// jsdom no implementa HTMLMediaElement.play(); la app solo usa sonidos como
// feedback y los ignora si fallan, así que se evita el ruido en los tests.
vi.stubGlobal(
	'Audio',
	class {
		play() {
			return Promise.resolve();
		}
	}
);
