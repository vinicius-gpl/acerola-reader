import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { THEMES } from '$lib/constants/themes';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useTheme } from './use-theme.svelte';

const getCurrentWindowMock = vi.mocked(getCurrentWindow);

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
});
