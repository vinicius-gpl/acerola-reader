import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/svelte';
import { afterEach, vi } from 'vitest';

// Mock para resolver problemas do jsdom e pointer events do radix/bits-ui
if (typeof window !== 'undefined') {
	window.HTMLElement.prototype.hasPointerCapture = vi.fn();
	window.HTMLElement.prototype.releasePointerCapture = vi.fn();
	window.HTMLElement.prototype.scrollIntoView = vi.fn();
	// jsdom já expõe `Element.prototype.animate` (não é `undefined`), só que devolve algo
	// incompleto — então o guard óbvio `if (!Element.prototype.animate)` nunca pega esse caso
	// (não dá pra usá-lo aqui). Função comum, NÃO `vi.fn()`, pelo mesmo motivo do `matchMedia`
	// abaixo: `vi.resetAllMocks()` no `beforeEach` de vários arquivos apagaria a implementation
	// de um spy. `@formkit/auto-animate` trata o retorno como um `Animation` de verdade (que
	// estende `EventTarget`) e registra listeners de `finish` nele — sem isso, quebra de forma
	// assíncrona (fora do corpo do teste) assim que uma lista com `use:autoAnimate` ganha/perde
	// um item.
	Element.prototype.animate = function animate() {
		return {
			finished: Promise.resolve(),
			cancel: vi.fn(),
			finish: vi.fn(),
			pause: vi.fn(),
			play: vi.fn(),
			reverse: vi.fn(),
			onfinish: null,
			oncancel: null,
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		} as unknown as Animation;
	};
	// @ts-ignore: Mock simples para testes que resolve o hasPointerCapture
	window.PointerEvent = class PointerEvent extends Event {};

	// jsdom não implementa ResizeObserver/IntersectionObserver — usados por actions como
	// `slidingIndicator` e `AcerolaSectionNav` (scroll-spy). Guardado com `if (!X)` (ao
	// contrário do `animate` acima, esses dois realmente não existem em jsdom por padrão), pra
	// não pisar em stubs que um teste específico já registra pra controlar manualmente os
	// callbacks (ex.: `comic-page.test.ts`).
	if (!window.ResizeObserver) {
		window.ResizeObserver = class ResizeObserver {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		};
	}
	if (!window.IntersectionObserver) {
		// @ts-ignore: stub mínimo, suficiente pra código que só chama observe/disconnect
		window.IntersectionObserver = class IntersectionObserver {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		};
	}

	// Mock localStorage
	Object.defineProperty(window, 'localStorage', {
		value: {
			getItem: vi.fn(),
			setItem: vi.fn(),
			removeItem: vi.fn(),
			clear: vi.fn()
		},
		writable: true
	});

	// Mock matchMedia — função comum, NÃO `vi.fn()`: vários arquivos de teste chamam
	// `vi.resetAllMocks()` no próprio `beforeEach`, o que apagaria a implementation de um spy e
	// devolveria `undefined` de `window.matchMedia(...)` pro resto daquele arquivo — quebrando
	// qualquer código que leia `.matches` no retorno (ex.: `@formkit/auto-animate` checando
	// `prefers-reduced-motion` no mount). Nenhum teste hoje assert em cima deste mock global
	// (`is-mobile.test.ts` registra o seu próprio, isolado, quando precisa espiar chamadas).
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		value: (query: string) => ({
			matches: false,
			media: query,
			onchange: null,
			addListener: vi.fn(), // Deprecated
			removeListener: vi.fn(), // Deprecated
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		})
	});
}

// Mock SvelteKit $app/environment — browser = true para testes de componente
vi.mock('$app/environment', async () => await import('./mocks/app-environment.ts'));
vi.mock('$app/state', async () => await import('./mocks/app-state.ts'));

// Mock Tauri window — getCurrentWindow não existe em jsdom
vi.mock('@tauri-apps/api/window', () => ({
	getCurrentWindow: vi.fn(() => ({
		theme: vi.fn().mockResolvedValue('light'),
		setTheme: vi.fn().mockResolvedValue(undefined),
		onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() })
	}))
}));

// Mock Tauri store — comportamento padrão: retorna null (sem valor salvo)
vi.mock('@tauri-apps/plugin-store', () => {
	return {
		LazyStore: class {
			constructor() {
				return {
					get: vi.fn().mockResolvedValue(null),
					set: vi.fn().mockResolvedValue(undefined)
				};
			}
		}
	};
});

afterEach(async () => {
	cleanup();
	await new Promise((resolve) => setTimeout(resolve, 30));
	vi.clearAllMocks();
});
