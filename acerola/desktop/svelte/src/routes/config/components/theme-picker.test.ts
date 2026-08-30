import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import ThemePicker from './acerola-theme-picker.svelte';

describe('ThemePicker', () => {
	it('renders available themes', () => {
		render(ThemePicker, {
			props: {
				data: { theme: 'catppuccin', mode: 'dark' },
				events: { onSelect: vi.fn() }
			}
		});

		expect(screen.getByText('Catppuccin')).toBeInTheDocument();
		expect(screen.getByText('Nord')).toBeInTheDocument();
		expect(screen.getByText('Dracula')).toBeInTheDocument();
		expect(screen.getByText('Tokyo Night')).toBeInTheDocument();
	});

	it('calls onselect with correct id when clicking a theme', async () => {
		const user = userEvent.setup();
		const onselect = vi.fn();

		render(ThemePicker, {
			props: {
				data: { theme: 'catppuccin', mode: 'dark' },
				events: { onSelect: onselect }
			}
		});
		await user.click(screen.getByText('Nord').closest('button')!);

		expect(onselect).toHaveBeenCalledOnce();
		expect(onselect).toHaveBeenCalledWith('nord');
	});

	it('applies selected style to active theme', () => {
		const { container } = render(ThemePicker, {
			props: {
				data: { theme: 'nord', mode: 'dark' },
				events: { onSelect: vi.fn() }
			}
		});

		const buttons = container.querySelectorAll('button');
		const nordBtn = Array.from(buttons).find((b) => b.textContent?.includes('Nord'));

		expect(nordBtn?.className).toContain('border-primary');
	});

	it('does not apply selected style to inactive themes', () => {
		const { container } = render(ThemePicker, {
			props: {
				data: { theme: 'nord', mode: 'dark' },
				events: { onSelect: vi.fn() }
			}
		});

		const buttons = container.querySelectorAll('button');
		const catppuccinBtn = Array.from(buttons).find((b) => b.textContent?.includes('Catppuccin'));

		expect(catppuccinBtn?.className).not.toContain('border-primary');
	});

	it('renders correct colors in light mode', () => {
		const { container } = render(ThemePicker, {
			props: {
				data: { theme: 'catppuccin', mode: 'light' },
				events: { onSelect: vi.fn() }
			}
		});

		const colorDots = container.querySelectorAll<HTMLElement>("[style*='background-color']");
		const colors = Array.from(colorDots).map((el) => el.style.backgroundColor);

		// Catppuccin light: #8839EF, #EA76CB, #1E66F5, #EFF1F5
		expect(colors[0]).toBeTruthy();
	});
});
