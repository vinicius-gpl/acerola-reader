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

describe('useReaderNavigation', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		invokeMock.mockResolvedValue(undefined);
		sessionStorage.clear();
		page.state = {};
	});

	afterEach(() => {
		sessionStorage.clear();
		page.state = {};
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

	it('swallows the error and resolves quietly when loadContext fails', async () => {
		page.state = { chapter: readerChapter(), comicDirectoryId: 'dir-6' };
		setupListeners();
		invokeMock.mockRejectedValueOnce(new Error('backend error'));
		const hook = await renderHook();

		await expect(hook.loadContext()).resolves.toBeUndefined();

		expect(invokeMock).toHaveBeenCalledWith(
			LIBRARY_COMMANDS.getComicChapters,
			expect.objectContaining({ comicDirectoryFk: 'dir-6' })
		);
	});

	it('comicChapters event resolves the chapter index for a pending load and persists it', async () => {
		page.state = { chapter: readerChapter({ id: 'ch-2' }), comicDirectoryId: 'dir-7' };
		const { callbacks } = setupListeners();
		const hook = await renderHook();

		const loadPromise = hook.loadContext();

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
