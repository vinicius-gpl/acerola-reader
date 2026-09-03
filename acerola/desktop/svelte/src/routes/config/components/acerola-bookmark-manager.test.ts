import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BOOKMARKS_COMMANDS } from '$lib/contracts/bookmarks/bookmarks.commands';
import { _resetBookmarksState } from '$lib/hooks/store/use-bookmarks.svelte';
import AcerolaBookmarkManager from './acerola-bookmark-manager.svelte';

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args)
}));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn() }));

function setupInvokeMock(overrides: Record<string, unknown> = {}) {
	const defaults: Record<string, unknown> = {
		[BOOKMARKS_COMMANDS.getCategories]: [],
		[BOOKMARKS_COMMANDS.getAllComicCategories]: []
	};
	mockInvoke.mockImplementation((cmd: string) =>
		Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined))
	);
}

describe('AcerolaBookmarkManager', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		_resetBookmarksState();
		setupInvokeMock();
	});

	it('creates a new bookmark category', async () => {
		setupInvokeMock({
			[BOOKMARKS_COMMANDS.createCategory]: { id: 1, name: 'Favoritos', color: 0 }
		});
		const user = userEvent.setup();
		render(AcerolaBookmarkManager);

		const nameInput = await screen.findByLabelText(/nome|name/i);
		await user.type(nameInput, 'Favoritos');
		await user.click(screen.getByRole('button', { name: /criar|create/i }));

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(
				BOOKMARKS_COMMANDS.createCategory,
				expect.objectContaining({ name: 'Favoritos' })
			)
		);
	});

	it('deletes an existing bookmark category', async () => {
		setupInvokeMock({
			[BOOKMARKS_COMMANDS.getCategories]: [{ id: 1, name: 'Favoritos', color: 0 }]
		});
		const user = userEvent.setup();
		render(AcerolaBookmarkManager);

		await screen.findByText('Favoritos');
		const buttons = screen.getAllByRole('button');
		await user.click(buttons[buttons.length - 1]);

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(BOOKMARKS_COMMANDS.deleteCategory, { id: 1 })
		);
	});
});
