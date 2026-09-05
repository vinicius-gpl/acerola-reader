import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkMyDeviceCard from './acerola-network-my-device-card.svelte';

vi.mock('svelte-sonner', () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

Object.assign(navigator, {
	clipboard: { writeText: vi.fn().mockResolvedValue(undefined) }
});

describe('AcerolaNetworkMyDeviceCard', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('shows the device name and short id', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'local'
				}
			}
		});

		expect(screen.getByText('Meu Notebook')).toBeInTheDocument();
	});

	it('disables the copy button when there is no local id yet', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: null,
					localId: null,
					mode: undefined
				}
			}
		});

		expect(screen.getByRole('button', { name: /Copy|Copiar/i })).toBeDisabled();
	});

	it('copies the local id to the clipboard when the copy button is clicked', async () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'relay'
				}
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /Copy|Copiar/i }));

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith('abcdefghijklmnop');
	});

	it('does not show a rename button when no onRenameDevice handler is provided', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'local'
				}
			}
		});

		expect(screen.queryByRole('button', { name: /Rename|Renomear/i })).not.toBeInTheDocument();
	});

	it('renames the device: opens the editor pre-filled, saves and calls the handler', async () => {
		const onRenameDevice = vi.fn().mockResolvedValue(undefined);
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'local'
				},
				events: { onRenameDevice }
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /Rename|Renomear/i }));

		const input = screen.getByPlaceholderText(/device's name|Nome deste dispositivo/i);
		expect(input).toHaveValue('Meu Notebook');

		await fireEvent.input(input, { target: { value: 'Notebook Novo' } });
		await fireEvent.click(screen.getByRole('button', { name: /Save|Salvar/i }));

		expect(onRenameDevice).toHaveBeenCalledWith('Notebook Novo');
		expect(
			screen.queryByPlaceholderText(/device's name|Nome deste dispositivo/i)
		).not.toBeInTheDocument();
	});

	it('cancels the rename without calling the handler', async () => {
		const onRenameDevice = vi.fn();
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'local'
				},
				events: { onRenameDevice }
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /Rename|Renomear/i }));
		await fireEvent.click(screen.getByRole('button', { name: /Cancel|Cancelar/i }));

		expect(onRenameDevice).not.toHaveBeenCalled();
		expect(screen.getByText('Meu Notebook')).toBeInTheDocument();
	});

	it('shows the local connection mode label', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'local'
				}
			}
		});

		expect(screen.getByText(/Local network|Rede local/i)).toBeInTheDocument();
	});

	it('shows the relay connection mode label', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'relay'
				}
			}
		});

		expect(screen.getByText(/^Relay$/i)).toBeInTheDocument();
	});
});
