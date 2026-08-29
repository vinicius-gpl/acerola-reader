import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { replaceState } from '$app/navigation';
import { page } from '$app/state';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';
import { LIBRARY_EVENTS } from '$lib/contracts/library/chapter.events';
import { SESSION_KEYS } from '$lib/constants/session-keys';
import type { ChapterFilePayload } from '$lib/contracts/library/chapter.payloads';
import type { ReaderChapterPayload } from '$lib/contracts/reader/reader.payloads';
import ReaderNavigationHarness from '../../../../tests/harness/hooks/reader-navigation-store.svelte';
import { useReaderNavigation, type ReaderNavigationState } from './use-reader-navigation.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const replaceStateMock = vi.mocked(replaceState);

function chapterFile(overrides: Partial<ChapterFilePayload> = {}): ChapterFilePayload {
	return {
		id: 'ch-1',
		name: 'Chapter 1',
		path: '/comics/ch-1.cbz',
		chapterSort: '1',
		volumeId: null,
		volumeName: null,
		isSpecial: false,
		lastModified: 0,
		...overrides
	};
}

function readerChapter(overrides: Partial<ReaderChapterPayload> = {}): ReaderChapterPayload {
	return chapterFile(overrides);
}

async function renderHook() {
	let hook: ReturnType<typeof useReaderNavigation> | undefined;

	render(ReaderNavigationHarness, {
		props: {
			onReady: (value) => {
				hook = value;
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

// jsdom implementa `Storage` com um Proxy interno, então `vi.spyOn(sessionStorage, ...)`
// não intercepta as chamadas (a call fica invisível pro spy). Trocamos o global inteiro
// por um mock em memória — mesmo padrão já usado pro `localStorage` em tests/setup.ts —
// pra conseguir contar chamadas de verdade e distinguir a escrita de `resolveInitialState`
// da escrita duplicada do `$effect` (linhas 48-55), que sempre re-persiste o mesmo pageState.
const originalSessionStorageDescriptor = Object.getOwnPropertyDescriptor(window, 'sessionStorage');

function installSessionStorageMock() {
	const store = new Map<string, string>();
	const mock = {
		getItem: vi.fn((key: string) => (store.has(key) ? (store.get(key) as string) : null)),
		setItem: vi.fn((key: string, value: string) => {
			store.set(key, value);
		}),
		removeItem: vi.fn((key: string) => {
			store.delete(key);
		}),
		clear: vi.fn(() => store.clear())
	};

	Object.defineProperty(window, 'sessionStorage', {
		value: mock,
		writable: true,
		configurable: true
	});

	return mock;
}

let sessionStorageMock: ReturnType<typeof installSessionStorageMock>;

describe('useReaderNavigation', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		invokeMock.mockResolvedValue(undefined);
		sessionStorageMock = installSessionStorageMock();
		page.state = {};
	});

	afterEach(() => {
		page.state = {};
		if (originalSessionStorageDescriptor) {
			Object.defineProperty(window, 'sessionStorage', originalSessionStorageDescriptor);
		}
	});

	it('resolves initial state from page.state when a chapter is present, and persists it', async () => {
		const navState: ReaderNavigationState = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-1',
			chapterIndex: 0,
			totalChapters: 3
		};
		page.state = navState;
		setupListeners();

		const hook = await renderHook();

		expect(hook.chapter).toEqual(navState.chapter);
		expect(hook.chapterIndex).toBe(0);
		expect(hook.totalChapters).toBe(3);
		expect(JSON.parse(sessionStorage.getItem(SESSION_KEYS.readerState) ?? '{}')).toMatchObject({
			comicDirectoryId: 'dir-1'
		});
	});

	it('falls back to sessionStorage when page.state has no chapter', async () => {
		const saved: ReaderNavigationState = {
			chapter: readerChapter({ id: 'saved-chapter' }),
			comicDirectoryId: 'dir-2',
			chapterIndex: 1,
			totalChapters: 5
		};
		sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(saved));
		page.state = {};
		setupListeners();

		const hook = await renderHook();

		expect(hook.chapter?.id).toBe('saved-chapter');
		expect(hook.chapterIndex).toBe(1);
	});

	it('starts with an undefined chapter and derived values when there is no state at all', async () => {
		page.state = {};
		setupListeners();

		const hook = await renderHook();

		expect(hook.chapter).toBeUndefined();
		expect(hook.chapterIndex).toBeUndefined();
		expect(hook.totalChapters).toBeUndefined();
		expect(hook.chaptersRemaining).toBeUndefined();
		expect(hook.hasNextChapter).toBe(false);
		expect(hook.hasPreviousChapter).toBe(false);
		expect(hook.initializing).toBe(false);
	});

	it('falls back to sessionStorage when page.state has keys but no chapter field', async () => {
		const saved: ReaderNavigationState = {
			chapter: readerChapter({ id: 'saved-chapter' }),
			comicDirectoryId: 'dir-2b'
		};
		sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(saved));
		// Tem chaves (length > 0) mas nenhuma delas é `chapter` — não pode ser tratado como
		// estado válido só por não estar vazio.
		page.state = { chapterScope: 'all' };
		setupListeners();

		const hook = await renderHook();

		expect(hook.chapter?.id).toBe('saved-chapter');
	});

	it('does not read sessionStorage or persist anything when resolving initial state outside a browser environment', async () => {
		// `browser` é mockado como `true` globalmente (tests/setup.ts), então pra exercitar o
		// branch `!browser` precisamos de uma cópia fresca do módulo com `$app/environment`
		// remockado. `vi.resetModules()` também reseta o `svelte` interno — por isso chamamos o
		// hook FORA de um componente, sem `render()`: `$effect`/`onMount` exigem um contexto de
		// componente e lançam `effect_orphan` de propósito, mas isso só acontece DEPOIS de
		// `resolveInitialState()` já ter rodado de forma síncrona, então o efeito colateral dela
		// (ou a ausência dele) já é observável antes do throw.
		vi.resetModules();
		vi.doMock('$app/environment', () => ({
			browser: false,
			dev: true,
			building: false,
			version: 'test'
		}));

		try {
			const { page: freshPage } = await import('$app/state');
			freshPage.state = { chapter: readerChapter(), comicDirectoryId: 'dir-ssr' };

			const freshModule = await import('./use-reader-navigation.svelte');

			expect(() => freshModule.useReaderNavigation()).toThrow('effect_orphan');
			expect(sessionStorageMock.setItem).not.toHaveBeenCalled();
		} finally {
			vi.doUnmock('$app/environment');
			vi.resetModules();
		}
	});

	// `resolveInitialState()` (linhas 30-44) e o `$effect` de sincronização (linhas 48-55) checam
	// exatamente a mesma condição sobre o mesmo `page.state` — e o `render()` do
	// @testing-library/svelte roda `flushSync()` internamente, então os dois já rodaram por
	// completo antes do `render()` retornar. Isso significa que, quando `page.state` tem um
	// `chapter`, o `$effect` sempre "conserta" o `state` resultante pro mesmo valor
	// independentemente do que `resolveInitialState()` computou sozinho — o único jeito de
	// distinguir um mutante isolado de UM dos dois é contar quantas vezes `sessionStorage.setItem`
	// foi chamado no total (cada um escreve o seu próprio `setItem`, de forma redundante).
	it('persists page.state exactly twice on mount — once from resolveInitialState, once from the sync effect', async () => {
		page.state = {
			chapter: readerChapter({ id: 'own-chapter' }),
			comicDirectoryId: 'dir-persist-twice'
		};
		setupListeners();

		await renderHook();

		expect(sessionStorageMock.setItem).toHaveBeenCalledTimes(2);
	});

	it('does not persist via resolveInitialState or the sync effect when the chapter is only inherited (not an own key) on page.state', async () => {
		// `Object.keys(pageState).length > 0` só conta propriedades PRÓPRIAS — um `chapter`
		// herdado do protótipo deixa `Object.keys(...).length` em 0 mesmo com `pageState.chapter`
		// acessível, isolando o boundary do `.length > 0` do boundary da presença do `chapter`.
		const chapterProto = { chapter: readerChapter({ id: 'inherited-chapter' }) };
		page.state = Object.create(chapterProto) as ReaderNavigationState;
		setupListeners();

		await renderHook();

		expect(sessionStorageMock.setItem).not.toHaveBeenCalled();
	});

	it('clamps a negative chapterIndex/totalChapters to 0 and rejects non-finite or non-number values', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-clamp',
			chapterIndex: -5,
			totalChapters: Infinity
		};
		setupListeners();

		const hook = await renderHook();

		expect(hook.chapterIndex).toBe(0);
		expect(hook.totalChapters).toBeUndefined();
	});

	it('rejects a non-number-typed chapterIndex/totalChapters (not just non-finite numbers)', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-clamp-typeof',
			// `as unknown as number` simula um valor inválido em runtime — o guard de tipo
			// (`typeof value === 'number'`) é o que barra isso, não o `Number.isFinite`.
			chapterIndex: 'not-a-number' as unknown as number,
			totalChapters: 'not-a-number' as unknown as number
		};
		setupListeners();

		const hook = await renderHook();

		expect(hook.chapterIndex).toBeUndefined();
		expect(hook.totalChapters).toBeUndefined();
	});

	it('rejects a non-finite numeric chapterIndex (NaN) even though typeof is "number"', async () => {
		// Distingue o `&&` de um `||` mutante: com `||`, `typeof value === 'number'` (true) já
		// bastaria pra devolver `Math.max(0, NaN)` em vez de `undefined`.
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-clamp-nan-index',
			chapterIndex: NaN,
			totalChapters: 3
		};
		setupListeners();

		const hook = await renderHook();

		expect(hook.chapterIndex).toBeUndefined();
	});

	it('rejects a non-finite numeric totalChapters (NaN) even though typeof is "number"', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-clamp-nan-total',
			chapterIndex: 0,
			totalChapters: NaN
		};
		setupListeners();

		const hook = await renderHook();

		expect(hook.totalChapters).toBeUndefined();
	});

	it('chaptersRemaining is undefined when either chapterIndex or totalChapters alone is missing', async () => {
		setupListeners();

		page.state = { chapter: readerChapter(), comicDirectoryId: 'dir-a', totalChapters: 3 };
		const hookMissingIndex = await renderHook();
		expect(hookMissingIndex.chaptersRemaining).toBeUndefined();
	});

	it('chaptersRemaining is undefined when chapterIndex is present but totalChapters alone is missing', async () => {
		page.state = { chapter: readerChapter(), comicDirectoryId: 'dir-b', chapterIndex: 0 };
		setupListeners();

		const hook = await renderHook();

		expect(hook.chaptersRemaining).toBeUndefined();
	});

	it('hasNextChapter/hasPreviousChapter reflect chapterIndex and totalChapters', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-3',
			chapterIndex: 1,
			totalChapters: 3
		};
		setupListeners();

		const hook = await renderHook();

		expect(hook.hasPreviousChapter).toBe(true);
		expect(hook.hasNextChapter).toBe(true);
		expect(hook.chaptersRemaining).toBe(1);
	});

	it('hasNextChapter is false at the last chapter and hasPreviousChapter is false at the first', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-4',
			chapterIndex: 0,
			totalChapters: 1
		};
		setupListeners();

		const hook = await renderHook();

		expect(hook.hasPreviousChapter).toBe(false);
		expect(hook.hasNextChapter).toBe(false);
	});

	it('loadContext calls the backend with the current chapter context', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-5',
			sortBy: 'number_desc'
		};
		setupListeners();
		const hook = await renderHook();

		await hook.loadContext();

		expect(invokeMock).toHaveBeenCalledWith(LIBRARY_COMMANDS.getComicChapters, {
			comicDirectoryFk: 'dir-5',
			volumeId: null,
			page: 0,
			pageSize: 99999,
			sortBy: 'number_desc',
			searchQuery: null
		});
	});

	it('loadContext does nothing without a chapter or comicDirectoryId', async () => {
		page.state = {};
		setupListeners();
		const hook = await renderHook();

		await hook.loadContext();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('loadContext does nothing with a chapter but no comicDirectoryId', async () => {
		page.state = { chapter: readerChapter() };
		setupListeners();
		const hook = await renderHook();

		await hook.loadContext();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('loadContext does nothing with a comicDirectoryId but no chapter', async () => {
		page.state = { comicDirectoryId: 'dir-only' };
		setupListeners();
		const hook = await renderHook();

		await hook.loadContext();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('loadContext defaults sortBy to number_asc when the state has none', async () => {
		page.state = { chapter: readerChapter(), comicDirectoryId: 'dir-default-sort' };
		setupListeners();
		const hook = await renderHook();

		await hook.loadContext();

		expect(invokeMock).toHaveBeenCalledWith(
			LIBRARY_COMMANDS.getComicChapters,
			expect.objectContaining({ sortBy: 'number_asc' })
		);
	});

	it('swallows the error, resolves quietly, and resets pendingAction to null when loadContext fails', async () => {
		page.state = { chapter: readerChapter({ id: 'ch-1' }), comicDirectoryId: 'dir-6' };
		const { callbacks } = setupListeners();
		invokeMock.mockRejectedValueOnce(new Error('backend error'));
		const hook = await renderHook();

		await expect(hook.loadContext()).resolves.toBeUndefined();

		expect(invokeMock).toHaveBeenCalledWith(
			LIBRARY_COMMANDS.getComicChapters,
			expect.objectContaining({ comicDirectoryFk: 'dir-6' })
		);

		// `pendingAction` é privado — a única forma de observar se o `catch` realmente o
		// resetou pra `null` (em vez de deixá-lo travado em `'load'`) é ver como um evento de
		// `comicChapters` SUBSEQUENTE se comporta: se `pendingAction` ainda fosse `'load'`, esse
		// evento cairia (incorretamente) no branch de "load pendente" e atualizaria
		// chapterIndex/totalChapters a partir da lista recebida.
		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: {
				archive: { items: [chapterFile({ id: 'ch-0' }), chapterFile({ id: 'ch-1' })], total: 2 }
			}
		});
		await tick();

		expect(hook.chapterIndex).toBeUndefined();
		expect(hook.totalChapters).toBeUndefined();
	});

	it('comicChapters event resolves the chapter index for a pending load and persists it', async () => {
		page.state = { chapter: readerChapter({ id: 'ch-2' }), comicDirectoryId: 'dir-7' };
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const loadPromise = hook.loadContext();
		sessionStorageMock.setItem.mockClear();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: {
				archive: {
					items: [chapterFile({ id: 'ch-1' }), chapterFile({ id: 'ch-2' })],
					total: 2
				}
			}
		});
		await loadPromise;
		await tick();

		expect(hook.chapterIndex).toBe(1);
		expect(hook.totalChapters).toBe(2);

		const lastPersistedCall = sessionStorageMock.setItem.mock.calls.at(-1);
		expect(lastPersistedCall?.[0]).toBe(SESSION_KEYS.readerState);
		expect(JSON.parse(lastPersistedCall?.[1] ?? '{}')).toMatchObject({
			chapterIndex: 1,
			totalChapters: 2
		});
	});

	it('does not update chapterIndex/totalChapters or persist anything when the pending chapter id is not found in the loaded items', async () => {
		page.state = { chapter: readerChapter({ id: 'ch-missing' }), comicDirectoryId: 'dir-notfound' };
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const loadPromise = hook.loadContext();
		sessionStorageMock.setItem.mockClear();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: {
				archive: {
					items: [chapterFile({ id: 'ch-1' }), chapterFile({ id: 'ch-2' })],
					total: 2
				}
			}
		});
		await loadPromise;
		await tick();

		expect(hook.chapterIndex).toBeUndefined();
		expect(hook.totalChapters).toBeUndefined();
		expect(sessionStorageMock.setItem).not.toHaveBeenCalled();
	});

	it('does not crash when the loaded-chapters event fires while state.chapter is missing', async () => {
		page.state = { chapter: readerChapter({ id: 'ch-1' }), comicDirectoryId: 'dir-optchain' };
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const loadPromise = hook.loadContext();

		// Simula o chapter sumindo do estado ANTES do evento `comicChapters` chegar — sem o
		// optional chaining (`state.chapter?.id`), `state.chapter.id` explodiria aqui.
		(hook.state as ReaderNavigationState).chapter = undefined;

		expect(() =>
			callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
				payload: {
					archive: {
						items: [chapterFile({ id: 'ch-1' }), chapterFile({ id: 'ch-2' })],
						total: 2
					}
				}
			})
		).not.toThrow();

		await loadPromise;
		await tick();

		// Sem `chapter`, nenhum item bate o id (`undefined !== 'ch-1'/'ch-2'`) — idx fica -1 e
		// nada é persistido.
		expect(hook.chapterIndex).toBeUndefined();
	});

	it('comicChaptersError resets initializing and pending navigation state', async () => {
		page.state = {
			chapter: readerChapter({ id: 'ch-1' }),
			comicDirectoryId: 'dir-8',
			chapterIndex: 0,
			totalChapters: 2
		};
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const navPromise = hook.goToNextChapter();
		expect(hook.initializing).toBe(true);

		callbacks.get(LIBRARY_EVENTS.comicChaptersError)?.({ payload: { message: 'failed' } });
		await navPromise;
		await tick();

		expect(hook.initializing).toBe(false);
		expect(hook.chapter?.id).toBe('ch-1');
	});

	it('ignores a comicChapters event when there is no pending load or navigate action at all', async () => {
		// Estado ocioso: `pendingAction`/`pendingTargetIndex` continuam nos seus defaults
		// (`null`/`null`) porque `loadContext`/`goToNextChapter`/`goToPreviousChapter` nunca
		// foram chamados. Sem o `&&` correto em `pendingAction === 'navigate' && pendingTargetIndex
		// !== null`, um evento espúrio processaria a navegação mesmo sem nada pendente.
		page.state = {
			chapter: readerChapter({ id: 'ch-1' }),
			comicDirectoryId: 'dir-idle',
			chapterIndex: 0,
			totalChapters: 2
		};
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: { archive: { items: [chapterFile({ id: 'ch-2', name: 'Chapter 2' })] } }
		});
		await tick();

		expect(replaceStateMock).not.toHaveBeenCalled();
		expect(hook.chapter?.id).toBe('ch-1');
		expect(hook.chapterIndex).toBe(0);
	});

	it('does not act on a stale pendingTargetIndex left over after a load overwrote a pending navigate', async () => {
		// `loadContext()` só sobrescreve `pendingAction` pra `'load'` — nunca zera
		// `pendingTargetIndex`. Se um `navigate` for iniciado e, antes do evento chegar, um
		// `loadContext()` for disparado por cima, o primeiro evento de `comicChapters` cai no
		// branch de `'load'` (que zera só `pendingAction`), deixando `pendingTargetIndex` órfão
		// (não-nulo) com `pendingAction` já `null`. Um SEGUNDO evento, nesse estado, expõe
		// mutantes que trocam o `&&` por `||` ou forçam o operando esquerdo pra `true`.
		page.state = {
			chapter: readerChapter({ id: 'ch-1' }),
			comicDirectoryId: 'dir-stale-target',
			chapterIndex: 0,
			totalChapters: 2
		};
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const navPromise = hook.goToNextChapter();
		const loadPromise = hook.loadContext();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: { archive: { items: [chapterFile({ id: 'ch-1' })], total: 1 } }
		});
		await loadPromise;
		await navPromise;
		await tick();

		replaceStateMock.mockClear();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: { archive: { items: [chapterFile({ id: 'ch-2', name: 'Chapter 2' })] } }
		});
		await tick();

		expect(replaceStateMock).not.toHaveBeenCalled();
		expect(hook.chapter?.id).toBe('ch-1');
	});

	it('calls the registered unlisten callbacks on unmount', async () => {
		page.state = { chapter: readerChapter(), comicDirectoryId: 'dir-cleanup' };
		const { unlisteners } = setupListeners();

		const result = render(ReaderNavigationHarness, { props: { onReady: () => {} } });
		// `setupListeners()` faz DOIS `await listen(...)` sequenciais (chapters, depois error) —
		// cada um precisa da sua própria volta de microtask pra `unlistenChapters`/`unlistenError`
		// serem de fato atribuídos antes do unmount.
		await tick();
		await Promise.resolve();
		await Promise.resolve();
		await Promise.resolve();

		result.unmount();
		await tick();

		expect(unlisteners.get(LIBRARY_EVENTS.comicChapters)).toHaveBeenCalledTimes(1);
		expect(unlisteners.get(LIBRARY_EVENTS.comicChaptersError)).toHaveBeenCalledTimes(1);
	});

	it('goToNextChapter does nothing without a comicDirectoryId even when there is a next chapter', async () => {
		page.state = {
			chapter: readerChapter(),
			chapterIndex: 0,
			totalChapters: 3
			// sem comicDirectoryId
		};
		setupListeners();
		const hook = await renderHook();

		await hook.goToNextChapter();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('goToNextChapter does nothing when there is no next chapter', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-9',
			chapterIndex: 0,
			totalChapters: 1
		};
		setupListeners();
		const hook = await renderHook();

		await hook.goToNextChapter();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('goToPreviousChapter does nothing at the first chapter', async () => {
		page.state = {
			chapter: readerChapter(),
			comicDirectoryId: 'dir-10',
			chapterIndex: 0,
			totalChapters: 3
		};
		setupListeners();
		const hook = await renderHook();

		await hook.goToPreviousChapter();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('goToNextChapter requests the next chapter and updates state via the event listener', async () => {
		page.state = {
			chapter: readerChapter({ id: 'ch-1' }),
			comicDirectoryId: 'dir-11',
			chapterIndex: 0,
			totalChapters: 2,
			chapterScope: 'all'
		};
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const navPromise = hook.goToNextChapter();
		expect(hook.initializing).toBe(true);

		expect(invokeMock).toHaveBeenCalledWith(LIBRARY_COMMANDS.getComicChapters, {
			comicDirectoryFk: 'dir-11',
			volumeId: null,
			page: 1,
			pageSize: 1,
			sortBy: 'number_asc',
			searchQuery: null
		});

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: { archive: { items: [chapterFile({ id: 'ch-2', name: 'Chapter 2' })] } }
		});
		await navPromise;
		await tick();

		expect(hook.initializing).toBe(false);
		expect(hook.chapter?.id).toBe('ch-2');
		expect(hook.chapterIndex).toBe(1);
		expect(replaceStateMock).toHaveBeenCalled();
	});

	it('goToPreviousChapter does nothing without a comicDirectoryId even when there is a previous chapter', async () => {
		page.state = {
			chapter: readerChapter(),
			chapterIndex: 1,
			totalChapters: 3
			// sem comicDirectoryId
		};
		setupListeners();
		const hook = await renderHook();

		await hook.goToPreviousChapter();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('goToPreviousChapter requests the previous chapter and updates state via the event listener', async () => {
		page.state = {
			chapter: readerChapter({ id: 'ch-2' }),
			comicDirectoryId: 'dir-16',
			chapterIndex: 1,
			totalChapters: 2
		};
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const navPromise = hook.goToPreviousChapter();
		expect(hook.initializing).toBe(true);

		expect(invokeMock).toHaveBeenCalledWith(LIBRARY_COMMANDS.getComicChapters, {
			comicDirectoryFk: 'dir-16',
			volumeId: null,
			page: 0,
			pageSize: 1,
			sortBy: 'number_asc',
			searchQuery: null
		});

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: { archive: { items: [chapterFile({ id: 'ch-1', name: 'Chapter 1' })] } }
		});
		await navPromise;
		await tick();

		expect(hook.initializing).toBe(false);
		expect(hook.chapter?.id).toBe('ch-1');
		expect(hook.chapterIndex).toBe(0);
		expect(replaceStateMock).toHaveBeenCalled();
	});

	it('goToPreviousChapter stops and resets initializing when the backend returns no chapter', async () => {
		page.state = {
			chapter: readerChapter({ id: 'ch-2' }),
			comicDirectoryId: 'dir-12',
			chapterIndex: 1,
			totalChapters: 2
		};
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const navPromise = hook.goToPreviousChapter();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: { archive: { items: [] } }
		});
		await navPromise;
		await tick();

		expect(hook.initializing).toBe(false);
		expect(replaceStateMock).not.toHaveBeenCalled();
	});

	it('goToNextChapter applies fallback defaults for missing optional chapter fields', async () => {
		page.state = {
			chapter: readerChapter({ id: 'ch-1' }),
			comicDirectoryId: 'dir-14',
			chapterIndex: 0,
			totalChapters: 2
		};
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const navPromise = hook.goToNextChapter();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: {
				archive: {
					items: [
						{
							id: 'ch-2',
							name: 'Chapter 2',
							path: '/comics/ch-2.cbz'
							// chapterSort, volumeId, volumeName, isSpecial, lastModified: ausentes
						}
					]
					// sem `total` -> deve cair pro length de items
				}
			}
		});
		await navPromise;
		await tick();

		expect(hook.chapter).toMatchObject({
			id: 'ch-2',
			chapterSort: '',
			volumeId: null,
			volumeName: null,
			isSpecial: false,
			lastModified: 0
		});
	});

	it('comicChapters event falls back totalChapters to items.length when archive.total is absent', async () => {
		page.state = { chapter: readerChapter({ id: 'ch-2' }), comicDirectoryId: 'dir-15' };
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const loadPromise = hook.loadContext();

		callbacks.get(LIBRARY_EVENTS.comicChapters)?.({
			payload: {
				archive: {
					items: [
						chapterFile({ id: 'ch-1' }),
						chapterFile({ id: 'ch-2' }),
						chapterFile({ id: 'ch-3' })
					]
					// sem `total`
				}
			}
		});
		await loadPromise;
		await tick();

		expect(hook.totalChapters).toBe(3);
	});

	it('does not throw when unmounted before the event listeners finish registering', async () => {
		page.state = {};
		let resolveListen: (unlisten: () => void) => void = () => {};
		listenMock.mockImplementation(
			() =>
				new Promise((resolve) => {
					resolveListen = resolve as (unlisten: () => void) => void;
				})
		);

		const result = render(ReaderNavigationHarness, { props: { onReady: () => {} } });

		// Desmonta ANTES do `await listen(...)` resolver — `unlistenChapters`/`unlistenError`
		// ainda são `undefined` nesse ponto.
		expect(() => result.unmount()).not.toThrow();

		resolveListen(vi.fn());
		await tick();
	});

	it('resets initializing/pendingAction when navigation invoke fails', async () => {
		page.state = {
			chapter: readerChapter({ id: 'ch-1' }),
			comicDirectoryId: 'dir-13',
			chapterIndex: 0,
			totalChapters: 2
		};
		setupListeners();
		invokeMock.mockRejectedValueOnce(new Error('navigate failed'));
		const hook = await renderHook();

		await hook.goToNextChapter();

		expect(hook.initializing).toBe(false);
	});

	it('allows manually setting initializing', async () => {
		page.state = {};
		setupListeners();
		const hook = await renderHook();

		hook.initializing = true;
		expect(hook.initializing).toBe(true);
	});
});
