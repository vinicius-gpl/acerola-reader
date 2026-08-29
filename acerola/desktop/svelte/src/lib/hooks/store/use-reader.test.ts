import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { READER_COMMANDS } from '$lib/contracts/reader/reader.commands';
import type {
	ReaderChapterPayload,
	ReaderPagePayload,
	ReaderSessionPayload
} from '$lib/contracts/reader/reader.payloads';
import { useReader } from './use-reader.svelte';
import ReaderHarness from '../../../../tests/harness/hooks/reader-store.svelte';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

const invokeMock = vi.mocked(invoke);

function chapter(id = 'chapter-1'): ReaderChapterPayload {
	return {
		id,
		name: `Chapter ${id}`,
		path: `C:/mangas/${id}.cbz`,
		chapterSort: '1',
		volumeId: null,
		volumeName: null,
		isSpecial: false,
		lastModified: 0
	};
}

function session(chapterPayload = chapter(), pageCount = 5): ReaderSessionPayload {
	return {
		chapter: chapterPayload,
		pageCount,
		currentPage: 0,
		cacheCapacity: 3
	};
}

function page(chapterId: string, index: number): ReaderPagePayload {
	return {
		chapterId,
		index,
		total: 5,
		mimeType: index % 2 === 0 ? 'image/jpeg' : 'image/png',
		bytes: [index, index + 1],
		cacheHit: false
	};
}

async function renderHook() {
	let reader: ReturnType<typeof useReader> | undefined;

	const result = render(ReaderHarness, {
		props: {
			onReady: (hook) => {
				reader = hook;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return { reader: reader!, unmount: result.unmount };
}

async function waitMicrotasks() {
	await Promise.resolve();
	await tick();
	await Promise.resolve();
}

function calls(command: string) {
	return invokeMock.mock.calls.filter(([calledCommand]) => calledCommand === command);
}

describe('useReader', () => {
	beforeEach(() => {
		vi.clearAllMocks();

		Object.defineProperty(URL, 'createObjectURL', {
			value: vi.fn((blob: Blob) => `blob:${blob.type}:${blob.size}`),
			configurable: true
		});

		Object.defineProperty(URL, 'revokeObjectURL', {
			value: vi.fn(),
			configurable: true
		});

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});
	});

	it('starts with no open session and not loading', async () => {
		const { reader } = await renderHook();

		expect(reader.session).toBeUndefined();
		expect(reader.pageCount).toBe(0);
		expect(reader.currentPage).toBe(0);
		expect(reader.loading).toBe(false);
		expect(reader.error).toBeNull();
	});

	it('opens chapter, clamps initial page and loads current page', async () => {
		const { reader } = await renderHook();

		const chapterPayload = chapter();
		const readerSession = await reader.open(chapterPayload, 10);
		await waitMicrotasks();

		expect(readerSession.chapter).toEqual(chapterPayload);
		expect(reader.session?.chapter.id).toBe('chapter-1');
		expect(reader.currentPage).toBe(4);
		expect(reader.current?.index).toBe(4);
		expect(reader.current?.mimeType).toBe('image/jpeg');
		// Blob real: type e tamanho vêm de `payload.mimeType`/`payload.bytes`, não de um
		// literal vazio — o mock de createObjectURL expõe isso no formato `blob:<type>:<size>`.
		expect(reader.current?.url).toBe('blob:image/jpeg:2');
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.openChapter, {
			chapter: chapterPayload
		});
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.loadPage, {
			index: 4,
			setCurrent: true
		});
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.prefetchWindow, {
			center: 4,
			radius: 2
		});
	});

	it('goToPage before any chapter is open does nothing (no session to clamp against)', async () => {
		const { reader } = await renderHook();

		const page = await reader.goToPage(5);

		expect(page).toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('rejects a negative page index and an index at/after pageCount, accepts the last valid index', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		expect(await reader.loadPage(-1)).toBeUndefined();
		expect(await reader.loadPage(5)).toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();

		// pageCount é 5 (0..4) — o último índice válido é a fronteira, não pageCount.
		const lastPage = await reader.loadPage(4);
		expect(lastPage?.index).toBe(4);
	});

	it('nextPage/previousPage move relative to currentPage and clamp at the session bounds', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 2);
		await waitMicrotasks();

		await reader.nextPage();
		await waitMicrotasks();
		expect(reader.currentPage).toBe(3);

		await reader.previousPage();
		await waitMicrotasks();
		expect(reader.currentPage).toBe(2);

		await reader.previousPage();
		await reader.previousPage();
		await reader.previousPage();
		await waitMicrotasks();
		// Clampado em 0, não fica negativo.
		expect(reader.currentPage).toBe(0);
	});

	it('reuses cached page without requesting bytes from backend again', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const cachedPage = await reader.loadPage(0);

		expect(cachedPage?.index).toBe(0);
		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.setCurrentPage, { index: 0 });
	});

	it('ignores invalid page without calling backend', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const invalidPage = await reader.loadPage(99);

		expect(invalidPage).toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('records error when opening fails', async () => {
		const { reader } = await renderHook();
		const error = new Error('file not found');

		invokeMock.mockRejectedValueOnce(error);

		await expect(reader.open(chapter())).rejects.toThrow(error);

		expect(reader.error).toBe('file not found');
		expect(reader.loading).toBe(false);
	});

	it('records error when page loading fails', async () => {
		const { reader } = await renderHook();
		const error = new Error('decode failed');

		await reader.open(chapter(), 4);
		await waitMicrotasks();
		invokeMock.mockClear();
		invokeMock.mockRejectedValueOnce(error);

		await expect(reader.loadPage(1)).rejects.toThrow(error);

		expect(reader.error).toBe('decode failed');
		expect(reader.loading).toBe(false);
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.loadPage, {
			index: 1,
			setCurrent: true
		});
	});

	it('ignores stale page from previous session', async () => {
		const { reader } = await renderHook();
		let activeChapterId = 'chapter-1';
		let resolveStalePage: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				activeChapterId = payload.chapter.id;
				return Promise.resolve(session(payload.chapter));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number; setCurrent: boolean };

				if (activeChapterId === 'chapter-1' && payload.index === 1 && payload.setCurrent) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveStalePage = resolve;
					});
				}

				return Promise.resolve(page(activeChapterId, payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 4);
		await waitMicrotasks();

		const staleLoad = reader.loadPage(1);
		await Promise.resolve();

		await reader.open(chapter('chapter-2'), 4);
		await waitMicrotasks();

		resolveStalePage(page('chapter-1', 1));

		await expect(staleLoad).resolves.toBeUndefined();
		expect(reader.session?.chapter.id).toBe('chapter-2');
		expect(reader.currentPage).toBe(4);
		expect(reader.pageAt(1)).toBeUndefined();
	});

	it('prefetches a window around the center, skipping the center itself and already-cached pages', async () => {
		const { reader } = await renderHook();

		// pageCount 10, abre na página 5: janela esperada = [3,4,6,7] (raio 2, sem o 5 que já
		// virou "current" e já está em cache).
		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				return session(chapter(), 10);
			}
			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}
			return undefined;
		});

		await reader.open(chapter(), 5);
		await waitMicrotasks();
		await waitMicrotasks();

		const loadedIndices = calls(READER_COMMANDS.loadPage).map(
			([, args]) => (args as { index: number }).index
		);

		expect(loadedIndices).toContain(5); // current
		for (const neighbor of [3, 4, 6, 7]) {
			expect(loadedIndices).toContain(neighbor);
		}
		expect(loadedIndices).not.toContain(2);
		expect(loadedIndices).not.toContain(8);
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.prefetchWindow, {
			center: 5,
			radius: 2
		});
	});

	it('clamps the prefetch window near the start of the chapter (no negative indices)', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();

		const loadedIndices = calls(READER_COMMANDS.loadPage).map(
			([, args]) => (args as { index: number }).index
		);

		// pageCount 5, center 0, raio 2 -> janela real é [0,1,2], nunca índice negativo.
		expect(Math.min(...loadedIndices)).toBe(0);
		expect(loadedIndices).toContain(2);
	});

	it('records an error from setCurrentPage without throwing, still returning the cached page', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const failure = new Error('backend unreachable');
		invokeMock.mockImplementation(async (command) => {
			if (command === READER_COMMANDS.setCurrentPage) throw failure;
			return undefined;
		});

		const cachedPage = await reader.loadPage(0);

		expect(cachedPage?.index).toBe(0);
		expect(reader.error).toBe('backend unreachable');
	});

	it('closes chapter, clears cache and revokes object urls', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();

		expect(reader.cacheKeys.length).toBeGreaterThan(0);

		await reader.close();

		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.closeChapter);
		expect(reader.session).toBeUndefined();
		expect(reader.currentPage).toBe(0);
		expect(reader.cacheKeys).toEqual([]);
		expect(reader.loading).toBe(false);
		expect(URL.revokeObjectURL).toHaveBeenCalled();
	});

	it('pageAt returns the cached page for a loaded index', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();

		expect(reader.pageAt(0)?.index).toBe(0);
	});

	it('cleans up resources when component unmounts', async () => {
		const { reader, unmount } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		unmount();
		await waitMicrotasks();

		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.closeChapter);
		expect(URL.revokeObjectURL).toHaveBeenCalled();
	});

	it('clears in-flight pending requests on reset, so a later request for the same index is not served the abandoned pending promise', async () => {
		const { reader } = await renderHook();

		let oldRequestIssued = false;

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				// O índice 4 fica fora do raio de prefetch do startPage 0 (a janela é
				// [0,1,2]), então nunca é tocado pelo prefetch automático abaixo.
				if (payload.index === 4 && !oldRequestIssued) {
					oldRequestIssued = true;
					// Nunca resolvida pelo teste — simula uma requisição abandonada por um reset.
					return new Promise<ReaderPagePayload>(() => {});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		// Inicia uma requisição em andamento para o índice 4 que nunca vai se resolver.
		void reader.loadPage(4, false).catch(() => undefined);
		await Promise.resolve();

		// Reabrir reseta o mapa de requisições pendentes — uma requisição posterior
		// para o MESMO índice não pode ser servida pela promise abandonada e pendente para sempre.
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		void reader.loadPage(4, false);
		await waitMicrotasks();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(1);
	});

	it('sets loading synchronously the moment open() is called, before the backend responds', async () => {
		const { reader } = await renderHook();
		let resolveOpenChapter: (value: ReaderSessionPayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				return new Promise<ReaderSessionPayload>((resolve) => {
					resolveOpenChapter = resolve;
				});
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		const openPromise = reader.open(chapter('chapter-1'), 0);

		// Ainda sem await/tick — isso já precisa ser true de forma síncrona.
		expect(reader.loading).toBe(true);

		resolveOpenChapter(session(chapter()));
		await openPromise;
		await waitMicrotasks();

		expect(reader.loading).toBe(false);
	});

	it('resets in-flight request bookkeeping before awaiting the new chapter, so a stale response cannot slip through while opening', async () => {
		const { reader } = await renderHook();

		let resolveStalePage: (payload: ReaderPagePayload) => void = () => {};
		let resolveOpenChapter2: (value: ReaderSessionPayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };

				if (payload.chapter.id === 'chapter-1') {
					return Promise.resolve(session(payload.chapter));
				}

				return new Promise<ReaderSessionPayload>((resolve) => {
					resolveOpenChapter2 = resolve;
				});
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 2) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveStalePage = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		// Uma requisição não relacionada ao openChapter começa e fica pendente.
		const stalePromise = reader.loadPage(2, true);
		await Promise.resolve();

		// Começa a abrir chapter-2 — o openChapter em si ainda está pendente, então
		// tudo até (e incluindo) o reset já precisa ter rodado de forma síncrona
		// no momento em que voltamos para cá.
		const openPromise = reader.open(chapter('chapter-2'), 0);
		await Promise.resolve();

		// Enquanto ainda se aguarda o openChapter, a requisição pendente ANTIGA resolve.
		resolveStalePage(page('chapter-1', 2));
		await Promise.resolve();
		await Promise.resolve();

		// Ela já precisa ter sido invalidada pelo reset que roda ANTES de aguardar
		// o openChapter — não apenas pelo reset posterior que vem depois dele.
		await expect(stalePromise).resolves.toBeUndefined();
		expect(reader.currentPage).toBe(0);
		expect(reader.pageAt(2)).toBeUndefined();

		resolveOpenChapter2(session(chapter('chapter-2')));
		await openPromise;
		await waitMicrotasks();

		expect(reader.session?.chapter.id).toBe('chapter-2');
	});

	it('applies the session cache capacity from the backend, evicting beyond it (not the default pre-session size)', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter(), 5);
		await waitMicrotasks();
		await waitMicrotasks();

		// session.cacheCapacity é 3 (veja o helper `session()`); o prefetch carrega 5
		// páginas distintas (3,4,5,6,7) ao redor do centro 5 — o cache precisa ter
		// evictado até a capacidade vinda do backend, não o padrão pré-sessão de 20.
		expect(reader.cacheKeys.length).toBe(3);
	});

	it('does not sync current page or touch backend setCurrentPage when loading an already-cached page with setCurrent=false', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const before = reader.currentPage;
		const cachedPage = await reader.loadPage(0, false);

		expect(cachedPage?.index).toBe(0);
		expect(reader.currentPage).toBe(before);
		expect(calls(READER_COMMANDS.setCurrentPage)).toHaveLength(0);
	});

	it('reuses the same in-flight promise for concurrent loadPage calls on the same index', async () => {
		const { reader } = await renderHook();
		let resolvePage3: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolvePage3 = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const first = reader.loadPage(3, false);
		const second = reader.loadPage(3, false);
		await Promise.resolve();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(1);

		resolvePage3(page('chapter-1', 3));

		const [firstResult, secondResult] = await Promise.all([first, second]);

		expect(firstResult).toBe(secondResult);
		expect(firstResult?.index).toBe(3);
	});

	it('discards an in-flight response purely because the request version changed, even when the chapter id still matches', async () => {
		const { reader } = await renderHook();
		let hangingResolve: ((payload: ReaderPagePayload) => void) | undefined;

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3 && !hangingResolve) {
					return new Promise<ReaderPagePayload>((resolve) => {
						hangingResolve = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const staleLoad = reader.loadPage(3, false);
		await Promise.resolve();

		// Reabre o MESMO id de capítulo — incrementa a versão da requisição, mas o
		// chapterId (tanto o capturado no momento da emissão quanto o atual) permanece
		// idêntico ao da requisição pendente.
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		// Resolve a requisição emitida ANTES da reabertura com um payload cujo chapterId
		// bate perfeitamente — só o contador de versão da requisição a marca como obsoleta.
		hangingResolve?.(page('chapter-1', 3));

		await expect(staleLoad).resolves.toBeUndefined();
		expect(reader.pageAt(3)).toBeUndefined();
	});

	it('discards a response whose chapterId does not match the active chapter, even when the request version has not changed', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				// O backend responde com um payload marcado para um capítulo DIFERENTE do
				// que está realmente aberto — simula uma resposta do backend desviada/cruzada.
				const payload = args as { index: number };
				return page('some-other-chapter', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const result = await reader.loadPage(3, false);

		expect(result).toBeUndefined();
		expect(reader.pageAt(3)).toBeUndefined();
	});

	it('does not crash and discards the response when the session is closed while a page request is still in flight', async () => {
		const { reader } = await renderHook();
		let resolvePendingPage: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolvePendingPage = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const pending = reader.loadPage(3, false);
		await Promise.resolve();

		await reader.close();

		expect(() => resolvePendingPage(page('chapter-1', 3))).not.toThrow();

		await expect(pending).resolves.toBeUndefined();
		expect(reader.session).toBeUndefined();
		expect(reader.pageAt(3)).toBeUndefined();
	});

	it('does not record an error from a stale (superseded) rejected request', async () => {
		const { reader } = await renderHook();
		let rejectPendingPage: (error: Error) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3) {
					return new Promise<ReaderPagePayload>((_resolve, reject) => {
						rejectPendingPage = reject;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const staleLoad = reader.loadPage(3, false).catch(() => undefined);
		await Promise.resolve();

		// Requisição substituta/incremento de versão (a reabertura invalida a requisição pendente).
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		rejectPendingPage(new Error('stale backend failure'));
		await staleLoad;
		await waitMicrotasks();

		expect(reader.error).toBeNull();
	});

	it('does not change currentPage when loading a fresh (non-cached) page with setCurrent=false, but does when setCurrent=true', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();

		const before = reader.currentPage;
		const loaded = await reader.loadPage(6, false);

		expect(loaded?.index).toBe(6);
		expect(reader.currentPage).toBe(before);

		const loadedCurrent = await reader.loadPage(7, true);

		expect(loadedCurrent?.index).toBe(7);
		expect(reader.currentPage).toBe(7);
	});

	it("does not let a superseded request's cleanup evict a newer in-flight request for the same page index", async () => {
		const { reader } = await renderHook();

		let resolveOldIndex5: (payload: ReaderPagePayload) => void = () => {};
		let resolveNewIndex5: (payload: ReaderPagePayload) => void = () => {};
		let oldRequestIssued = false;

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 5) {
					if (!oldRequestIssued) {
						oldRequestIssued = true;
						return new Promise<ReaderPagePayload>((resolve) => {
							resolveOldIndex5 = resolve;
						});
					}

					return new Promise<ReaderPagePayload>((resolve) => {
						resolveNewIndex5 = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const oldRequest = reader.loadPage(5, false).catch(() => undefined);
		await Promise.resolve();

		// Reabre o mesmo capítulo: incrementa a versão da requisição e limpa o mapa de
		// pendentes, mas a chamada ANTIGA ao backend para o índice 5 continua rodando por baixo.
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const newRequest = reader.loadPage(5, false);
		await Promise.resolve();

		invokeMock.mockClear();

		// A requisição obsoleta (de antes da reabertura) finalmente se resolve agora.
		resolveOldIndex5(page('chapter-1', 5));
		await oldRequest;
		await waitMicrotasks();

		// Uma terceira chamada para o mesmo índice, enquanto a requisição NOVA ainda
		// está pendente, precisa reutilizá-la em vez de disparar uma chamada redundante
		// ao backend — provando que a limpeza da requisição obsoleta não evictou
		// indevidamente a entrada pendente mais nova.
		void reader.loadPage(5, false);
		await waitMicrotasks();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);

		resolveNewIndex5(page('chapter-1', 5));
		await newRequest;
	});

	it('only clears loading when the completing request is itself tracking the current page (setCurrent=true)', async () => {
		const { reader } = await renderHook();

		let resolveSide: (payload: ReaderPagePayload) => void = () => {};
		let resolveMain: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 4) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveSide = resolve;
					});
				}

				if (payload.index === 6) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveMain = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		// Requisição com setCurrent=false: nunca pode forçar `loading` true/false sozinha.
		const sideRequest = reader.loadPage(4, false);
		await Promise.resolve();
		expect(reader.loading).toBe(false);

		// Requisição com setCurrent=true: liga o loading.
		const mainRequest = reader.loadPage(6, true);
		await Promise.resolve();
		expect(reader.loading).toBe(true);

		// A conclusão da requisição com setCurrent=false NÃO pode limpar o loading
		// enquanto a requisição com setCurrent=true ainda está em andamento.
		resolveSide(page('chapter-1', 4));
		await sideRequest;
		await waitMicrotasks();
		expect(reader.loading).toBe(true);

		// Concluir a requisição com setCurrent=true limpa o loading.
		resolveMain(page('chapter-1', 6));
		await mainRequest;
		await waitMicrotasks();
		expect(reader.loading).toBe(false);
	});

	it('does not clear loading from a stale (superseded) setCurrent request while a newer open() is still in flight', async () => {
		const { reader } = await renderHook();

		let resolveStalePage: (payload: ReaderPagePayload) => void = () => {};
		let resolveOpenChapter2: (value: ReaderSessionPayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };

				if (payload.chapter.id === 'chapter-1') {
					return Promise.resolve(session(payload.chapter, 10));
				}

				return new Promise<ReaderSessionPayload>((resolve) => {
					resolveOpenChapter2 = resolve;
				});
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 4) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveStalePage = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const stale = reader.loadPage(4, true).catch(() => undefined);
		await Promise.resolve();

		// Começa a abrir um capítulo diferente — o próprio load com setCurrent dele fica
		// parado no meio do await assim que o openChapter resolve, mas por enquanto o
		// openChapter em si ainda não resolveu.
		const openPromise = reader.open(chapter('chapter-2'), 0).catch(() => undefined);
		await Promise.resolve();

		// Enquanto a abertura ainda está em andamento, a requisição obsoleta do índice 4
		// de ANTES resolve. Ela não pode limpar `loading` — o open() já o forçou para true,
		// e um load de verdade ainda está pendente.
		resolveStalePage(page('chapter-1', 4));
		await stale;
		await waitMicrotasks();

		expect(reader.loading).toBe(true);

		resolveOpenChapter2(session(chapter('chapter-2'), 10));
		await openPromise;
		await waitMicrotasks();

		expect(reader.loading).toBe(false);
	});

	it('prefetchWindow does nothing when no chapter is open', async () => {
		const { reader } = await renderHook();

		await expect(reader.prefetchWindow(0)).resolves.toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('records an error when the backend prefetchWindow call rejects', async () => {
		const { reader } = await renderHook();

		// pageCount 3 + radius 2 faz a janela inteira ser [0,1,2] — exatamente a
		// cacheCapacity de 3 da sessão, então nada é evictado após o prefetch.
		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 3);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter('chapter-1'), 1);
		await waitMicrotasks();
		await waitMicrotasks();

		// A essa altura, todas as páginas do capítulo já estão em cache via o prefetch
		// inicial, então uma segunda chamada a prefetchWindow não emite mais chamadas
		// a loadPage — isolando a rejeição do prefetchWindow no backend como a única
		// fonte de `error`.
		invokeMock.mockClear();
		const failure = new Error('prefetch failed');
		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.prefetchWindow) {
				throw failure;
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.prefetchWindow(1);
		await waitMicrotasks();

		expect(reader.error).toBe('prefetch failed');
		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);
	});

	it('prefetch never loads the center index itself, even if it is not yet cached', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				return session(chapter(), 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();
		invokeMock.mockClear();

		// Chama prefetchWindow diretamente com um centro que nunca foi carregado via
		// goToPage, garantindo que ele esteja ausente do cache.
		await reader.prefetchWindow(8);
		await waitMicrotasks();

		const loadedIndices = calls(READER_COMMANDS.loadPage).map(
			([, args]) => (args as { index: number }).index
		);

		// A janela é [6,7,8,9] (raio 2, limitado a pageCount-1=9) menos o centro.
		expect(loadedIndices).not.toContain(8);
		expect(loadedIndices).toEqual(expect.arrayContaining([6, 7, 9]));
	});

	it('prefetch loads the current-page index exactly once and skips a repeat prefetch of already-cached indices', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				// A janela [3,4,5,6,7] tem 5 páginas — aumenta o cacheCapacity acima disso
				// para que nada seja evictado, mantendo este teste focado na lógica de
				// pular repetições, não no comportamento de capacidade/eviction do cache.
				return { ...session(payload.chapter, 10), cacheCapacity: 10 };
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter('chapter-1'), 5);
		await waitMicrotasks();
		await waitMicrotasks();

		const centerLoads = calls(READER_COMMANDS.loadPage).filter(
			([, args]) => (args as { index: number }).index === 5
		);
		expect(centerLoads).toHaveLength(1);

		invokeMock.mockClear();
		await reader.prefetchWindow(5);
		await waitMicrotasks();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);
	});

	it("does not trigger prefetch when goToPage's own load is discarded (falsy result)", async () => {
		const { reader } = await renderHook();

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.loadPage) {
				// O backend responde para um capítulo diferente do que está aberto —
				// loadPage descarta isso e resolve para undefined.
				const payload = args as { index: number };
				return page('some-other-chapter', payload.index);
			}

			return undefined;
		});

		const result = await reader.goToPage(3);

		expect(result).toBeUndefined();
		expect(calls(READER_COMMANDS.prefetchWindow)).toHaveLength(0);
	});
});
