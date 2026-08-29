import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CONTEXT_KEYS } from '$lib/constants/context-keys';
import { ActiveComicState } from '$lib/state/comic-context.svelte';
import { HOME_COMMANDS } from '$lib/contracts/home/home.commands';
import { HOME_EVENTS } from '$lib/contracts/home/home.events';
import type { ComicSummaryPayload, ComicSummaryItemPayload } from '$lib/contracts/home/home.payloads';
import { _resetComicSummaryState } from '$lib/hooks/store/use-comic-summary.svelte';
import { _resetComicSelectionState } from '$lib/hooks/store/use-comic-selection.svelte';
import { _resetBookmarksState } from '$lib/hooks/store/use-bookmarks.svelte';
import HomePage from '../+page.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// Assim como na tela do quadrinho: o back-end responde `getComicSummarySorted` via evento
// (`home:data`/`home:error`), não pelo retorno do invoke — capturamos os callbacks por nome
// de evento pra poder simular a chegada dos dados.
const listenCallbacks = new Map<string, (event: { payload: unknown }) => void>();

vi.mock('@tauri-apps/api/event', () => ({
	listen: (event: string, callback: (event: { payload: unknown }) => void) => {
		listenCallbacks.set(event, callback);
		return Promise.resolve(vi.fn());
	}
}));

const { mockStoreLoad } = vi.hoisted(() => ({ mockStoreLoad: vi.fn() }));

vi.mock('@tauri-apps/plugin-store', () => ({ load: mockStoreLoad }));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(), debug: vi.fn() }));

vi.mock('svelte-sonner', () => ({
	toast: { info: vi.fn(), success: vi.fn(), error: vi.fn() }
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}, rejects: string[] = []) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: [],
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
		metadata: { title: 'Acerola', externalSync: false, activeSource: 'LOCAL', chapterCount: 3 },
		artwork: { cover: null, banner: null },
		...overrides
	};
}

function summaryPayload(overrides: Partial<ComicSummaryPayload> = {}): ComicSummaryPayload {
	return {
		total: 1,
		fetchedAt: '2026-06-07T12:00:00.000Z',
		comics: [comic()],
		...overrides
	};
}

function renderHomePage() {
	const activeComic = new ActiveComicState();
	return render(HomePage, { context: new Map([[CONTEXT_KEYS.activeComic, activeComic]]) });
}

async function emitSummary(payload: ComicSummaryPayload) {
	await waitFor(() => expect(listenCallbacks.has(HOME_EVENTS.homeData)).toBe(true));
	listenCallbacks.get(HOME_EVENTS.homeData)?.({ payload });
	await tick();
}

describe('home +page', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		listenCallbacks.clear();
		_resetComicSummaryState();
		_resetComicSelectionState();
		_resetBookmarksState();

		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			delete: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined),
			reload: vi.fn().mockResolvedValue(undefined)
		});
	});

	it('shows the empty state when the library has no comics', async () => {
		renderHomePage();

		await emitSummary(summaryPayload({ total: 0, comics: [] }));

		expect(await screen.findByText(/nenhum quadrinho|no comics/i)).toBeInTheDocument();
	});

	it('renders the comic grid once the summary event arrives', async () => {
		renderHomePage();

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(
				HOME_COMMANDS.getComicSummarySorted,
				expect.objectContaining({ sortBy: 'title', sortOrder: 'asc' })
			)
		);

		await emitSummary(summaryPayload());

		expect(await screen.findByText('Acerola')).toBeInTheDocument();
	});

	it('opens the comic on card click and stores it in the active comic context', async () => {
		const user = userEvent.setup();
		renderHomePage();

		await emitSummary(summaryPayload());

		// O título fica FORA do <button> clicável (só a imagem/placeholder é clicável, ver
		// acerola-card-image.svelte) — sobe até o card e clica no botão dentro dele.
		const title = await screen.findByText('Acerola');
		const cardButton = title.closest('.group')?.querySelector('button');
		await user.click(cardButton as HTMLElement);

		expect(mockGoto).toHaveBeenCalledWith('/comic/Acerola');
	});

	it('shows an error toast when the summary fetch fails', async () => {
		const { toast } = await import('svelte-sonner');
		renderHomePage();

		await waitFor(() => expect(listenCallbacks.has(HOME_EVENTS.homeError)).toBe(true));
		listenCallbacks.get(HOME_EVENTS.homeError)?.({
			payload: { errorType: 'Unknown', message: 'falha ao carregar' }
		});

		await waitFor(() => expect(toast.error).toHaveBeenCalledWith('falha ao carregar'));
	});

	it('enters selection mode when clicking a card action button', async () => {
		const user = userEvent.setup();
		renderHomePage();
		await emitSummary(summaryPayload());
		await screen.findByText('Acerola');

		// O botão de ações do card é só o ícone MoreVertical (sem nome acessível) —
		// localiza pelo svg do lucide em vez de por texto/role.
		const actionButton = document
			.querySelector('svg.lucide-more-vertical, svg.lucide-ellipsis-vertical')
			?.closest('button');
		expect(actionButton).toBeTruthy();

		await user.click(actionButton as HTMLElement);

		expect(await screen.findByText(/1 selecionado|1 selected/i)).toBeInTheDocument();
	});
});
