import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { error as tauriError } from '@tauri-apps/plugin-log';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useBookmarks, _resetBookmarksState } from './use-bookmarks.svelte';
import { BOOKMARKS_COMMANDS } from '$lib/contracts/bookmarks/bookmarks.commands';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const errorMock = vi.mocked(tauriError);

async function renderBookmarksHook() {
	let hook: ReturnType<typeof useBookmarks> | undefined;

	render(HookHarness, {
		props: {
			create: () => useBookmarks(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useBookmarks>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

describe('useBookmarks', () => {
	// Capturado no beforeAll, que roda antes do primeiro beforeEach disparar.
	// É o único ponto de toda a suíte onde o state do hook reflete os valores
	// iniciais de verdade do `$state(...)` do módulo, em vez dos valores escritos
	// por `_resetBookmarksState()` (que todo outro teste observa depois do beforeEach).
	let pristineBookmarksSnapshot: unknown;
	let pristineAssignmentsSnapshot: unknown;
	let pristineIsLoadingSnapshot: boolean;
	let invokeCalledDuringPristineLoad: boolean;
	let fetchedAfterPristineLoad: unknown;

	beforeAll(async () => {
		const pristineHook = await renderBookmarksHook();

		pristineBookmarksSnapshot = [...pristineHook.bookmarks];
		pristineAssignmentsSnapshot = [...pristineHook.assignments];
		pristineIsLoadingSnapshot = pristineHook.isLoading;

		// isInitialized não tem getter, então observamos seu valor inicial de verdade
		// indiretamente: se começou `false`, essa primeiríssima chamada de loadBookmarks()
		// realmente busca os dados.
		invokeMock.mockResolvedValueOnce([{ id: 99, name: 'Pristine', color: 1 }]);
		invokeMock.mockResolvedValueOnce([]);

		await pristineHook.loadBookmarks();

		invokeCalledDuringPristineLoad = invokeMock.mock.calls.length > 0;
		fetchedAfterPristineLoad = [...pristineHook.bookmarks];

		invokeMock.mockClear();
	});

	it('has a genuinely empty bookmarks/assignments state and is not loading before anything ever ran', () => {
		expect(pristineBookmarksSnapshot).toEqual([]);
		expect(pristineBookmarksSnapshot).toHaveLength(0);
		expect(pristineAssignmentsSnapshot).toEqual([]);
		expect(pristineAssignmentsSnapshot).toHaveLength(0);
		expect(pristineIsLoadingSnapshot).toBe(false);
	});

	it('starts uninitialized, so the very first loadBookmarks call actually performs the fetch', () => {
		expect(invokeCalledDuringPristineLoad).toBe(true);
		expect(fetchedAfterPristineLoad).toEqual([{ id: 99, name: 'Pristine', color: 1 }]);
	});

	beforeEach(() => {
		vi.clearAllMocks();
		_resetBookmarksState();
	});

	it('loads bookmarks successfully', async () => {
		const mockBookmarks = [{ id: 1, name: 'Favoritos', color: 0xfff44336 }];
		const mockAssignments = [{ id: 1, comic_directory_fk: 123, category_id: 1 }];

		invokeMock.mockResolvedValueOnce(mockBookmarks);
		invokeMock.mockResolvedValueOnce(mockAssignments);

		const hook = await renderBookmarksHook();

		expect(hook.isLoading).toBe(false);
		await hook.loadBookmarks();

		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.getCategories);
		expect(hook.bookmarks).toEqual(mockBookmarks);
		expect(hook.isLoading).toBe(false);
	});

	it('is loading while the request is in flight, then settles back to false', async () => {
		let resolveGetCategories: (value: unknown) => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveGetCategories = resolve;
				})
		);
		invokeMock.mockResolvedValueOnce([]);

		const hook = await renderBookmarksHook();
		const pending = hook.loadBookmarks();
		await Promise.resolve();

		expect(hook.isLoading).toBe(true);

		resolveGetCategories([]);
		await pending;

		expect(hook.isLoading).toBe(false);
	});

	it('does not reload once already initialized', async () => {
		invokeMock.mockResolvedValueOnce([]);
		invokeMock.mockResolvedValueOnce([]);

		const hook = await renderBookmarksHook();
		await hook.loadBookmarks();
		invokeMock.mockClear();

		await hook.loadBookmarks();

		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('handles error when loading bookmarks', async () => {
		invokeMock.mockRejectedValueOnce('Network error');
		invokeMock.mockResolvedValueOnce([]);

		const hook = await renderBookmarksHook();
		await hook.loadBookmarks();

		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.getCategories);
		expect(errorMock).toHaveBeenCalledWith('Failed to load bookmarks: Network error');
		expect(hook.bookmarks).toEqual([]);
		expect(hook.isLoading).toBe(false);
	});

	it('creates bookmark and adds it to the list', async () => {
		const newBookmark = { id: 2, name: 'Lidos', color: 0xffe91e63 };
		invokeMock.mockResolvedValueOnce(newBookmark);

		const hook = await renderBookmarksHook();

		const result = await hook.createBookmark('Lidos', 0xffe91e63);

		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.createCategory, {
			name: 'Lidos',
			color: 0xffe91e63
		});
		expect(result).toEqual(newBookmark);
		expect(hook.bookmarks).toContainEqual(newBookmark);
	});

	it('deletes bookmark and removes only that one from the list', async () => {
		const mockBookmarks = [
			{ id: 1, name: 'Fav', color: 0xfff44336 },
			{ id: 2, name: 'Lidos', color: 0xffe91e63 }
		];
		invokeMock.mockResolvedValueOnce(mockBookmarks);
		invokeMock.mockResolvedValueOnce([]);

		const hook = await renderBookmarksHook();
		await hook.loadBookmarks(); // Preenche a lista
		expect(hook.bookmarks).toHaveLength(2);

		invokeMock.mockResolvedValueOnce(undefined); // delete_category

		await hook.deleteBookmark(1);

		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.deleteCategory, { id: 1 });
		expect(hook.bookmarks).toEqual([{ id: 2, name: 'Lidos', color: 0xffe91e63 }]);
	});

	it('assigns bookmark to comic, replacing any prior assignment for that same comic but keeping others', async () => {
		invokeMock.mockResolvedValueOnce([]);
		invokeMock.mockResolvedValueOnce([
			{ id: 1, comic_directory_fk: 123, category_id: 9 },
			{ id: 2, comic_directory_fk: 456, category_id: 9 }
		]);
		const hook = await renderBookmarksHook();
		await hook.loadBookmarks();

		const assignment = { id: 3, comic_directory_fk: 123, category_id: 1 };
		invokeMock.mockResolvedValueOnce(undefined); // remove_category_from_comic
		invokeMock.mockResolvedValueOnce(assignment); // assign_category_to_comic

		const result = await hook.assignToComic(123, 1);

		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.removeCategoryFromComic, {
			comicId: '123'
		});
		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.assignCategoryToComic, {
			comicId: '123',
			categoryId: 1
		});
		expect(result).toEqual(assignment);
		// A atribuição antiga da comic 123 some (substituída), mas a da 456 (outra comic)
		// continua intacta.
		expect(hook.assignments).toEqual([
			{ id: 2, comic_directory_fk: 456, category_id: 9 },
			assignment
		]);
	});

	it('removes bookmark from comic, keeping assignments for other comics', async () => {
		invokeMock.mockResolvedValueOnce([]);
		invokeMock.mockResolvedValueOnce([
			{ id: 1, comic_directory_fk: 123, category_id: 9 },
			{ id: 2, comic_directory_fk: 456, category_id: 9 }
		]);
		const hook = await renderBookmarksHook();
		await hook.loadBookmarks();

		invokeMock.mockResolvedValueOnce(undefined); // remove_category_from_comic

		await hook.removeComicBookmark(123);

		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.removeCategoryFromComic, {
			comicId: '123'
		});
		expect(hook.assignments).toEqual([{ id: 2, comic_directory_fk: 456, category_id: 9 }]);
	});

	it('resets assignments back to a genuinely empty array, not just a falsy/truthy one', async () => {
		invokeMock.mockResolvedValueOnce([]);
		invokeMock.mockResolvedValueOnce([{ id: 1, comic_directory_fk: 123, category_id: 9 }]);

		const hook = await renderBookmarksHook();
		await hook.loadBookmarks();
		expect(hook.assignments).toHaveLength(1);

		_resetBookmarksState();

		expect(hook.assignments).toEqual([]);
		expect(hook.assignments).toHaveLength(0);
		expect(hook.bookmarks).toEqual([]);
		expect(hook.isLoading).toBe(false);
	});

	it('gets comic bookmark', async () => {
		const mockBookmark = { id: 1, name: 'Fav', color: 0xfff44336 };
		invokeMock.mockResolvedValueOnce(mockBookmark); // get_comic_category

		const hook = await renderBookmarksHook();
		const result = await hook.getComicBookmark(123);

		expect(invokeMock).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.getComicCategory, {
			comicId: '123'
		});
		expect(result).toEqual(mockBookmark);
	});
});
