import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkPeerList, { type DisplayPeer } from './acerola-network-peer-list.svelte';

describe('AcerolaNetworkPeerList', () => {
	const onlinePeer: DisplayPeer = {
		peerId: 'peer-1',
		deviceName: 'Meu Celular',
		nickname: null,
		connected: true
	};
	const offlinePeer: DisplayPeer = {
		peerId: 'peer-2',
		deviceName: null,
		nickname: null,
		connected: false
	};

	const mockEvents = {
		onSyncHistory: vi.fn(),
		onSyncFiles: vi.fn(),
		onSyncAll: vi.fn(),
		onBrowseLibrary: vi.fn(),
		onRemove: vi.fn(),
		onRename: vi.fn()
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

	it('prefers the local nickname over the device name when both are set', () => {
		const nicknamedPeer: DisplayPeer = {
			peerId: 'peer-3',
			deviceName: 'Meu Celular',
			nickname: 'Celular da Sala',
			connected: true
		};

		render(AcerolaNetworkPeerList, {
			props: {
				data: {
					peers: [nicknamedPeer],
					addrFor: () => [1, 2, 3],
					statusLabel: () => 'online',
					isSyncing: () => false
				},
				events: mockEvents
			}
		});

		expect(screen.getByText('Celular da Sala')).toBeInTheDocument();
		expect(screen.queryByText('Meu Celular')).not.toBeInTheDocument();
	});

	it('renames a peer: opens the editor pre-filled with the current nickname and saves it', async () => {
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

		await fireEvent.click(document.querySelector('[data-popover-trigger]')!);
		await fireEvent.click(screen.getByRole('button', { name: /Rename|Renomear/i }));

		const input = screen.getByPlaceholderText(/Nickname for this device|Apelido pra esse dispositivo/i);
		expect(input).toHaveValue('');

		await fireEvent.input(input, { target: { value: '  Celular da Sala  ' } });
		await fireEvent.click(screen.getByRole('button', { name: /Save|Salvar/i }));

		expect(mockEvents.onRename).toHaveBeenCalledWith(onlinePeer.peerId, 'Celular da Sala');
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
