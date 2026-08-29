import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { error as tauriError } from '@tauri-apps/plugin-log';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useNetworkSync } from './use-network-sync.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn()
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const errorMock = vi.mocked(tauriError);

async function renderHook() {
	let hook: ReturnType<typeof useNetworkSync> | undefined;

	render(HookHarness, {
		props: {
			create: () => useNetworkSync(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useNetworkSync>;
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

describe('useNetworkSync', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		invokeMock.mockResolvedValue(undefined);
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('loads the persisted sync history log on startListening', async () => {
		setupListeners();
		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return Promise.resolve([
					{
						id: 3,
						peerId: 'peer-1',
						kind: 'history',
						status: 'complete',
						message: null,
						createdAt: 1000
					}
				]);
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		await hook.startListening();

		expect(hook.log).toHaveLength(1);
		expect(hook.log[0]).toMatchObject({
			id: -3,
			peerId: 'peer-1',
			kind: 'history',
			status: 'complete',
			message: 'peer-1',
			timestamp: 1000
		});
	});

	it('registers listeners for every sync event and appends live entries', async () => {
		const { callbacks, unlisteners } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		expect(unlisteners.size).toBe(11);

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log[0]).toMatchObject({ peerId: 'peer-1', kind: 'history', status: 'started' });

		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-1' });
		expect(hook.log[0]).toMatchObject({ peerId: 'peer-1', kind: 'history', status: 'complete' });
		expect(hook.log).toHaveLength(1);
	});

	it('parses JSON error payloads and clears the syncing flag for that peer', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `isSyncing` reflete um sync que ESTE cliente iniciou (via `withSyncGuard`), não o
		// evento `historyStarted` recebido do backend — esse evento só alimenta `log`.
		await hook.syncHistory('peer-1', [1, 2, 3]);
		expect(hook.isSyncing('peer-1', 'history')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ peerId: 'peer-1', message: 'stream closed' })
		});

		expect(hook.isSyncing('peer-1', 'history')).toBe(false);
		expect(hook.log[0]).toMatchObject({ status: 'error', message: 'stream closed' });
	});

	it('treats a non-JSON error payload as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesError)?.({ payload: 'connection refused' });

		expect(hook.log[0]).toMatchObject({ status: 'error', message: 'connection refused' });
	});

	it('files:progress entries always create a new row (no peerId to correlate)', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: '10%' });
		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: '20%' });

		expect(hook.log).toHaveLength(2);
		// Prepended (mais recente primeiro) com id sempre crescente: a 2ª entrada (topo) tem
		// id maior que a 1ª (mais antiga, agora em baixo).
		expect(hook.log[1].id).toBeLessThan(hook.log[0].id);
	});

	it('started -> progress -> complete updates the same row in place, then a new started creates a fresh row', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(1);
		const startedId = hook.log[0].id;

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(1);
		expect(hook.log[0].id).toBe(startedId);
		expect(hook.log[0].status).toBe('started');

		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(1);
		expect(hook.log[0].id).toBe(startedId);
		expect(hook.log[0].status).toBe('complete');

		// `complete` é terminal: o próximo `started` do mesmo peer/kind não pode reaproveitar
		// a linha já resolvida, tem que abrir uma linha nova.
		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(2);
		expect(hook.log[0].id).not.toBe(startedId);
		expect(hook.log[0].status).toBe('started');
	});

	it('isSyncing keys peer and kind together, independently of other peers/kinds', async () => {
		setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncHistory('peer-a', []);

		expect(hook.isSyncing('peer-a', 'history')).toBe(true);
		expect(hook.isSyncing('peer-a', 'files')).toBe(false);
		expect(hook.isSyncing('peer-b', 'history')).toBe(false);

		resolveSync();
		await pending;
	});

	it('caps the log at MAX_LOG_ENTRIES, dropping the oldest entries', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		for (let i = 0; i < 201; i++) {
			callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: `peer-${i}` });
		}

		expect(hook.log).toHaveLength(200);
		// A mais recente (peer-200) fica, a mais antiga (peer-0) foi descartada.
		expect(hook.log[0]).toMatchObject({ peerId: 'peer-200' });
		expect(hook.log.some((entry) => entry.peerId === 'peer-0')).toBe(false);
	});

	it('logs when loading the persisted log fails, without throwing', async () => {
		setupListeners();
		invokeMock.mockRejectedValue(new Error('db locked'));

		const hook = await renderHook();
		await expect(hook.startListening()).resolves.toBeUndefined();

		expect(errorMock).toHaveBeenCalledWith(
			expect.stringContaining('failed to load persisted sync history log: Error: db locked')
		);
		expect(hook.log).toEqual([]);
	});

	it('treats a JSON error payload that is not an object as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyError)?.({ payload: '42' });

		expect(hook.log[0]).toMatchObject({ status: 'error', message: '42', peerId: '' });
	});

	it('treats a JSON error payload without a string message field as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		const payload = JSON.stringify({ peerId: 'peer-1' });
		callbacks.get(NETWORK_EVENTS.filesError)?.({ payload });

		expect(hook.log[0]).toMatchObject({ status: 'error', message: payload, peerId: '' });
	});

	it('does not clear a syncing flag for an error payload missing peerId', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncHistory('peer-7', []);
		expect(hook.isSyncing('peer-7', 'history')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ message: 'no peer id here' })
		});

		expect(hook.isSyncing('peer-7', 'history')).toBe(true);

		resolveSync();
		await pending;
	});

	it('syncHistory calls the backend and guards against a concurrent duplicate call', async () => {
		setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);

		const first = hook.syncHistory('peer-1', [1, 2, 3]);
		expect(hook.isSyncing('peer-1', 'history')).toBe(true);

		await hook.syncHistory('peer-1', [1, 2, 3]);

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncHistory, {
			peerId: 'peer-1',
			addrs: [1, 2, 3]
		});
		expect(invokeMock).toHaveBeenCalledTimes(1);

		resolveSync();
		await first;
	});

	it('clears the syncing flag and rethrows when the sync command fails', async () => {
		setupListeners();
		const failure = new Error('peer unreachable');
		invokeMock.mockRejectedValueOnce(failure);
		const hook = await renderHook();

		await expect(hook.syncFiles('peer-2', [])).rejects.toThrow(failure);
		expect(hook.isSyncing('peer-2', 'files')).toBe(false);
	});

	it('syncAll marks both history and files kinds as syncing', async () => {
		setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncAll('peer-3', []);

		expect(hook.isSyncing('peer-3', 'history')).toBe(true);
		expect(hook.isSyncing('peer-3', 'files')).toBe(true);

		resolveSync();
		await pending;
	});

	it('syncComic calls the backend with the comic name and guards the "comic" kind specifically', async () => {
		setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncComic('peer-4', [9], 'One Piece');

		// Guarda especificamente o kind 'comic' — não 'history'/'files', que syncComic nem
		// toca.
		expect(hook.isSyncing('peer-4', 'comic')).toBe(true);
		expect(hook.isSyncing('peer-4', 'history')).toBe(false);
		expect(hook.isSyncing('peer-4', 'files')).toBe(false);

		resolveSync();
		await pending;

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncComic, {
			peerId: 'peer-4',
			addrs: [9],
			comicName: 'One Piece'
		});
	});

	it('lastSyncedAt returns the timestamp of the most recent complete entry for a peer', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		expect(hook.lastSyncedAt('peer-5')).toBeUndefined();

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-5' });
		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-5' });

		expect(hook.lastSyncedAt('peer-5')).toBeDefined();
	});

	it('lastSyncedAt ignores entries from other peers and non-complete entries of the same peer', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		// peer-6 sincronizou com sucesso.
		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-6' });
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-6' });

		// peer-7 tem uma sessão em aberto (não completa) e nada de "complete".
		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-7' });

		expect(hook.lastSyncedAt('peer-6')).toBeDefined();
		// Só tem entrada "started" pra peer-7 — não pode "vazar" o timestamp de outro peer
		// nem contar um status não-complete como sync concluído.
		expect(hook.lastSyncedAt('peer-7')).toBeUndefined();
	});

	it('stopListening unregisters every listener', async () => {
		const { unlisteners } = setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		hook.stopListening();

		for (const unlisten of unlisteners.values()) {
			expect(unlisten).toHaveBeenCalledOnce();
		}
	});

	it('auto-clears an in-flight syncing flag after the timeout elapses', async () => {
		vi.useFakeTimers();
		setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncHistory('peer-6', []);
		expect(hook.isSyncing('peer-6', 'history')).toBe(true);

		vi.advanceTimersByTime(60_000);
		expect(hook.isSyncing('peer-6', 'history')).toBe(false);

		resolveSync();
		await pending;
	});
});
