import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { STORE_KEYS } from '$lib/constants/store-plugin';
import { THEMES } from '$lib/constants/themes';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useTheme } from './use-theme.svelte';

const getCurrentWindowMock = vi.mocked(getCurrentWindow);

/**
 * Registra um mock de `$app/environment` para a próxima importação dinâmica
 * do módulo (usado com `vi.resetModules()` + `import()`), já que o módulo
 * roda sua IIFE de carregamento de tema no top-level, no momento do import.
 */
function mockBrowserEnvironment(browserValue: boolean) {
	vi.doMock('$app/environment', () => ({
		browser: browserValue,
		dev: true,
		building: false,
		version: 'test'
	}));
}

/**
 * Registra um mock de `@tauri-apps/plugin-store` cujo `LazyStore` sempre
 * devolve a MESMA instância (ao invés de uma nova a cada `new LazyStore()`),
 * pra podermos guardar uma referência direta aos spies de `get`/`set` antes
 * mesmo de importar o módulo fresco.
 */
function mockPersistedStore(values: Record<string, unknown> = {}) {
	const store = {
		get: vi.fn((key: string) => Promise.resolve(values[key] ?? null)),
		set: vi.fn().mockResolvedValue(undefined)
	};

	vi.doMock('@tauri-apps/plugin-store', () => ({
		LazyStore: vi.fn().mockImplementation(function () {
			return store;
		})
	}));

	return store;
}

/** Aguarda alguns microtasks — suficiente pra IIFE de init do módulo resolver. */
async function flushMicrotasks() {
	for (let i = 0; i < 4; i++) {
		await Promise.resolve();
	}
}

async function renderHook() {
	let hook: ReturnType<typeof useTheme> | undefined;

	render(HookHarness, {
		props: {
			create: useTheme,
			onReady: (value) => {
				hook = value as ReturnType<typeof useTheme>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

async function flushTheme() {
	await tick();
	await Promise.resolve();
	await tick();
}

describe('useTheme', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		document.documentElement.removeAttribute('data-theme');
		document.documentElement.classList.remove('dark');
	});

	it('applies theme and light mode to the document', async () => {
		const hook = await renderHook();

		await hook.setMode('light');
		await hook.setTheme('nord');
		await flushTheme();

		expect(hook.theme).toBe('nord');
		expect(hook.mode).toBe('light');
		expect(hook.resolved).toBe('light');
		expect(document.documentElement.getAttribute('data-theme')).toBe(THEMES.nord.light);
		expect(document.documentElement.classList.contains('dark')).toBe(false);
	});

	it('resolves system mode from current window', async () => {
		const hook = await renderHook();

		// `theme`/`mode`/`resolved` são estado de módulo (singleton, sem reset entre
		// testes) — fixa o tema explicitamente em vez de assumir o valor inicial padrão,
		// que pode já ter sido trocado por outro teste antes deste.
		await hook.setTheme('catppuccin');
		await hook.setMode('system');
		await flushTheme();

		expect(hook.mode).toBe('system');
		expect(hook.resolved).toBe('light');
		expect(document.documentElement.classList.contains('dark')).toBe(false);
		// applyTheme() precisa ter rodado de fato pra fixar o data-theme, não só o mode
		// interno — sem isso um mutante que remove a chamada de applyTheme passa despercebido.
		expect(document.documentElement.getAttribute('data-theme')).toBe(THEMES.catppuccin.light);
	});

	it('applies dark mode directly (not via system), adding the dark class', async () => {
		const hook = await renderHook();

		await hook.setMode('dark');
		await hook.setTheme('nord');
		await flushTheme();

		expect(hook.resolved).toBe('dark');
		expect(document.documentElement.classList.contains('dark')).toBe(true);
		expect(document.documentElement.getAttribute('data-theme')).toBe(THEMES.nord.dark);
	});

	it('does not query window.theme() for a non-system mode', async () => {
		const themeQuery = vi.fn().mockResolvedValue('light');
		getCurrentWindowMock.mockReturnValueOnce({
			theme: themeQuery,
			setTheme: vi.fn().mockResolvedValue(undefined),
			onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() })
		} as unknown as ReturnType<typeof getCurrentWindow>);

		const hook = await renderHook();
		// Fixa data-theme em nord.dark ANTES do setMode isolado abaixo — se o setMode não
		// reaplicar o tema de fato, o data-theme fica preso nesse valor antigo.
		await hook.setTheme('nord');
		await hook.setMode('dark');
		await flushTheme();
		expect(document.documentElement.getAttribute('data-theme')).toBe(THEMES.nord.dark);

		await hook.setMode('light');
		await flushTheme();

		expect(themeQuery).not.toHaveBeenCalled();
		expect(document.documentElement.getAttribute('data-theme')).toBe(THEMES.nord.light);
	});

	it('uses the resolved window theme as-is (not a truthy fallback) when it differs from the default', async () => {
		getCurrentWindowMock.mockReturnValueOnce({
			theme: vi.fn().mockResolvedValue('dark'),
			setTheme: vi.fn().mockResolvedValue(undefined),
			onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() })
		} as unknown as ReturnType<typeof getCurrentWindow>);

		const hook = await renderHook();
		await hook.setMode('system');
		await flushTheme();

		// window.theme() resolveu 'dark' — precisa refletir exatamente isso, não um
		// fallback fixo pra 'light' só porque o valor era truthy.
		expect(hook.resolved).toBe('dark');
	});

	describe('module init (fresh module instance per test)', () => {
		// `theme`/`mode`/`resolved` e a IIFE de carregamento do tema salvo rodam no
		// top-level do módulo, no momento do import — por isso os testes abaixo usam
		// `vi.resetModules()` + `import()` dinâmico pra obter uma instância nova do
		// módulo (com sua própria IIFE, seu próprio `store` e seu `browser` mockado),
		// ao invés de reusar o singleton já carregado estaticamente no topo do arquivo.

		it('initializes mode and resolved to dark synchronously, and keeps the defaults when nothing was persisted', async () => {
			mockBrowserEnvironment(true);
			const store = mockPersistedStore({});
			vi.resetModules();

			const { useTheme: freshUseTheme } = await import('./use-theme.svelte');
			const hook = freshUseTheme();

			// Valores síncronos do `$state` logo após o import, antes de qualquer
			// microtask da IIFE de init rodar — prova que o valor inicial declarado
			// é 'dark', não apenas que ele converge pra 'dark' eventualmente.
			expect(hook.mode).toBe('dark');
			expect(hook.resolved).toBe('dark');

			await flushMicrotasks();

			// Prova que a IIFE realmente chamou `store.get` com as chaves certas
			// (mata o mutante que troca `Promise.all([...])` por `Promise.all([])`).
			expect(store.get).toHaveBeenCalledWith(STORE_KEYS.theme);
			expect(store.get).toHaveBeenCalledWith(STORE_KEYS.mode);

			// `savedTheme`/`savedMode` resolveram `null` (nada persistido) — não podem
			// sobrescrever os defaults com `null`.
			expect(hook.theme).toBe('catppuccin');
			expect(hook.mode).toBe('dark');
			expect(hook.resolved).toBe('dark');

			// `applyTheme(theme, mode)` precisa ter rodado de fato ao final da IIFE.
			expect(document.documentElement.getAttribute('data-theme')).toBe(THEMES.catppuccin.dark);
			expect(document.documentElement.classList.contains('dark')).toBe(true);
		});

		it('overrides theme and mode with the values persisted in the store on module init', async () => {
			mockBrowserEnvironment(true);
			const store = mockPersistedStore({
				[STORE_KEYS.theme]: 'dracula',
				[STORE_KEYS.mode]: 'light'
			});
			vi.resetModules();

			const { useTheme: freshUseTheme } = await import('./use-theme.svelte');
			const hook = freshUseTheme();

			await flushMicrotasks();

			expect(store.get).toHaveBeenCalledWith(STORE_KEYS.theme);
			expect(store.get).toHaveBeenCalledWith(STORE_KEYS.mode);

			// savedTheme truthy → sobrescreve o default 'catppuccin'.
			expect(hook.theme).toBe('dracula');
			// savedMode truthy → sobrescreve o default 'dark'.
			expect(hook.mode).toBe('light');
			// `applyTheme` rodou com os valores carregados, não com os defaults.
			expect(hook.resolved).toBe('light');
			expect(document.documentElement.getAttribute('data-theme')).toBe(THEMES.dracula.light);
			expect(document.documentElement.classList.contains('dark')).toBe(false);
		});

		it('does not load the persisted theme or apply it when not running in a browser', async () => {
			mockBrowserEnvironment(false);
			const store = mockPersistedStore({
				[STORE_KEYS.theme]: 'dracula',
				[STORE_KEYS.mode]: 'light'
			});
			vi.resetModules();

			const { useTheme: freshUseTheme } = await import('./use-theme.svelte');
			const hook = freshUseTheme();

			await flushMicrotasks();

			// `browser && (...)()` nunca deve rodar fora de um browser — nada é lido
			// do store, mesmo com valores salvos disponíveis.
			expect(store.get).not.toHaveBeenCalled();
			expect(hook.theme).toBe('catppuccin');
			expect(hook.mode).toBe('dark');
			expect(document.documentElement.hasAttribute('data-theme')).toBe(false);

			// `applyTheme` também guarda em `browser` internamente — chamá-la
			// (indiretamente, via setTheme/setMode) fora de um browser precisa ser
			// um no-op: nem `getCurrentWindow()` nem o DOM devem ser tocados.
			await hook.setTheme('nord');
			await hook.setMode('light');
			await flushMicrotasks();

			expect(getCurrentWindowMock).not.toHaveBeenCalled();
			expect(document.documentElement.hasAttribute('data-theme')).toBe(false);
			expect(document.documentElement.classList.contains('dark')).toBe(false);
			expect(hook.resolved).toBe('dark');
		});
	});
});
