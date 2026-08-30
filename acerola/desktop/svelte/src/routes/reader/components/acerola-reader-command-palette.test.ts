import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaReaderCommandPalette from './acerola-reader-command-palette.svelte';

function props(overrides = {}) {
	return {
		data: {
			zoomMode: false
		},
		state: {
			open: true,
			value: '',
			readingMode: 'vertical'
		},
		events: {
			onOpenChange: vi.fn(),
			onValueChange: vi.fn(),
			onReadingModeChange: vi.fn(),
			onToggleZoomMode: vi.fn(),
			onZoomIn: vi.fn(),
			onZoomOut: vi.fn(),
			onResetZoom: vi.fn()
		},
		...overrides
	} as const;
}

async function selectCommand(label: string) {
	const user = userEvent.setup();
	await user.click(screen.getByText(label));
}

describe('AcerolaReaderCommandPalette', () => {
	it('does not render when closed', () => {
		render(AcerolaReaderCommandPalette, {
			props: props({
				state: {
					...props().state,
					open: false
				}
			})
		});

		expect(screen.queryByPlaceholderText('Comandos do leitor...')).not.toBeInTheDocument();
		expect(screen.queryByText('Zoom')).not.toBeInTheDocument();
	});

	it('renders groups, input and commands when open', () => {
		render(AcerolaReaderCommandPalette, { props: props() });

		expect(screen.getByPlaceholderText('Comandos do leitor...')).toBeInTheDocument();
		expect(screen.getByText('Zoom')).toBeInTheDocument();
		expect(screen.getByText('Leitura')).toBeInTheDocument();
		expect(screen.getByText('Ativar modo zoom')).toBeInTheDocument();
		expect(screen.getByText('Aumentar zoom')).toBeInTheDocument();
		expect(screen.getByText('Reduzir zoom')).toBeInTheDocument();
		expect(screen.getByText('Resetar zoom')).toBeInTheDocument();
		expect(screen.getByText('Paginado vertical')).toBeInTheDocument();
		expect(screen.getByText('Paginado horizontal')).toBeInTheDocument();
		expect(screen.getByText('Webtoon')).toBeInTheDocument();
	});

	it('changes zoom command text when zoom mode is active', () => {
		render(AcerolaReaderCommandPalette, {
			props: props({
				data: {
					zoomMode: true
				}
			})
		});

		expect(screen.getByText('Desativar modo zoom')).toBeInTheDocument();
		expect(screen.queryByText('Ativar modo zoom')).not.toBeInTheDocument();
	});

	it('closes and clears search when clicking backdrop', async () => {
		const user = userEvent.setup();
		const paletteProps = props({
			state: {
				...props().state,
				value: 'zoom'
			}
		});

		render(AcerolaReaderCommandPalette, { props: paletteProps });
		await user.click(screen.getByLabelText('Fechar comandos'));

		expect(paletteProps.events.onOpenChange).toHaveBeenCalledWith(false);
		expect(paletteProps.events.onValueChange).toHaveBeenCalledWith('');
	});

	it('propagates search value change', async () => {
		const user = userEvent.setup();
		const paletteProps = props();

		render(AcerolaReaderCommandPalette, { props: paletteProps });
		await user.type(screen.getByPlaceholderText('Comandos do leitor...'), 'zoom');

		expect(paletteProps.events.onValueChange).toHaveBeenCalled();
	});

	it('executes zoom commands and closes palette', async () => {
		const paletteProps = props();

		render(AcerolaReaderCommandPalette, { props: paletteProps });

		await selectCommand('Ativar modo zoom');
		await selectCommand('Aumentar zoom');
		await selectCommand('Reduzir zoom');
		await selectCommand('Resetar zoom');

		expect(paletteProps.events.onToggleZoomMode).toHaveBeenCalledOnce();
		expect(paletteProps.events.onZoomIn).toHaveBeenCalledOnce();
		expect(paletteProps.events.onZoomOut).toHaveBeenCalledOnce();
		expect(paletteProps.events.onResetZoom).toHaveBeenCalledOnce();
		expect(paletteProps.events.onOpenChange).toHaveBeenCalledWith(false);
		expect(paletteProps.events.onValueChange).toHaveBeenCalledWith('');
	});

	it('executes reading mode commands and closes palette', async () => {
		const paletteProps = props();

		render(AcerolaReaderCommandPalette, { props: paletteProps });

		await selectCommand('Paginado vertical');
		await selectCommand('Paginado horizontal');
		await selectCommand('Webtoon');

		expect(paletteProps.events.onReadingModeChange).toHaveBeenCalledWith('vertical');
		expect(paletteProps.events.onReadingModeChange).toHaveBeenCalledWith('horizontal');
		expect(paletteProps.events.onReadingModeChange).toHaveBeenCalledWith('webtoon');
		expect(paletteProps.events.onOpenChange).toHaveBeenCalledWith(false);
		expect(paletteProps.events.onValueChange).toHaveBeenCalledWith('');
	});
});
