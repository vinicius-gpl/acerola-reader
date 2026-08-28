import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import type { PairedPeerPayload } from '$lib/contracts/network/network.payloads';
import {
	InvalidConnectionCodeError,
	encodeConnectionCode,
	type LocalPeerAddr
} from '$lib/utils/connection-code.utils';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { usePeerConnection } from './use-peer-connection.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);

async function renderHook() {
	let hook: ReturnType<typeof usePeerConnection> | undefined;

	render(HookHarness, {
		props: {
			create: () => usePeerConnection(),
			onReady: (value) => {
				hook = value as ReturnType<typeof usePeerConnection>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

function setupListeners() {
	const callbacks = new Map<string, (event: { payload: unknown }) => void>();
	const unlisteners = new Map<string, ReturnType<typeof vi.fn>>();

	listenMock.mockImplementation((event, callback) => {
		callbacks.set(String(event), callback as (event: { payload: unknown }) => void);
		const unlisten = vi.fn();
		unlisteners.set(String(event), unlisten);
		return Promise.resolve(unlisten);
	});

	return { callbacks, unlisteners };
}

function defaultInvokeImpl(overrides: Record<string, unknown> = {}) {
	const defaults: Record<string, unknown> = {
		[NETWORK_COMMANDS.getLocalId]: 'local-id',
		[NETWORK_COMMANDS.getLocalAddr]: { id: { id: 'local-id', device_id: null }, addrs: [1, 2] },
		[NETWORK_COMMANDS.getLocalDeviceInfo]: { name: 'Desktop', os: 'windows', version: '1.0' },
		[NETWORK_COMMANDS.getRelayInfo]: { defaultRelay: 'relay-a', activeRelay: 'relay-a' },
		[NETWORK_COMMANDS.getNetworkStatus]: undefined,
		[NETWORK_COMMANDS.getPairedPeers]: []
	};

	invokeMock.mockImplementation((command: string) =>
		Promise.resolve(command in overrides ? overrides[command] : defaults[command])
	);
}

describe('usePeerConnection', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loadLocalInfo populates local identity fields from the backend', async () => {
		defaultInvokeImpl();
		const hook = await renderHook();

		await hook.loadLocalInfo();

		expect(hook.localId).toBe('local-id');
		expect(hook.localDeviceInfo).toEqual({ name: 'Desktop', os: 'windows', version: '1.0' });
		expect(hook.relayInfo).toEqual({ defaultRelay: 'relay-a', activeRelay: 'relay-a' });
	});

	it('startListening registers status/handshake listeners and loads status + peers', async () => {
		const { unlisteners } = setupListeners();
		const pairedPeers: PairedPeerPayload[] = [
			{ peerId: 'peer-1', addrs: [7, 8], deviceName: 'Phone' }
		];
		defaultInvokeImpl({ [NETWORK_COMMANDS.getPairedPeers]: pairedPeers });

		const hook = await renderHook();
		await hook.startListening();

		expect(unlisteners.size).toBe(3);
		expect(hook.pairedPeers).toEqual(pairedPeers);
		expect(hook.getKnownAddr('peer-1')).toEqual([7, 8]);
		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.getNetworkStatus);
	});

	it('updates status when a network:status event arrives', async () => {
		const { callbacks } = setupListeners();
		defaultInvokeImpl();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.status)?.({
			payload: { mode: 'local', peers: [] }
		});

		expect(hook.status).toEqual({ mode: 'local', peers: [] });
	});

	it('re-syncs status and paired peers when a handshake completes', async () => {
		const { callbacks } = setupListeners();
		defaultInvokeImpl();
		const hook = await renderHook();
		await hook.startListening();

		invokeMock.mockClear();
		callbacks.get(NETWORK_EVENTS.deviceInfoReceived)?.({ payload: 'peer-2' });
		await Promise.resolve();
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.getNetworkStatus);
		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.getPairedPeers);
	});

	it('connectionCode is undefined until local address is loaded', async () => {
		defaultInvokeImpl();
		const hook = await renderHook();

		expect(hook.connectionCode()).toBeUndefined();

		await hook.loadLocalInfo();

		expect(hook.connectionCode()).toMatch(/^acerola1:/);
	});

	it('connectWithCode decodes the code, connects and refreshes peers', async () => {
		setupListeners();
		defaultInvokeImpl();
		const hook = await renderHook();

		const addr: LocalPeerAddr = { id: { id: 'remote-1', device_id: null }, addrs: [4, 5] };
		const code = encodeConnectionCode(addr);

		await hook.connectWithCode(code);

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.connectToPeer, {
			peerId: 'remote-1',
			addrs: [4, 5],
			alpn: 'acerola/handshake/1'
		});
		expect(hook.getKnownAddr('remote-1')).toEqual([4, 5]);
		expect(hook.connecting).toBe(false);
	});

	it('connectWithCode rejects with InvalidConnectionCodeError for a malformed code, without calling the backend', async () => {
		defaultInvokeImpl();
		const hook = await renderHook();

		await expect(hook.connectWithCode('not-a-real-code')).rejects.toThrow(
			InvalidConnectionCodeError
		);
		expect(hook.connecting).toBe(false);
		expect(invokeMock).not.toHaveBeenCalledWith(
			NETWORK_COMMANDS.connectToPeer,
			expect.anything()
		);
	});

	it('resets connecting to false when the connect command fails', async () => {
		const addr: LocalPeerAddr = { id: { id: 'remote-2', device_id: null }, addrs: [] };
		const code = encodeConnectionCode(addr);

		defaultInvokeImpl();
		const hook = await renderHook();

		invokeMock.mockImplementation((command: string) => {
			if (command === NETWORK_COMMANDS.connectToPeer) {
				return Promise.reject(new Error('handshake failed'));
			}
			return Promise.resolve(undefined);
		});

		await expect(hook.connectWithCode(code)).rejects.toThrow('handshake failed');
		expect(hook.connecting).toBe(false);
	});

	it('removePeer drops the peer from the list and known addresses', async () => {
		const pairedPeers: PairedPeerPayload[] = [
			{ peerId: 'peer-3', addrs: [1], deviceName: 'Tablet' }
		];
		defaultInvokeImpl({
			[NETWORK_COMMANDS.getPairedPeers]: pairedPeers,
			[NETWORK_COMMANDS.removePairedPeer]: undefined
		});
		setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.removePeer('peer-3');

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.removePairedPeer, {
			peerId: 'peer-3'
		});
		expect(hook.pairedPeers).toEqual([]);
		expect(hook.getKnownAddr('peer-3')).toBeUndefined();
	});

	it('peerLabel prioritizes the live device name over the paired one', async () => {
		const pairedPeers: PairedPeerPayload[] = [
			{ peerId: 'peer-4', addrs: [], deviceName: 'Old Name' }
		];
		defaultInvokeImpl({ [NETWORK_COMMANDS.getPairedPeers]: pairedPeers });
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.status)?.({
			payload: {
				mode: 'local',
				peers: [{ peerId: 'peer-4', alpn: 'x', device: { name: 'Live Name', os: '', version: '' } }]
			}
		});

		expect(hook.peerLabel('peer-4')).toBe('Live Name');
	});

	it('peerLabel falls back to the paired device name, then to a shortened id', async () => {
		const pairedPeers: PairedPeerPayload[] = [
			{ peerId: 'peer-5', addrs: [], deviceName: 'Paired Name' }
		];
		defaultInvokeImpl({ [NETWORK_COMMANDS.getPairedPeers]: pairedPeers });
		setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		expect(hook.peerLabel('peer-5')).toBe('Paired Name');
		expect(hook.peerLabel('unknown-peer-id-1234567890')).toBe('unknown-…567890');
	});

	it('stopListening disposes the hook and stops mutating state afterwards', async () => {
		const { unlisteners } = setupListeners();
		defaultInvokeImpl();
		const hook = await renderHook();
		await hook.startListening();

		hook.stopListening();

		for (const unlisten of unlisteners.values()) {
			expect(unlisten).toHaveBeenCalledOnce();
		}

		// loadLocalInfo still calls the backend, but the `disposed` guard must prevent it
		// from writing into state after this instance has been torn down.
		expect(hook.localId).toBeUndefined();
		await hook.loadLocalInfo();
		expect(hook.localId).toBeUndefined();
	});
});
