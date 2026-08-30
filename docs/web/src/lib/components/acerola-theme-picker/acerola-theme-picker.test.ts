import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { MODE_STORAGE_KEY, THEME_STORAGE_KEY } from '$lib/constants/themes';
import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
import AcerolaThemePicker from './acerola-theme-picker.svelte';

// useTheme() has module-level singleton $state (no per-instance state, no onMount/$effect
// inside the hook itself), so it can be called directly like a plain function. Its top-level
// `if (browser)` block reads localStorage and mutates document.documentElement on module load,
// so state leaks across tests in this file unless it's reset here.
const themeCtx = useTheme();

beforeEach(() => {
	localStorage.clear();
	document.documentElement.removeAttribute('data-theme');
	document.documentElement.classList.remove('dark');
	themeCtx.setTheme('catppuccin');
	themeCtx.setMode('dark');
});

describe('useTheme', () => {
	it('defaults to the catppuccin theme in dark mode', () => {
		expect(themeCtx.theme).toBe('catppuccin');
		expect(themeCtx.mode).toBe('dark');
		expect(themeCtx.resolved).toBe('dark');
	});

	it('setTheme updates the theme, persists it, and applies it to the document', () => {
		themeCtx.setTheme('nord');

		expect(themeCtx.theme).toBe('nord');
		expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe('nord');
		expect(document.documentElement.getAttribute('data-theme')).toBe('nord-dark');
	});

	it('setMode updates mode/resolved, persists it, and toggles the dark class', () => {
		themeCtx.setMode('light');

		expect(themeCtx.mode).toBe('light');
		expect(themeCtx.resolved).toBe('light');
		expect(localStorage.getItem(MODE_STORAGE_KEY)).toBe('light');
		expect(document.documentElement.classList.contains('dark')).toBe(false);
	});
});

describe('AcerolaThemePicker', () => {
	it('cycles the mode dark -> system -> light -> dark on repeated clicks', async () => {
		render(AcerolaThemePicker);
		const user = userEvent.setup();
		const button = screen.getByRole('button', { name: 'Mudar tema' });

		await user.click(button);
		expect(themeCtx.mode).toBe('system');

		await user.click(button);
		expect(themeCtx.mode).toBe('light');

		await user.click(button);
		expect(themeCtx.mode).toBe('dark');
	});
});
