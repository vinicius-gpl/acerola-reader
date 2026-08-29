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
});
