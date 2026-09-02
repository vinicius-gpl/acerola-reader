import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { READER_COMMANDS } from '$lib/contracts/reader/reader.commands';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';
import type {
	ReaderSessionPayload,
	ReaderPagePayload
} from '$lib/contracts/reader/reader.payloads';
import type { ReaderChapterPayload } from '$lib/contracts/reader/reader.payloads';
import ReaderPage from '../+page.svelte';

const { mockGoto, mockReplaceState } = vi.hoisted(() => ({
	mockGoto: vi.fn(),
	mockReplaceState: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: mockGoto,
	replaceState: mockReplaceState
}));

const { mockPageState } = vi.hoisted(() => ({
	mockPageState: {
		url: new URL('http://localhost/reader'),
		params: {},
		route: { id: null },
		status: 200,
		error: null,
		data: {},
		form: null,
		state: {} as Record<string, unknown>
	}
}));

vi.mock('$app/state', () => ({ page: mockPageState }));

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: () => Promise.resolve(vi.fn())
}));

const { mockStoreLoad } = vi.hoisted(() => ({ mockStoreLoad: vi.fn() }));

vi.mock('@tauri-apps/plugin-store', () => ({ load: mockStoreLoad }));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(), debug: vi.fn() }));

function chapter(overrides: Partial<ReaderChapterPayload> = {}): ReaderChapterPayload {
	return {
		id: 'ch-1',
		name: 'Capítulo 1',
		path: '/path/ch1.cbz',
		chapterSort: '1',
		volumeId: null,
		volumeName: null,
		isSpecial: false,
		lastModified: 0,
		...overrides
	};
}

function session(overrides: Partial<ReaderSessionPayload> = {}): ReaderSessionPayload {
	return {
		chapter: chapter(),
		pageCount: 3,
		currentPage: 0,
		cacheCapacity: 8,
		...overrides
	};
}

function pagePayload(overrides: Partial<ReaderPagePayload> = {}): ReaderPagePayload {
	return {
		chapterId: 'ch-1',
		index: 0,
		total: 3,
		mimeType: 'image/png',
		bytes: [137, 80, 78, 71],
		cacheHit: false,
		...overrides
	};
}

function setupInvokeMock(overrides: Record<string, unknown> = {}, rejects: string[] = []) {
	const defaults: Record<string, unknown> = {
		[READER_COMMANDS.openChapter]: session(),
		[READER_COMMANDS.loadPage]: pagePayload(),
		[READER_COMMANDS.setCurrentPage]: undefined,
		[READER_COMMANDS.prefetchWindow]: undefined,
		[READER_COMMANDS.closeChapter]: undefined,
		[LIBRARY_COMMANDS.getComicChapters]: undefined,
		history_update_reading: undefined
	};
	mockInvoke.mockImplementation((cmd: string) => {
		if (rejects.includes(cmd)) return Promise.reject(new Error('boom'));
		return Promise.resolve(cmd in overrides ? overrides[cmd] : defaults[cmd]);
	});
}

function renderReaderPage(state: Record<string, unknown> = {}) {
	mockPageState.state = {
		chapter: chapter(),
		comicDirectoryId: 'dir-1',
		startPage: 0,
		...state
	};

	return render(ReaderPage);
}

describe('reader +page', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		sessionStorage.clear();

		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			delete: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		});

		// +page.svelte cria seu próprio IntersectionObserver pra rastrear qual página está
		// visível — jsdom não implementa, então sem esse stub o mount inteiro quebra.
		globalThis.IntersectionObserver = class {
			constructor() {}
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof IntersectionObserver;

		// jsdom nesta versão expõe Element.prototype.animate mas devolve undefined em vez de
		// um Animation — quebra qualquer componente bits-ui que anime (ToggleGroupItem aqui).
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

	it('opens the chapter from navigation state and renders the chapter title', async () => {
		renderReaderPage();

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(
				READER_COMMANDS.openChapter,
				expect.objectContaining({ chapter: expect.objectContaining({ id: 'ch-1' }) })
			)
		);

		expect(await screen.findByText('Capítulo 1')).toBeInTheDocument();
	});

	it('shows the loading fallback before the chapter finishes opening', () => {
		renderReaderPage();

		expect(screen.getByText(/carregando|loading/i)).toBeInTheDocument();
	});

	it('goes back to /home when there is no previous history entry', async () => {
		const user = userEvent.setup();
		renderReaderPage();

		await screen.findByText('Capítulo 1');

		await user.click(screen.getByRole('button', { name: /voltar|back/i }));

		expect(mockGoto).toHaveBeenCalledWith('/home');
	});

	it('opens the command palette on ctrl+k', async () => {
		const user = userEvent.setup();
		renderReaderPage();

		await screen.findByText('Capítulo 1');

		await user.keyboard('{Control>}k{/Control}');

		expect(await screen.findByPlaceholderText(/comando|command/i)).toBeInTheDocument();
	});

	it('shows the open-failed state when opening the chapter fails', async () => {
		setupInvokeMock({}, [READER_COMMANDS.openChapter]);

		renderReaderPage();

		expect(await screen.findByText(/não foi possível abrir|unable to open/i)).toBeInTheDocument();
	});
});
