import { render, screen } from '@testing-library/svelte';
import { fireEvent } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import AcerolaPeerPicker from './acerola-peer-picker.svelte';
import type { PairedPeerPayload } from '$lib/contracts/network/network.payloads';

describe('AcerolaPeerPicker', () => {
	const mockPeers: PairedPeerPayload[] = [
		{ peerId: 'peer-1', addrs: [1, 2, 3], deviceName: 'Meu Celular' },
		{ peerId: 'peer-2-longid-abcdefghij', addrs: [4, 5, 6], deviceName: null }
	];

	const mockOnOpenChange = vi.fn();
	const mockOnSelect = vi.fn();

	it('renders nothing when open is false', () => {
		render(AcerolaPeerPicker, {
			props: {
				state: { open: false },
				data: { peers: mockPeers },
				events: { onOpenChange: mockOnOpenChange, onSelect: mockOnSelect }
			}
		});

		expect(screen.queryByText('Meu Celular')).not.toBeInTheDocument();
	});

	it('renders the list of peers when open', () => {
		render(AcerolaPeerPicker, {
			props: {
				state: { open: true },
				data: { peers: mockPeers },
				events: { onOpenChange: mockOnOpenChange, onSelect: mockOnSelect }
			}
		});

		expect(screen.getByText(/Choose a device|Escolher dispositivo/i)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Meu Celular' })).toBeInTheDocument();
	});

	it('falls back to a shortened peer id when deviceName is null', () => {
		render(AcerolaPeerPicker, {
			props: {
				state: { open: true },
				data: { peers: mockPeers },
				events: { onOpenChange: mockOnOpenChange, onSelect: mockOnSelect }
			}
		});

		expect(screen.queryByText('peer-2-longid-abcdefghij')).not.toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'peer-2-l…efghij' })).toBeInTheDocument();
	});

	it('calls onSelect with the clicked peer', async () => {
		render(AcerolaPeerPicker, {
			props: {
				state: { open: true },
				data: { peers: mockPeers },
				events: { onOpenChange: mockOnOpenChange, onSelect: mockOnSelect }
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Meu Celular' }));

		expect(mockOnSelect).toHaveBeenCalledWith(mockPeers[0]);
	});

	it('shows an empty state message when there are no peers', () => {
		render(AcerolaPeerPicker, {
			props: {
				state: { open: true },
				data: { peers: [] },
				events: { onOpenChange: mockOnOpenChange, onSelect: mockOnSelect }
			}
		});

		expect(
			screen.getByText(/No devices paired yet|Nenhum dispositivo pareado ainda/i)
		).toBeInTheDocument();
		// Only the dialog's own close button remains — no peer row buttons.
		expect(screen.getAllByRole('button')).toHaveLength(1);
	});
});
