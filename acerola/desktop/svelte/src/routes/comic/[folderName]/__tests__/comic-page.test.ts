import { render, screen, waitFor } from '@testing-library/svelte';
import { tick } from 'svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CONTEXT_KEYS } from '$lib/constants/context-keys';
import { ActiveComicState } from '$lib/state/comic-context.svelte';
import { LIBRARY_EVENTS } from '$lib/contracts/library/chapter.events';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';
import { METADATA_COMMANDS } from '$lib/contracts/metadata/metadata.commands';
import type { ChapterPayload } from '$lib/contracts/library/chapter.payloads';
import type { ComicSummaryItemPayload } from '$lib/contracts/home/home.payloads';
import { _resetBookmarksState } from '$lib/hooks/store/use-bookmarks.svelte';
import { _resetChapterSelectionState } from '$lib/hooks/store/use-chapter-selection.svelte';
import ComicPage from '../+page.svelte';

const { mockGoto, mockInvalidateAll } = vi.hoisted(() => ({
	mockGoto: vi.fn(),
	mockInvalidateAll: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: mockGoto,
	invalidateAll: mockInvalidateAll
}));

const { mockInvoke, mockListen } = vi.hoisted(() => ({
	mockInvoke: vi.fn(),
	mockListen: vi.fn()
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// A tela dispara listen() pra 4 fontes ao montar (comic:chapters, comic:chapters:error via
// useComicChapters, e mais uma dezena via peers.startListening()/p2pSync.startListening()) —
// capturamos os callbacks por nome de evento pra poder simular o evento de dados chegando
// depois do invoke() (o back-end responde via evento, o retorno do invoke em si é descartado).
const listenCallbacks = new Map<string, (event: { payload: unknown }) => void>();
let intersectionCallback: IntersectionObserverCallback | undefined;

vi.mock('@tauri-apps/api/event', () => ({
	listen: (event: string, callback: (event: { payload: unknown }) => void) => {
		listenCallbacks.set(event, callback);
		return Promise.resolve(vi.fn());
	}
}));

const { mockStoreLoad } = vi.hoisted(() => ({ mockStoreLoad: vi.fn() }));

vi.mock('@tauri-apps/plugin-store', () => ({
	load: mockStoreLoad
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn(),
	debug: vi.fn()
}));

vi.mock('svelte-sonner', () => ({
	toast: {
		info: vi.fn(),
		loading: vi.fn(() => 'mock-toast-id'),
		success: vi.fn(),
		error: vi.fn()
	}
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}, rejects: string[] = []) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: [],
		get_comic_category: null,
		history_get_comic: null,
		history_get_read_chapters: [],
		get_paired_peers: [],
		get_network_status: undefined,
		get_sync_history_log: []
	};
	mockInvoke.mockImplementation((cmd: string) => {
		if (rejects.includes(cmd)) return Promise.reject(new Error('offline'));
		return Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined));
	});
}

function comic(overrides: Partial<ComicSummaryItemPayload> = {}): ComicSummaryItemPayload {
	return {
		relations: { directoryId: 'dir-1', metadataId: null },
		filesystem: { folderName: 'Acerola' },
		metadata: {
			title: 'Acerola Vol. 1',
			externalSync: false,
			activeSource: 'LOCAL',
			chapterCount: 2,
			description: 'Uma sinopse',
			status: 'ongoing',
			author: 'Autor X',
			rating: null
		},
		artwork: { cover: null, banner: null },
		...overrides
	};
}

function chapterPayload(overrides: Partial<ChapterPayload> = {}): ChapterPayload {
	return {
		hasVolumeStructure: false,
		showVolumeHeaders: false,
		effectiveViewMode: 'CHAPTER' as never,
		archive: {
			items: [
				{
					id: 'ch-1',
					name: 'Capítulo 1',
					path: '/path/ch1.cbz',
					chapterSort: '1',
					volumeId: null,
					volumeName: null,
					isSpecial: false,
					lastModified: 0
				}
			],
			volumes: [],
			page: 0,
			pageSize: 1_000_000,
			total: 1,
			volumeSections: []
		},
		...overrides
	};
}

function renderComicPage(data: { comic: ComicSummaryItemPayload | null }) {
	const activeComic = new ActiveComicState();

	return render(ComicPage, {
		props: { data: { ...data, initialVolumeViewMode: undefined } },
		context: new Map([[CONTEXT_KEYS.activeComic, activeComic]])
	});
}

// O título aparece duas vezes (header mobile sticky + ComicMetadataPanel), então
// findByText falha por ambiguidade — findAllByText espera o render terminar igual.
async function waitForTitle(title: string) {
	await waitFor(() => expect(screen.getAllByText(title).length).toBeGreaterThan(0));
}

// O título do capítulo é reaproveitado como description no card (comic.name vira title E
// fileName na página) — getAllByText em vez de getByText evita a ambiguidade.
async function findChapterCard(text: string) {
	return (await screen.findAllByText(text))[0];
}

async function emitChapters(payload: ChapterPayload) {
	await waitFor(() => expect(listenCallbacks.has(LIBRARY_EVENTS.comicChapters)).toBe(true));
	listenCallbacks.get(LIBRARY_EVENTS.comicChapters)?.({ payload });
	await tick();

	const firstPage = document.querySelector<HTMLElement>('[data-page="0"]');
	if (firstPage) {
		intersectionCallback?.(
			[{ target: firstPage, isIntersecting: true } as unknown as IntersectionObserverEntry],
			{} as IntersectionObserver
		);
		await tick();
	}
}

describe('comic/[folderName] +page', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		listenCallbacks.clear();
		_resetBookmarksState();
		_resetChapterSelectionState();

		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			delete: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		});

		// acerola-comic-chapter-list.svelte só monta o conteúdo real (AcerolaHeroButton) quando o
		// IntersectionObserver reporta a página como visível — capturamos o callback pra
		// simular isso manualmente (ver revealFirstPage), igual acerola-comic-chapter-list.test.ts.
		intersectionCallback = undefined;
		globalThis.IntersectionObserver = class {
			constructor(callback: IntersectionObserverCallback) {
				intersectionCallback = callback;
			}
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof IntersectionObserver;

		// jsdom nesta versão expõe Element.prototype.animate mas devolve undefined em vez de
		// um Animation — o guard `if (!Element.prototype.animate)` do setup.ts global não pega
		// esse caso, e o bits-ui ToggleGroupItem quebra tentando setar `.onfinish` no retorno.
		Element.prototype.animate = vi.fn().mockImplementation(() => ({
			finished: Promise.resolve(),
			cancel: vi.fn(),
			finish: vi.fn(),
			pause: vi.fn(),
			play: vi.fn(),
			reverse: vi.fn(),
			onfinish: null,
			oncancel: null,
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		})) as unknown as typeof Element.prototype.animate;
	});

	it('shows the loading spinner while there is no comic data', () => {
		renderComicPage({ comic: null });

		expect(document.querySelector('.animate-spin')).toBeInTheDocument();
	});

	it('renders the comic header and chapter list once data arrives', async () => {
		renderComicPage({ comic: comic() });

		await waitForTitle('Acerola Vol. 1');
		expect(mockInvoke).toHaveBeenCalledWith(
			LIBRARY_COMMANDS.getComicChapters,
			expect.objectContaining({ comicDirectoryFk: 'dir-1' })
		);

		await emitChapters(chapterPayload());

		expect(await findChapterCard('Capítulo 1')).toBeInTheDocument();
	});

	it('opens the reader when a chapter is clicked', async () => {
		const user = userEvent.setup();
		renderComicPage({ comic: comic() });

		await waitForTitle('Acerola Vol. 1');
		await emitChapters(chapterPayload());

		await user.click(await findChapterCard('Capítulo 1'));

		expect(mockGoto).toHaveBeenCalledWith(
			'/reader',
			expect.objectContaining({
				state: expect.objectContaining({
					comicDirectoryId: 'dir-1',
					chapter: expect.objectContaining({ id: 'ch-1' })
				})
			})
		);
	});

	it('switches to the preferences tab', async () => {
		const user = userEvent.setup();
		renderComicPage({ comic: comic() });

		await waitForTitle('Acerola Vol. 1');
		await emitChapters(chapterPayload());

		await user.click(screen.getByRole('radio', { name: /preferences|preferências/i }));

		// ComicPreferences abre na lista de categorias (Leitura/Sincronização/Avançado) por
		// padrão — checa que o painel de preferências realmente montou.
		expect(await screen.findByText(/^leitura$/i)).toBeInTheDocument();
	});

	it('shows an error toast when syncing metadata from MangaDex fails', async () => {
		const { toast } = await import('svelte-sonner');
		setupInvokeMock({}, [METADATA_COMMANDS.syncMangadex]);

		const user = userEvent.setup();
		renderComicPage({ comic: comic({ metadata: { ...comic().metadata, externalSync: true } }) });

		await waitForTitle('Acerola Vol. 1');
		await emitChapters(chapterPayload());

		await user.click(screen.getByRole('radio', { name: /preferences|preferências/i }));
		await user.click(await screen.findByText(/^sincronização$/i));
		// título do card ("MangaDex Sync" / "Sincronização com MangaDex") — a description do
		// card também contém "mangadex", então precisa da frase completa pra não ambiguar.
		await user.click(await screen.findByText(/mangadex sync|sincronização com mangadex/i));

		await waitFor(() => expect(toast.error).toHaveBeenCalled());
	});
});
