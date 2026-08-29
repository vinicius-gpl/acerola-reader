import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import type { ComicSummary } from '$lib/contracts/network/network.payloads';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useRemoteLibrary } from './use-remote-library.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(),
	convertFileSrc: vi.fn((path: string) => `asset://${path}`)
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const convertFileSrcMock = vi.mocked(convertFileSrc);

async function renderHook() {
	let hook: ReturnType<typeof useRemoteLibrary> | undefined;

	render(HookHarness, {
		props: {
			create: () => useRemoteLibrary(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useRemoteLibrary>;
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

function comic(overrides: Partial<ComicSummary> = {}): ComicSummary {
	return { comicName: 'One Piece', chapterCount: 10, coverVersion: 1, ...overrides };
}

describe('useRemoteLibrary', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		invokeMock.mockResolvedValue(undefined);
	});

	it('registers listeners for library and cover query events', async () => {
		const { unlisteners } = setupListeners();
		const hook = await renderHook();

		await hook.startListening();

		expect(unlisteners.size).toBe(4);
	});

	it('queryRemoteLibrary marks the peer as loading and calls the backend', async () => {
		setupListeners();
		const hook = await renderHook();

		const pending = hook.queryRemoteLibrary('peer-1', [1, 2]);
		expect(hook.isLoading('peer-1')).toBe(true);
		await pending;

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.queryRemoteLibrary, {
			peerId: 'peer-1',
			addrs: [1, 2]
		});
	});

	it('clears a previous error for the peer as soon as queryRemoteLibrary is called again', async () => {
		setupListeners();
		invokeMock.mockRejectedValueOnce(new Error('peer offline'));
		const hook = await renderHook();

		await expect(hook.queryRemoteLibrary('peer-1b', [])).rejects.toThrow('peer offline');
		expect(hook.errorFor('peer-1b')).toBe('Error: peer offline');

		invokeMock.mockResolvedValueOnce(undefined);
		const pending = hook.queryRemoteLibrary('peer-1b', [1]);
		// Limpo de imediato, antes mesmo do invoke resolver.
		expect(hook.errorFor('peer-1b')).toBeUndefined();
		await pending;
	});

	it('sets the error and rethrows when queryRemoteLibrary fails', async () => {
		setupListeners();
		invokeMock.mockRejectedValueOnce(new Error('peer offline'));
		const hook = await renderHook();

		await expect(hook.queryRemoteLibrary('peer-2', [])).rejects.toThrow('peer offline');

		expect(hook.isLoading('peer-2')).toBe(false);
		expect(hook.errorFor('peer-2')).toBe('Error: peer offline');
	});

	it('libraryQueryResult stores comics for the peer and clears loading/errors', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-3', [1]);

		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({ peerId: 'peer-3', comics: [comic()] })
		});
		await Promise.resolve();

		expect(hook.comicsFor('peer-3')).toEqual([comic()]);
		expect(hook.isLoading('peer-3')).toBe(false);
		expect(hook.errorFor('peer-3')).toBeUndefined();
	});

	it('libraryQueryResult clears a pre-existing error entry for the peer', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-3b', []);
		callbacks.get(NETWORK_EVENTS.libraryQueryError)?.({
			payload: JSON.stringify({ peerId: 'peer-3b', message: 'previous failure' })
		});
		expect(hook.errorFor('peer-3b')).toBe('previous failure');

		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({ peerId: 'peer-3b', comics: [] })
		});
		await Promise.resolve();

		// `errors.delete(payload.peerId)` precisa realmente remover a entrada — não
		// basta `loading` ter sido limpo.
		expect(hook.errorFor('peer-3b')).toBeUndefined();
	});

	it('automatically queries covers for comics received in the library result', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-4', [9, 9]);
		invokeMock.mockClear();

		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({ peerId: 'peer-4', comics: [comic({ comicName: 'Naruto' })] })
		});
		await Promise.resolve();
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.queryRemoteCover, {
			peerId: 'peer-4',
			addrs: [9, 9],
			comicName: 'Naruto',
			knownVersion: null
		});
	});

	it('libraryQueryError sets the error message and clears loading for that peer', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-5', []);

		callbacks.get(NETWORK_EVENTS.libraryQueryError)?.({
			payload: JSON.stringify({ peerId: 'peer-5', message: 'timed out' })
		});

		expect(hook.isLoading('peer-5')).toBe(false);
		expect(hook.errorFor('peer-5')).toBe('timed out');
	});

	it('treats a JSON error payload that is not an object as a raw message', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		// Só é observável combinado com um peerId real, já que sem peerId o handler
		// descarta o evento (ver teste abaixo) — usamos o formato certo pra chegar no
		// fallback de mensagem crua sem cair nesse outro guard.
		await hook.queryRemoteLibrary('peer-5b', []);
		callbacks.get(NETWORK_EVENTS.libraryQueryError)?.({
			payload: JSON.stringify({ peerId: 'peer-5b', message: 42 })
		});

		expect(hook.errorFor('peer-5b')).toBeUndefined();
	});

	// Mutantes equivalentes de `parseErrorPayload` (ConditionalExpression e LogicalOperator em
	// 21:7/21:17, ObjectLiteral em 27:9) já documentados com `// Stryker disable next-line`
	// diretamente no source (use-remote-library.svelte.ts) — ver lá a explicação completa,
	// incluindo por que o `&&` → `||` também é equivalente (confirmado por Stryker de verdade,
	// não só por inspeção: um teste anterior desta suíte que supostamente o matava não matava).

	it('libraryQueryError with no peerId in the payload is ignored (no peer to attach the error to)', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-5c', []);

		callbacks.get(NETWORK_EVENTS.libraryQueryError)?.({
			payload: JSON.stringify({ message: 'no peer id here' })
		});

		// O guard `if (!peerId) return;` descarta o evento inteiro — loading continua
		// como estava (não foi limpo) e nenhum erro foi registrado pra peer-5c.
		expect(hook.isLoading('peer-5c')).toBe(true);
		expect(hook.errorFor('peer-5c')).toBeUndefined();
	});

	it('libraryQueryError with an empty-string peerId is also discarded by the guard', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.libraryQueryError)?.({
			payload: JSON.stringify({ peerId: '', message: 'should be discarded' })
		});

		// `!peerId` também precisa pegar peerId vazio (falsy), não só ausente —
		// nada deveria ter sido registrado sob a chave ''.
		expect(hook.errorFor('')).toBeUndefined();
	});

	it('coverQueryResult resolves and stores the cover path when status is changed', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-6',
				comicName: 'Bleach',
				status: 'changed',
				coverVersion: 3,
				path: 'C:\\covers\\bleach.jpg'
			})
		});

		expect(hook.coverPathFor('peer-6', 'Bleach')).toBe('asset://C:/covers/bleach.jpg');
	});

	it('coverQueryResult does not store a path when status is not_modified', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-7',
				comicName: 'Bleach',
				status: 'not_modified',
				coverVersion: 3,
				path: null
			})
		});

		expect(hook.coverPathFor('peer-7', 'Bleach')).toBeUndefined();
	});

	it('coverQueryResult does not store a path when status is not_modified even if a path is present', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-7c',
				comicName: 'Bleach',
				status: 'not_modified',
				coverVersion: 3,
				path: 'C:\\covers\\bleach.jpg'
			})
		});

		expect(hook.coverPathFor('peer-7c', 'Bleach')).toBeUndefined();
	});

	it('coverQueryResult does not store a path when status is changed but path is missing', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-7b',
				comicName: 'Bleach',
				status: 'changed',
				coverVersion: 3,
				path: null
			})
		});

		expect(hook.coverPathFor('peer-7b', 'Bleach')).toBeUndefined();
	});

	it('coverQueryResult does not store a path when the resolved artwork path is falsy', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		// Força `resolveArtworkPath` (que delega em `convertFileSrc`) a devolver um
		// valor falsy pra essa única chamada, simulando `resolved` vazio mesmo com
		// status "changed" e `path` presente.
		convertFileSrcMock.mockReturnValueOnce('');

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-7d',
				comicName: 'Ghost',
				status: 'changed',
				coverVersion: 1,
				path: 'C:\\covers\\ghost.jpg'
			})
		});

		// `if (resolved) coverPaths.set(key, resolved);` não deveria gravar nada
		// quando `resolved` é falsy.
		expect(hook.coverPathFor('peer-7d', 'Ghost')).toBeUndefined();
	});

	it('coverQueryResult with coverVersion null does not update the known-version cache', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		// `coverVersion: null` chega antes — o guard `!== null` não deveria gravar
		// nada em `knownCoverVersions` pra essa chave.
		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-nullver',
				comicName: 'Null Comic',
				status: 'not_modified',
				coverVersion: null,
				path: null
			})
		});

		await hook.queryRemoteLibrary('peer-nullver', [7]);
		invokeMock.mockClear();

		// Um quadrinho com o mesmo `coverVersion: null` chega na lista. Se
		// `knownCoverVersions` tivesse sido atualizado com `null` (mutante), a
		// comparação `knownCoverVersions.get(key) === comic.coverVersion` seria
		// `null === null` e a busca de capa seria pulada — o que não deve
		// acontecer, já que o cache nunca deveria ter sido escrito com `null`.
		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-nullver',
				comics: [{ comicName: 'Null Comic', chapterCount: 1, coverVersion: null }]
			})
		});
		await Promise.resolve();
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.queryRemoteCover, {
			peerId: 'peer-nullver',
			addrs: [7],
			comicName: 'Null Comic',
			knownVersion: null
		});
	});

	it('coverKey keys by peer AND comic name — same comic name for different peers does not collide', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-A',
				comicName: 'One Piece',
				status: 'changed',
				coverVersion: 1,
				path: 'C:\\covers\\peer-a.jpg'
			})
		});

		expect(hook.coverPathFor('peer-A', 'One Piece')).toBe('asset://C:/covers/peer-a.jpg');
		expect(hook.coverPathFor('peer-B', 'One Piece')).toBeUndefined();
	});

	it('skips re-querying a cover whose version already matches the known version', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		// Registra a versão conhecida via um coverQueryResult "not_modified" anterior.
		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-8',
				comicName: 'One Piece',
				status: 'not_modified',
				coverVersion: 5,
				path: null
			})
		});

		await hook.queryRemoteLibrary('peer-8', [1]);
		invokeMock.mockClear();

		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-8',
				comics: [comic({ comicName: 'One Piece', coverVersion: 5 })]
			})
		});
		await Promise.resolve();
		await Promise.resolve();

		expect(invokeMock).not.toHaveBeenCalledWith(
			NETWORK_COMMANDS.queryRemoteCover,
			expect.anything()
		);
	});

	it('fetchCoversFor does nothing for a peer whose addrs were never recorded via queryRemoteLibrary', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		// libraryQueryResult chega sem que `queryRemoteLibrary` tenha sido chamado antes pra
		// esse peer — não há `addrs` guardado.
		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({ peerId: 'peer-9', comics: [comic()] })
		});
		await Promise.resolve();
		await Promise.resolve();

		expect(invokeMock).not.toHaveBeenCalledWith(
			NETWORK_COMMANDS.queryRemoteCover,
			expect.anything()
		);
	});

	it('coverQueryError is a silent no-op', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		expect(() =>
			callbacks.get(NETWORK_EVENTS.coverQueryError)?.({ payload: 'ignored' })
		).not.toThrow();
	});

	it('comicsFor/errorFor/coverPathFor default to empty/undefined for unknown peers', async () => {
		const hook = await renderHook();

		expect(hook.comicsFor('unknown')).toEqual([]);
		expect(hook.errorFor('unknown')).toBeUndefined();
		expect(hook.coverPathFor('unknown', 'x')).toBeUndefined();
		expect(hook.isLoading('unknown')).toBe(false);
	});

	it('stopListening unregisters every listener', async () => {
		const { unlisteners } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		hook.stopListening();

		for (const unlisten of unlisteners.values()) {
			expect(unlisten).toHaveBeenCalledOnce();
		}
	});
});
