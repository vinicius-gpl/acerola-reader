import { render } from 'vitest-browser-svelte';
import { expect, it, describe, vi, beforeEach, afterEach } from 'vitest';
import ComicPage from '../+page.svelte';
import { LIBRARY_EVENTS } from '$lib/contracts/library/chapter.events';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Mock das APIs do Tauri funcionalmente
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(),
	convertFileSrc: vi.fn((path) => `asset://${path}`)
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn()
}));

vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn().mockResolvedValue({
		get: vi.fn().mockResolvedValue('cover'),
		set: vi.fn().mockResolvedValue(undefined),
		save: vi.fn().mockResolvedValue(undefined)
	})
}));

vi.mock('$lib/state/comic-context.svelte', () => ({
	useComicContext: vi.fn(() => ({
		item: null,
		coverUrl: null,
		set: vi.fn(),
		clear: vi.fn()
	}))
}));

vi.mock('$lib/hooks/store/use-bookmarks.svelte', () => ({
	useBookmarks: vi.fn(() => ({
		bookmarks: [],
		assignments: [],
		isLoading: false,
		loadBookmarks: vi.fn(),
		createBookmark: vi.fn(),
		deleteBookmark: vi.fn(),
		assignToComic: vi.fn(),
		removeComicBookmark: vi.fn(),
		getComicBookmark: vi.fn(),
		getBookmarkForComic: vi.fn()
	}))
}));

vi.mock('$lib/assets/placeholder/placeholder_manga.svg?component', () => ({
	default: vi.fn(() => null)
}));

class MockIntersectionObserver {
	readonly root: Element | Document | null;
	readonly rootMargin: string;
	readonly thresholds: ReadonlyArray<number>;
	readonly observedElements = new Map<number, HTMLElement>();

	constructor(
		readonly callback: IntersectionObserverCallback,
		options?: IntersectionObserverInit
	) {
		this.root = options?.root ?? null;
		this.rootMargin = options?.rootMargin ?? '';
		this.thresholds = Array.isArray(options?.threshold)
			? options.threshold
			: [options?.threshold ?? 0];
	}

	observe = vi.fn((target: Element) => {
		const pageIndex = Number((target as HTMLElement).dataset.page);

		this.observedElements.set(pageIndex, target as HTMLElement);
	});

	unobserve = vi.fn((target: Element) => {
		const pageIndex = Number((target as HTMLElement).dataset.page);

		this.observedElements.delete(pageIndex);
	});

	disconnect = vi.fn(() => {
		this.observedElements.clear();
	});

	takeRecords = vi.fn(() => []);

	emitVisiblePages(pageIndexes: number[]) {
		const entries = Array.from(this.observedElements.entries()).map(([pageIndex, target]) => {
			const isIntersecting = pageIndexes.includes(pageIndex);
			const rect = target.getBoundingClientRect();

			return {
				boundingClientRect: rect,
				intersectionRatio: isIntersecting ? 1 : 0,
				intersectionRect: rect,
				isIntersecting,
				rootBounds: null,
				target,
				time: performance.now()
			} as IntersectionObserverEntry;
		});

		this.callback(entries, this as unknown as IntersectionObserver);
	}
}

describe('ComicPage Scroll Integration', () => {
	const TOTAL_CHAPTERS = 1000;
	const RENDER_CHUNK_SIZE = 25;
	const ITEM_HEIGHT = 112;

	let rustEventEmitter:
		| ((event: { payload: ReturnType<typeof generateChaptersPayload> }) => void)
		| undefined;
	let intersectionObservers: MockIntersectionObserver[] = [];

	beforeEach(() => {
		vi.clearAllMocks();
		rustEventEmitter = undefined;
		intersectionObservers = [];

		vi.stubGlobal(
			'IntersectionObserver',
			class extends MockIntersectionObserver {
				constructor(callback: IntersectionObserverCallback, options?: IntersectionObserverInit) {
					super(callback, options);
					intersectionObservers.push(this);
				}
			} as unknown as typeof IntersectionObserver
		);

		(listen as any).mockImplementation(
			(
				eventName: string,
				callback: (event: { payload: ReturnType<typeof generateChaptersPayload> }) => void
			) => {
				if (eventName === LIBRARY_EVENTS.comicChapters) {
					rustEventEmitter = callback;
				}
				return Promise.resolve(() => {});
			}
		);

		// `+page.svelte` dispara `usePeerConnection()`/`useNetworkSync()` no onMount (fora do
		// escopo deste describe, mas ainda assim rodam) — get_paired_peers/get_sync_history_log
		// tem contrato de array, então precisam de um default diferente do `{}` genérico, senão
		// os `for...of`/`.map` desses hooks quebram tentando iterar um objeto vazio.
		(invoke as any).mockImplementation(async (command: string) => {
			if (
				command === NETWORK_COMMANDS.getPairedPeers ||
				command === NETWORK_COMMANDS.getSyncHistoryLog
			) {
				return [];
			}
			return {};
		});
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	// INFO: A busca não é mais paginada — o backend responde com todos os
	// TOTAL_CHAPTERS capítulos numa única resposta (ver FETCH_ALL_PAGE_SIZE em
	// use-comic-chapters.svelte.ts).
	const generateChaptersPayload = () => ({
		archive: {
			items: Array.from({ length: TOTAL_CHAPTERS }, (_, index) => ({
				id: `id-${index}`,
				name: `Chapter ${index + 1}`,
				path: `path-${index}`,
				chapterSort: `${index + 1}`,
				volumeId: null,
				volumeName: null,
				isSpecial: false,
				lastModified: 0
			})),
			volumes: [],
			pageSize: TOTAL_CHAPTERS,
			page: 0,
			total: TOTAL_CHAPTERS,
			volumeSections: []
		},
		showVolumeHeaders: false,
		hasVolumeStructure: false,
		effectiveViewMode: 'CHAPTER'
	});

	const loaderDataMock = {
		comic: {
			relations: { directoryId: '123', metadataId: null },
			filesystem: { folderName: 'test-manga' },
			metadata: {
				title: 'Production Test Manga',
				externalSync: false,
				activeSource: 'LOCAL',
				chapterCount: TOTAL_CHAPTERS
			},
			artwork: { cover: null, banner: null }
		},
		initialVolumeViewMode: 'cover' as 'cover' | 'banner'
	};

	function getLatestObserver() {
		const observer = intersectionObservers[intersectionObservers.length - 1];

		if (!observer) {
			throw new Error('IntersectionObserver was not created');
		}

		return observer;
	}

	function getChapterFetchCallCount() {
		return (invoke as any).mock.calls.filter(
			(callArgs: any[]) => callArgs[0] === LIBRARY_COMMANDS.getComicChapters
		).length;
	}

	function getChapterTitles(container: HTMLElement) {
		return Array.from(container.querySelectorAll('[data-slot="item-title"]')).map((element) =>
			element.textContent?.trim()
		);
	}

	async function waitForChapterFetch() {
		await vi.waitFor(() => {
			expect(getChapterFetchCallCount()).toBeGreaterThan(0);
		});
	}

	async function waitForTrackedPage(pageIndex: number) {
		await vi.waitFor(() => {
			expect(getLatestObserver().observedElements.has(pageIndex)).toBe(true);
		});
	}

	async function waitForChapterTitle(container: HTMLElement, title: string) {
		await vi.waitFor(() => {
			expect(getChapterTitles(container)).toContain(title);
		});
	}

	async function waitForChapterTitleGone(container: HTMLElement, title: string) {
		await vi.waitFor(() => {
			expect(getChapterTitles(container)).not.toContain(title);
		});
	}

	async function renderLoadedComicPage() {
		const view = await render(ComicPage, { data: loaderDataMock });

		await vi.waitFor(() => expect(rustEventEmitter).toBeDefined());
		await waitForChapterFetch();
		rustEventEmitter!({ payload: generateChaptersPayload() });
		await waitForTrackedPage(0);
		getLatestObserver().emitVisiblePages([0]);
		await waitForChapterTitle(view.container, 'Chapter 1');

		return view;
	}

	it('fetches all chapters in a single call regardless of scroll depth', async () => {
		const { container } = await renderLoadedComicPage();

		expect(getChapterFetchCallCount()).toBe(1);

		for (let pageIndex = 1; pageIndex <= 6; pageIndex++) {
			await waitForTrackedPage(pageIndex);
			getLatestObserver().emitVisiblePages([pageIndex]);
			await waitForChapterTitle(container, `Chapter ${pageIndex * RENDER_CHUNK_SIZE + 1}`);
		}

		// Nenhuma chamada extra de fetch — tudo já tinha chegado na primeira resposta.
		expect(getChapterFetchCallCount()).toBe(1);
	});

	it('unmounts the block leaving viewport and remounts when returning without refetching', async () => {
		const { container } = await renderLoadedComicPage();

		await waitForTrackedPage(1);
		getLatestObserver().emitVisiblePages([1]);
		await waitForChapterTitle(container, `Chapter ${RENDER_CHUNK_SIZE + 1}`);

		// Página 0 saiu da janela de renderização (só a 1 está visível agora).
		await waitForChapterTitleGone(container, 'Chapter 1');

		const firstPageBlock = container.querySelector('[data-page="0"]') as HTMLElement | null;
		expect(firstPageBlock).not.toBeNull();
		expect(firstPageBlock?.style.height).toBe(`${RENDER_CHUNK_SIZE * ITEM_HEIGHT}px`);
		expect(firstPageBlock?.querySelector('[data-slot="item-title"]')).toBeNull();

		// Volta pra página 0 — o dado já está em cache, então reaparece sem novo fetch.
		getLatestObserver().emitVisiblePages([0]);
		await waitForChapterTitle(container, 'Chapter 1');

		expect(getChapterFetchCallCount()).toBe(1);
	});

	it('renders chapter titles using clean name coming from Rust', async () => {
		const { container } = await renderLoadedComicPage();

		await vi.waitFor(
			() => {
				expect(getChapterTitles(container)).toContain('Chapter 1');
			},
			{ timeout: 3000 }
		);
	});
});
