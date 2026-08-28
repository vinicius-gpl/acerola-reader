import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
import ColorPicker from './color-picker.svelte';

// useTheme() is a module-level singleton (see use-theme.svelte.ts) — force it back to the
// default theme before every test so selections made in one test don't leak into the next.
const themeCtx = useTheme();

beforeEach(() => {
	themeCtx.setTheme('catppuccin');
});

describe('ColorPicker', () => {
	it('lists every theme family, name and description, once opened', async () => {
		render(ColorPicker);
		const user = userEvent.setup();

		await user.click(screen.getByRole('button', { name: 'Mudar paleta de cores' }));

		expect(await screen.findByText('Catppuccin')).toBeInTheDocument();
		expect(screen.getByText('Paleta pastel suave com tons roxos e rosados')).toBeInTheDocument();
		expect(screen.getByText('Nord')).toBeInTheDocument();
		expect(screen.getByText('Dracula')).toBeInTheDocument();
		expect(screen.getByText('Tokyo Night')).toBeInTheDocument();
	});

	it('selecting a family calls setTheme with that family', async () => {
		render(ColorPicker);
		const user = userEvent.setup();

		await user.click(screen.getByRole('button', { name: 'Mudar paleta de cores' }));
		await user.click(await screen.findByText('Nord'));

		expect(themeCtx.theme).toBe('nord');
	});
});
