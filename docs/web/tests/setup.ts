import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/svelte';
import { afterEach, vi } from 'vitest';

// Mock para resolver problemas do jsdom e pointer events do bits-ui (shadcn-svelte)
if (typeof window !== 'undefined') {
	window.HTMLElement.prototype.hasPointerCapture = vi.fn();
	window.HTMLElement.prototype.releasePointerCapture = vi.fn();
	window.HTMLElement.prototype.scrollIntoView = vi.fn();
	if (!Element.prototype.animate) {
		Element.prototype.animate = vi.fn().mockImplementation(() => ({
			finished: Promise.resolve(),
			cancel: vi.fn(),
			finish: vi.fn(),
			pause: vi.fn(),
			play: vi.fn(),
			reverse: vi.fn(),
			onfinish: null,
			oncancel: null
		}));
	}
	// @ts-ignore: Mock simples pra resolver hasPointerCapture em testes
	window.PointerEvent = class PointerEvent extends Event {};

	// Mock localStorage — use-theme.svelte.ts lê/escreve tema e modo aqui diretamente
	const localStorageStore = new Map<string, string>();
	Object.defineProperty(window, 'localStorage', {
		value: {
			getItem: vi.fn((key: string) => localStorageStore.get(key) ?? null),
			setItem: vi.fn((key: string, value: string) => {
				localStorageStore.set(key, value);
			}),
			removeItem: vi.fn((key: string) => localStorageStore.delete(key)),
			clear: vi.fn(() => localStorageStore.clear())
		},
		writable: true
	});

	// Mock matchMedia — use-theme.svelte.ts usa pra resolver o modo 'system'
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		value: vi.fn().mockImplementation((query) => ({
			matches: false,
			media: query,
			onchange: null,
			addListener: vi.fn(),
			removeListener: vi.fn(),
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		}))
	});
}

afterEach(async () => {
	cleanup();
	await new Promise((resolve) => setTimeout(resolve, 30));
	vi.clearAllMocks();
});
