import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { METADATA_COMMANDS } from '$lib/contracts/metadata/metadata.commands';
import { DIRECTORY_SCAN_COMMANDS } from '$lib/contracts/library/library.commands';
import { notificationStore } from '$lib/components/acerola-notification/acerola-notification.svelte';
import { _resetBookmarksState } from '$lib/hooks/store/use-bookmarks.svelte';
import ConfigPage from '../+page.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: () => Promise.resolve(vi.fn())
}));

const { mockStoreLoad } = vi.hoisted(() => ({ mockStoreLoad: vi.fn() }));

// use-theme.svelte.ts instancia LazyStore no top-level do módulo (fora de qualquer hook) —
// precisa continuar mockado aqui como no setup.ts global, já que o vi.mock deste arquivo
// substitui o módulo inteiro em vez de estender o mock global.
vi.mock('@tauri-apps/plugin-store', () => ({
	load: mockStoreLoad,
	LazyStore: class {
		constructor() {
			return {
				get: vi.fn().mockResolvedValue(null),
				set: vi.fn().mockResolvedValue(undefined)
			};
		}
	}
}));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(), debug: vi.fn() }));

vi.mock('svelte-sonner', () => ({
	toast: { info: vi.fn(), success: vi.fn(), error: vi.fn() }
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}, rejects: string[] = []) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: []
	};
	mockInvoke.mockImplementation((cmd: string) => {
		if (rejects.includes(cmd)) return Promise.reject(new Error('offline'));
		return Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined));
	});
}

function mockStore(getValues: Record<string, unknown> = {}) {
	mockStoreLoad.mockResolvedValue({
		get: vi.fn((key: string) => Promise.resolve(getValues[key] ?? undefined)),
		set: vi.fn().mockResolvedValue(undefined),
		delete: vi.fn().mockResolvedValue(undefined),
		save: vi.fn().mockResolvedValue(undefined),
		reload: vi.fn().mockResolvedValue(undefined)
	});
}

describe('config +page', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		mockStore();
		notificationStore.clearAll();
		_resetBookmarksState();

		Element.prototype.animate = vi.fn().mockImplementation(() => ({
			finished: Promise.resolve(),
			cancel: vi.fn(),
			finish: vi.fn(),
			pause: vi.fn(),
			play: vi.fn(),
			reverse: vi.fn(),
			onfinish: null,
			oncancel: null
		})) as unknown as typeof Element.prototype.animate;
	});

	it('shows the saved library path once loaded', async () => {
		mockStore({ library_path: '/home/user/comics' });

		render(ConfigPage);

		expect(await screen.findByText(/\/home\/user\/comics/)).toBeInTheDocument();
	});

	it('navigates to the templates page', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(await screen.findByText(/nomenclatura|templates/i));

		expect(mockGoto).toHaveBeenCalledWith('/config/templates');
	});

	it('triggers a quick library sync when a folder is already configured', async () => {
		mockStore({ library_path: '/home/user/comics' });
		const user = userEvent.setup();
		render(ConfigPage);

		await screen.findByText(/\/home\/user\/comics/);
		await user.click(await screen.findByText(/sincronização rápida|quick sync|fast sync/i));

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(
				DIRECTORY_SCAN_COMMANDS.refreshLibrary,
				expect.anything()
			)
		);
	});

	it('notifies success after syncing metadata with MangaDex', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(await screen.findByText(/sincronização com mangadex|mangadex sync/i));

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(
				METADATA_COMMANDS.syncAllMangadex,
				expect.objectContaining({ language: 'pt-br' })
			)
		);
		expect(notificationStore.notifications.length).toBeGreaterThan(0);
	});

	it('notifies an error when syncing metadata with AniList fails', async () => {
		setupInvokeMock({}, [METADATA_COMMANDS.syncAllAnilist]);
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(await screen.findByText(/sincronização com anilist|anilist sync/i));

		await waitFor(() =>
			expect(
				notificationStore.notifications.some((n) => /offline/i.test(n.message))
			).toBe(true)
		);
	});

	it('creates a new bookmark category', async () => {
		mockInvoke.mockImplementation((cmd: string) => {
			if (cmd === 'get_categories') return Promise.resolve([]);
			if (cmd === 'get_all_comic_categories') return Promise.resolve([]);
			if (cmd === 'create_category') return Promise.resolve({ id: 1, name: 'Favoritos', color: 0 });
			return Promise.resolve(undefined);
		});

		const user = userEvent.setup();
		render(ConfigPage);

		const nameInput = await screen.findByLabelText(/nome|name/i);
		await user.type(nameInput, 'Favoritos');
		await user.click(screen.getByRole('button', { name: /criar|create/i }));

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(
				'create_category',
				expect.objectContaining({ name: 'Favoritos' })
			)
		);
	});
});
