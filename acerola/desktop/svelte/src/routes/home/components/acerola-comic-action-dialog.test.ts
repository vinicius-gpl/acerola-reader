import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaComicActionDialog from './acerola-comic-action-dialog.svelte';
import type { Category } from '$lib/contracts/bookmarks/bookmarks.payloads';

// Mock do toast
vi.mock('svelte-sonner', () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

// Mock do error do Tauri
vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

// Mock do element.animate para jsdom
if (!window.HTMLElement.prototype.animate) {
	window.HTMLElement.prototype.animate = function () {
		const self = {} as Animation;
		return {
			finished: Promise.resolve(self),
			cancel: () => {},
			finish: () => {},
			play: () => {},
			pause: () => {},
			reverse: () => {}
		} as Animation;
	};
}

describe('AcerolaComicActionDialog', () => {
	const mockBookmarks: Category[] = [
		{ id: 1, name: 'Favoritos', color: 0xff0000 },
		{ id: 2, name: 'Lendo', color: 0x00ff00 }
	];

	const mockOnHide = vi.fn();
	const mockOnDelete = vi.fn();
	const mockOnClearMetadata = vi.fn();
	const mockOnBookmark = vi.fn();
	const mockOnClose = vi.fn();
	const mockOnSelectAll = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders nothing when open is false', () => {
		render(AcerolaComicActionDialog, {
			props: {
				state: { open: false },
				data: {
					selectedIds: [],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		expect(screen.queryByText(/Comic Actions|Ações do Quadrinho/i)).not.toBeInTheDocument();
	});

	it('renders the dialog when open is true', () => {
		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2, 3],
					totalCount: 10,
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onSelectAll: mockOnSelectAll,
					onClose: mockOnClose
				}
			}
		});

		expect(screen.getByText(/Comic Actions|Ações do Quadrinho/i)).toBeInTheDocument();
		expect(
			screen.getByText(/3 (comic\(s\) selected|quadrinho\(s\) selecionado\(s\))/i)
		).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /Hide|Ocultar/i })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /Delete|Deletar/i })).toBeInTheDocument();
		expect(
			screen.getByRole('button', { name: /Bookmark|Adicionar Bookmark/i })
		).toBeInTheDocument();
		expect(
			screen.getByRole('button', { name: /Select all|Selecionar todos/i })
		).toBeInTheDocument();
	});

	it('calls onSelectAll when clicking select all button inside dialog', async () => {
		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1],
					totalCount: 10,
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onSelectAll: mockOnSelectAll,
					onClose: mockOnClose
				}
			}
		});

		const selectAllBtn = screen.getByRole('button', { name: /Select all|Selecionar todos/i });
		await fireEvent.click(selectAllBtn);

		expect(mockOnSelectAll).toHaveBeenCalled();
	});

	it('opens confirmation dialog and calls onHide when confirmed', async () => {
		mockOnHide.mockResolvedValueOnce(undefined);

		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		const hideButton = screen.getByRole('button', { name: /Hide|Ocultar/i });
		await fireEvent.click(hideButton);

		expect(screen.getByText(/Hide Comics|Ocultar Quadrinhos/i)).toBeInTheDocument();

		const confirmButtons = screen.getAllByRole('button', { name: /Hide|Ocultar/i });
		await fireEvent.click(confirmButtons[confirmButtons.length - 1]);

		expect(mockOnHide).toHaveBeenCalledWith([1, 2]);
	});

	it('cancels hide dialog when Cancel is clicked', async () => {
		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		const hideButton = screen.getByRole('button', { name: /Hide|Ocultar/i });
		await fireEvent.click(hideButton);

		const cancelButton = screen.getByRole('button', { name: /Cancel|Cancelar/i });
		await fireEvent.click(cancelButton);

		expect(mockOnHide).not.toHaveBeenCalled();
	});

	it('opens confirmation dialog when Delete is clicked', async () => {
		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		const deleteButton = screen.getByRole('button', { name: /Delete|Deletar/i });
		await fireEvent.click(deleteButton);

		expect(screen.getByText(/Delete Comics|Excluir Quadrinhos/i)).toBeInTheDocument();
	});

	it('calls onDelete when confirmed in dialog', async () => {
		mockOnDelete.mockResolvedValueOnce(undefined);

		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		const deleteButton = screen.getByRole('button', { name: /Delete|Deletar/i });
		await fireEvent.click(deleteButton);

		const confirmButtons = screen.getAllByRole('button', { name: /Delete|Excluir|Deletar/i });
		await fireEvent.click(confirmButtons[confirmButtons.length - 1]);

		expect(mockOnDelete).toHaveBeenCalledWith([1, 2]);
	});

	it('displays bookmarks menu when Bookmark button is clicked', async () => {
		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		const bookmarkButton = screen.getByRole('button', { name: /Bookmark|Adicionar Bookmark/i });
		await fireEvent.click(bookmarkButton);

		expect(screen.getByText('Favoritos')).toBeInTheDocument();
		expect(screen.getByText('Lendo')).toBeInTheDocument();
	});

	it('calls onBookmark with the correct categoryId', async () => {
		mockOnBookmark.mockResolvedValueOnce(undefined);

		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		const bookmarkButton = screen.getByRole('button', { name: /Bookmark|Adicionar Bookmark/i });
		await fireEvent.click(bookmarkButton);

		const favoritosOption = screen.getByText('Favoritos');
		await fireEvent.click(favoritosOption);

		expect(mockOnBookmark).toHaveBeenCalledWith([1, 2], 1);
	});

	it('cancels delete dialog when Cancel is clicked', async () => {
		render(AcerolaComicActionDialog, {
			props: {
				state: { open: true },
				data: {
					selectedIds: [1, 2],
					bookmarks: mockBookmarks
				},
				events: {
					onHide: mockOnHide,
					onDelete: mockOnDelete,
					onClearMetadata: mockOnClearMetadata,
					onBookmark: mockOnBookmark,
					onClose: mockOnClose
				}
			}
		});

		const deleteButton = screen.getByRole('button', { name: /Delete|Deletar/i });
		await fireEvent.click(deleteButton);

		const cancelButton = screen.getByRole('button', { name: /Cancel|Cancelar/i });
		await fireEvent.click(cancelButton);

		expect(mockOnDelete).not.toHaveBeenCalled();
	});
});
