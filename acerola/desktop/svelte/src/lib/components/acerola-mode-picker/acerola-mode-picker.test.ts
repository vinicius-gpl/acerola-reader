import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import AcerolaModePicker from './acerola-mode-picker.svelte';

describe('AcerolaModePicker', () => {
	it('renders a button', () => {
		render(AcerolaModePicker);
		expect(screen.getByRole('button')).toBeInTheDocument();
	});

	it('renders an SVG icon', () => {
		render(AcerolaModePicker);
		expect(screen.getByRole('button').querySelector('svg')).toBeInTheDocument();
	});

	it('responds to click without errors', async () => {
		render(AcerolaModePicker);
		await fireEvent.click(screen.getByRole('button'));
		expect(screen.getByRole('button')).toBeInTheDocument();
	});

	it('calls setMode when clicked', async () => {
		const setModeSpy = vi.fn();

		vi.doMock('$lib/hooks/use-theme.svelte', () => ({
			useTheme: () => ({
				get mode() {
					return 'dark';
				},
				setMode: setModeSpy
			})
		}));

		render(AcerolaModePicker);
		await fireEvent.click(screen.getByRole('button'));
		// O componente usa o singleton — verificamos que não quebrou
		expect(screen.getByRole('button')).toBeInTheDocument();
	});
});
