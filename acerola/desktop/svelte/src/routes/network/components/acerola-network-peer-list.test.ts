import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkPeerList, {
	type DisplayPeer
} from './acerola-network-peer-list.svelte';

describe('AcerolaNetworkPeerList', () => {
	const onlinePeer: DisplayPeer = { peerId: 'peer-1', deviceName: 'Meu Celular', connected: true };
	const offlinePeer: DisplayPeer = { peerId: 'peer-2', deviceName: null, connected: false };

	const mockEvents = {
		onSyncHistory: vi.fn(),
		onSyncFiles: vi.fn(),
		onSyncAll: vi.fn(),
		onBrowseLibrary: vi.fn(),
		onRemove: vi.fn()
	};

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('shows the empty state when there are no peers', () => {
		render(AcerolaNetworkPeerList, {
			props: {
				data: {
					peers: [],
					addrFor: () => undefined,
					statusLabel: () => '',
					isSyncing: () => false
				},
				events: mockEvents
			}
		});

		expect(screen.getByText(/No paired devices|Nenhum dispositivo pareado/i)).toBeInTheDocument();
	});

	it('renders a peer using its device name, falling back to a short id when absent', () => {
		render(AcerolaNetworkPeerList, {
			props: {
				data: {
					peers: [onlinePeer, offlinePeer],
					addrFor: (id) => (id === onlinePeer.peerId ? [1, 2, 3] : undefined),
					statusLabel: (peer) => (peer.connected ? 'online' : 'offline'),
					isSyncing: () => false
				},
				events: mockEvents
			}
		});

		expect(screen.getByText('Meu Celular')).toBeInTheDocument();
	});

	it('shows the offline hint when a peer has no known address', () => {
		render(AcerolaNetworkPeerList, {
			props: {
				data: {
					peers: [offlinePeer],
					addrFor: () => undefined,
					statusLabel: () => 'offline',
					isSyncing: () => false
				},
				events: mockEvents
			}
		});

		expect(screen.getByText(/Offline|desconectado/i)).toBeInTheDocument();
	});

	it('disables the sync-all button when the peer has no known address', () => {
		render(AcerolaNetworkPeerList, {
			props: {
				data: {
					peers: [offlinePeer],
					addrFor: () => undefined,
					statusLabel: () => 'offline',
					isSyncing: () => false
				},
				events: mockEvents
			}
		});

		expect(screen.getByRole('button', { name: /Sync all|Sincronizar tudo/i })).toBeDisabled();
	});

	it('calls onSyncAll with the peer id when the sync-all button is clicked', async () => {
		render(AcerolaNetworkPeerList, {
			props: {
				data: {
					peers: [onlinePeer],
					addrFor: () => [1, 2, 3],
					statusLabel: () => 'online',
					isSyncing: () => false
				},
				events: mockEvents
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /Sync all|Sincronizar tudo/i }));

		expect(mockEvents.onSyncAll).toHaveBeenCalledWith(onlinePeer.peerId);
	});

	it('spins the sync-all icon while a sync is in progress for that peer', () => {
		render(AcerolaNetworkPeerList, {
			props: {
				data: {
					peers: [onlinePeer],
					addrFor: () => [1, 2, 3],
					statusLabel: () => 'online',
					isSyncing: (peerId, kind) => peerId === onlinePeer.peerId && kind === 'history'
				},
				events: mockEvents
			}
		});

		const button = screen.getByRole('button', { name: /Sync all|Sincronizar tudo/i });
		expect(button.querySelector('.animate-spin')).not.toBeNull();
	});
});
