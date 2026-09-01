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
					},
					{
						id: 4,
						peerId: 'peer-2',
						kind: 'history',
						status: 'error',
						message: 'boom',
						createdAt: 2000
					}
				]);
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		await hook.startListening();

		expect(hook.log).toHaveLength(2);
		expect(hook.log[0]).toMatchObject({
			id: -3,
			peerId: 'peer-1',
			kind: 'history',
			status: 'complete',
			// Não-erro: `message` carrega o peer id (usado por `peers.peerLabel`), não o
			// texto de erro (que nem existe aqui, é `null`).
			message: 'peer-1',
			timestamp: 1000
		});
		expect(hook.log[1]).toMatchObject({
			id: -4,
			peerId: 'peer-2',
			kind: 'history',
			status: 'error',
			// Erro: `message` carrega o texto de erro de verdade, NÃO o peerId.
			message: 'boom',
			timestamp: 2000
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
		expect(hook.log[0]).toMatchObject({
			kind: 'history',
			status: 'error',
			message: 'stream closed'
		});
	});

	it('treats a non-JSON error payload as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesError)?.({ payload: 'connection refused' });

		expect(hook.log[0]).toMatchObject({
			kind: 'files',
			status: 'error',
			message: 'connection refused'
		});
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

	it('an error is a terminal status too: a later started for the same peer/kind opens a fresh row', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-16' });
		const startedId = hook.log[0].id;

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ peerId: 'peer-16', message: 'boom' })
		});
		expect(hook.log).toHaveLength(1);
		expect(hook.log[0].id).toBe(startedId);
		expect(hook.log[0].status).toBe('error');

		// `error` fecha a sessão (igual `complete`) — o próximo `started` não pode
		// reaproveitar a linha já resolvida com erro.
		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-16' });
		expect(hook.log).toHaveLength(2);
		expect(hook.log[0].id).not.toBe(startedId);
		expect(hook.log[0].status).toBe('started');
	});

	it('a complete/error entry created without a prior in-flight row is not incorrectly reused by a later started event', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `historyComplete` chega sem nenhum `historyStarted` antes — cria linha nova, mas
		// como `complete` não é um status "aberto", essa linha NÃO pode ficar correlacionável
		// (senão o próximo `started` do mesmo peer/kind atualiza a linha já resolvida em vez
		// de abrir uma nova).
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-17' });
		expect(hook.log).toHaveLength(1);
		const completeId = hook.log[0].id;
		expect(hook.log[0].status).toBe('complete');

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-17' });
		expect(hook.log).toHaveLength(2);
		expect(hook.log[0].id).not.toBe(completeId);
		expect(hook.log[0].status).toBe('started');
		expect(hook.log[1].id).toBe(completeId);
		expect(hook.log[1].status).toBe('complete');
	});

	it('updates the correct in-flight entry when sessions are interleaved (not just the most recent log row)', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-A' });
		const peerAId = hook.log[0].id;

		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-B' });
		const peerBId = hook.log[0].id;
		expect(hook.log).toHaveLength(2);

		// peer-A não está mais no topo do log (peer-B foi prepended por cima) — só a
		// entrada com o `id` certo pode ser atualizada, não a mais recente do array.
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-A' });

		expect(hook.log).toHaveLength(2);
		const entryA = hook.log.find((entry) => entry.id === peerAId);
		const entryB = hook.log.find((entry) => entry.id === peerBId);
		expect(entryA).toMatchObject({ peerId: 'peer-A', kind: 'history', status: 'complete' });
		expect(entryB).toMatchObject({ peerId: 'peer-B', kind: 'files', status: 'started' });
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

	it('caps the log at MAX_LOG_ENTRIES, dropping the oldest resolved entries', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// Cada sessão termina (started -> complete) antes da próxima começar, então nenhuma
		// fica "pendente" — o corte deve valer normalmente pra elas, igual antes.
		for (let i = 0; i < 201; i++) {
			callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: `peer-${i}` });
			callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: `peer-${i}` });
		}

		expect(hook.log).toHaveLength(200);
		// A mais recente (peer-200) fica, a mais antiga (peer-0) foi descartada.
		expect(hook.log[0]).toMatchObject({ peerId: 'peer-200' });
		expect(hook.log.some((entry) => entry.peerId === 'peer-0')).toBe(false);
	});

	/// Reproduz o bug real: uma sincronização grande gera muitos eventos `sync:comic:progress`
	/// (sem peer id — ver comentário de `appendNewEntry`), cada um virando uma linha nova no
	/// log. Antes do fix, isso enchia o corte de `MAX_LOG_ENTRIES` e empurrava a linha "started"
	/// (a única rastreada por peer, em `inFlightEntryId`) pra fora do array antes do evento
	/// `complete` chegar — a linha ficava presa girando "sincronizando..." pra sempre na tela de
	/// Rede, mesmo com a sessão já concluída no backend.
	it('keeps an in-flight entry past MAX_LOG_ENTRIES until it resolves, instead of losing it silently', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-stuck' });
		for (let i = 0; i < 200; i++) {
			callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: `chapter-${i}` });
		}

		expect(
			hook.log.find((entry) => entry.peerId === 'peer-stuck' && entry.status === 'started')
		).toBeDefined();

		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-stuck' });

		expect(hook.log.find((entry) => entry.peerId === 'peer-stuck')?.status).toBe('complete');
		expect(hook.log.length).toBeLessThanOrEqual(201);
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

	it('discards a successful persisted-log load if stopListening ran before it resolved', async () => {
		setupListeners();
		let resolveLoad: (rows: unknown[]) => void = () => {};
		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return new Promise((resolve) => {
					resolveLoad = resolve;
				});
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		const startPromise = hook.startListening();
		hook.stopListening();

		resolveLoad([
			{ id: 1, peerId: 'peer-1', kind: 'history', status: 'complete', message: null, createdAt: 1 }
		]);
		await startPromise;

		// `disposed` já era true quando a promise resolveu — o resultado tem que ser
		// descartado, não gravado em `log`.
		expect(hook.log).toEqual([]);
	});

	it('discards a persisted-log load failure if stopListening ran before it rejected (does not log)', async () => {
		setupListeners();
		let rejectLoad: (err: unknown) => void = () => {};
		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return new Promise((_resolve, reject) => {
					rejectLoad = reject;
				});
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		const startPromise = hook.startListening();
		hook.stopListening();

		rejectLoad(new Error('db locked'));
		await startPromise;

		// `disposed` já era true quando a promise rejeitou — não deve logar o erro.
		expect(errorMock).not.toHaveBeenCalled();
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

	it('treats a JSON error payload with a non-string message field as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `message` presente mas não-string (número): `typeof parsed.message === 'string'`
		// tem que ser falso de verdade, não só "existe uma chave message".
		const payload = JSON.stringify({ peerId: 'peer-1', message: 123 });
		callbacks.get(NETWORK_EVENTS.filesError)?.({ payload });

		expect(hook.log[0]).toMatchObject({ status: 'error', message: payload, peerId: '' });
		expect(typeof hook.log[0].message).toBe('string');
	});

	it('treats a JSON error payload that parses to a truthy non-object primitive as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `JSON.parse('42')` é um valor truthy (número), mas não é um objeto — a condição
		// tem que checar `typeof parsed === 'object'` de verdade, não só a truthiness de
		// `parsed`, senão isso vaza um `{ peerId: undefined, message: undefined }`.
		callbacks.get(NETWORK_EVENTS.historyError)?.({ payload: '42' });

		expect(hook.log[0]).toMatchObject({
			kind: 'history',
			status: 'error',
			message: '42',
			peerId: ''
		});
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

	it('does not clear a syncing flag when the error payload resolves to an empty (falsy) peerId', async () => {
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
		// peerId vazio: `syncKey('', 'history')` é uma chave distinta de qualquer peer real,
		// mas serve pra provar que o `if (peerId)` — não um `if (true)` — é o que decide se
		// `clearSyncing` roda.
		const pending = hook.syncHistory('', []);
		expect(hook.isSyncing('', 'history')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ peerId: '', message: 'boom' })
		});

		expect(hook.isSyncing('', 'history')).toBe(true);

		resolveSync();
		await pending;
	});

	it('historyComplete clears the syncing flag for the "history" kind specifically, not "files"', async () => {
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
		const pending = hook.syncAll('peer-9', []);
		expect(hook.isSyncing('peer-9', 'history')).toBe(true);
		expect(hook.isSyncing('peer-9', 'files')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-9' });

		expect(hook.isSyncing('peer-9', 'history')).toBe(false);
		expect(hook.isSyncing('peer-9', 'files')).toBe(true);

		resolveSync();
		await pending;
	});

	it('comicComplete clears the syncing flag for the "comic" kind specifically', async () => {
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
		const pending = hook.syncComic('peer-12', [], 'Some Comic');
		expect(hook.isSyncing('peer-12', 'comic')).toBe(true);

		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-12' });

		expect(hook.isSyncing('peer-12', 'comic')).toBe(false);

		resolveSync();
		await pending;
	});

	it('filesError clears the syncing flag for the "files" kind when the payload resolves to a peerId', async () => {
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
		const pending = hook.syncFiles('peer-10', []);
		expect(hook.isSyncing('peer-10', 'files')).toBe(true);

		callbacks.get(NETWORK_EVENTS.filesError)?.({
			payload: JSON.stringify({ peerId: 'peer-10', message: 'boom' })
		});

		expect(hook.isSyncing('peer-10', 'files')).toBe(false);

		resolveSync();
		await pending;
	});

	it('does not clear a "files" syncing flag when a filesError payload resolves to an empty peerId', async () => {
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
		const pending = hook.syncFiles('', []);
		expect(hook.isSyncing('', 'files')).toBe(true);

		callbacks.get(NETWORK_EVENTS.filesError)?.({
			payload: JSON.stringify({ peerId: '', message: 'boom' })
		});

		expect(hook.isSyncing('', 'files')).toBe(true);

		resolveSync();
		await pending;
	});

	it('each live event pushes a log entry with the exact kind and status for that event', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-f1' });
		expect(hook.log[0]).toMatchObject({ kind: 'files', status: 'started', peerId: 'peer-f1' });

		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: '50%' });
		expect(hook.log[0]).toMatchObject({
			kind: 'files',
			status: 'progress',
			peerId: '',
			message: '50%'
		});

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-c1' });
		expect(hook.log[0]).toMatchObject({ kind: 'comic', status: 'started', peerId: 'peer-c1' });

		callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: '75%' });
		expect(hook.log[0]).toMatchObject({
			kind: 'comic',
			status: 'progress',
			peerId: '',
			message: '75%'
		});

		// peer diferente de peer-c1 de propósito, pra não correlacionar com a entrada
		// "started" acima e continuar entrando como uma linha nova no topo (índice 0).
		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-c2' });
		expect(hook.log[0]).toMatchObject({ kind: 'comic', status: 'complete', peerId: 'peer-c2' });
	});

	it('clearSyncing cancels the pending in-flight timeout and removes its entry from the map', async () => {
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
		const pending = hook.syncHistory('peer-15', []);
		expect(hook.isSyncing('peer-15', 'history')).toBe(true);

		const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout');
		const deleteSpy = vi.spyOn(Map.prototype, 'delete');
		clearTimeoutSpy.mockClear();
		deleteSpy.mockClear();

		// Nenhum `historyStarted` foi disparado pra peer-15, então `inFlightEntryId` não tem
		// entrada pra essa chave. As duas chamadas de `.delete('peer-15:history')` esperadas
		// aqui vêm de `syncingKeys.delete(key)` (SvelteSet, que delega pro seu Map interno de
		// sources) e de `inFlightTimeouts.delete(key)`, ambas dentro de `clearSyncing`.
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-15' });

		expect(clearTimeoutSpy).toHaveBeenCalledTimes(1);
		const deleteCallsForKey = deleteSpy.mock.calls.filter((call) => call[0] === 'peer-15:history');
		expect(deleteCallsForKey).toHaveLength(2);

		clearTimeoutSpy.mockRestore();
		deleteSpy.mockRestore();

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

	it('syncFiles marks "files" specifically (not another kind), calls the backend with the right args, and clears + rethrows on failure', async () => {
		setupListeners();
		const failure = new Error('peer unreachable');
		let rejectSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((_resolve, reject) => {
					rejectSync = () => reject(failure);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncFiles('peer-2', []);
		// Marca a chave 'files' — não 'history'/'comic', que syncFiles nem toca.
		expect(hook.isSyncing('peer-2', 'files')).toBe(true);
		expect(hook.isSyncing('peer-2', 'history')).toBe(false);
		expect(hook.isSyncing('peer-2', 'comic')).toBe(false);
		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncFiles, {
			peerId: 'peer-2',
			addrs: []
		});

		rejectSync();
		await expect(pending).rejects.toThrow(failure);
		expect(hook.isSyncing('peer-2', 'files')).toBe(false);
	});

	it('withSyncGuard bails out if ANY requested kind is already syncing, not only when ALL are', async () => {
		setupListeners();
		let resolveHistorySync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveHistorySync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const historyPending = hook.syncHistory('peer-13', []);
		expect(hook.isSyncing('peer-13', 'history')).toBe(true);
		expect(hook.isSyncing('peer-13', 'files')).toBe(false);

		// 'history' já está sincronizando — syncAll(['history','files']) tem que abortar
		// por completo (nem marcar 'files', nem chamar o backend de novo), não só quando
		// TODOS os kinds pedidos já estiverem sincronizando.
		await hook.syncAll('peer-13', []);

		expect(hook.isSyncing('peer-13', 'files')).toBe(false);
		expect(invokeMock).not.toHaveBeenCalledWith(NETWORK_COMMANDS.syncAll, expect.anything());

		resolveHistorySync();
		await historyPending;
	});

	it('syncAll marks both history and files kinds as syncing and calls the backend with the right args', async () => {
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
		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncAll, {
			peerId: 'peer-3',
			addrs: []
		});

		resolveSync();
		await pending;
	});

	it('syncComic calls the backend with the comic name and guards the "comic" kind specifically', async () => {
		const { callbacks } = setupListeners();
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
		// O `invoke` resolver só enfileira a conexão — sem a sessão P2P ter "terminado" de
		// verdade (evento abaixo), a promise de `syncComic` não resolve, então nada além do
		// invoke pode ser checado até ali.
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncComic, {
			peerId: 'peer-4',
			addrs: [9],
			comicName: 'One Piece'
		});
		expect(hook.isSyncing('peer-4', 'comic')).toBe(true);

		await hook.startListening();
		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-4' });

		await expect(pending).resolves.toBe('peer-4');
		expect(hook.isSyncing('peer-4', 'comic')).toBe(false);
	});

	it('syncComic rejects with the real error once sync:comic:error arrives, not just on an immediate invoke failure', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();
		await hook.startListening();

		const pending = hook.syncComic('peer-4', [9], 'One Piece');
		expect(hook.isSyncing('peer-4', 'comic')).toBe(true);

		callbacks.get(NETWORK_EVENTS.comicError)?.({
			payload: JSON.stringify({ peerId: 'peer-4', message: 'peer disconnected' })
		});

		await expect(pending).rejects.toBe('peer disconnected');
		expect(hook.isSyncing('peer-4', 'comic')).toBe(false);
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

	it('stopListening cancels pending in-flight timeouts so they cannot fire later', async () => {
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

		const pending = hook.syncHistory('peer-14', []);
		expect(hook.isSyncing('peer-14', 'history')).toBe(true);

		hook.stopListening();
		vi.advanceTimersByTime(60_000);

		// Se o timeout de 60s não tivesse sido cancelado no stopListening, ele dispararia
		// `clearSyncing` aqui e apagaria a flag mesmo sem nenhum evento de conclusão/erro.
		expect(hook.isSyncing('peer-14', 'history')).toBe(true);

		resolveSync();
		await pending;
	});

	it('stopListening clears the in-flight timeout and entry-id maps', async () => {
		setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		const clearSpy = vi.spyOn(Map.prototype, 'clear');
		clearSpy.mockClear();

		hook.stopListening();

		// `inFlightTimeouts.clear()` + `inFlightEntryId.clear()` + `pendingSettlement.clear()`
		// — exatamente três chamadas de `Map.prototype.clear` nesta janela síncrona.
		expect(clearSpy).toHaveBeenCalledTimes(3);

		clearSpy.mockRestore();
	});
});
