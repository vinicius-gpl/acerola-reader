import { render, screen, fireEvent, within } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import AcerolaFilterPanel from './acerola-filter-panel.svelte';
import type { Category } from '$lib/contracts/bookmarks/bookmarks.payloads';

// Not anchored: selected sort options render extra content (A–Z/Z–A badge, check icon)
// appended to their accessible name, so an exact/anchored match would miss them.
const TITLE_SORT = /Title|Título/i;
const CHAPTER_COUNT_SORT = /Chapter Count|Quantidade de Capítulos/i;
const APPLY = /Apply|Aplicar/i;
const RESET = /Reset|Resetar/i;
const ALL = /All|Todos/i;
const NO_BOOKMARK = /No bookmark|Sem bookmark/i;
const SHOW_HIDDEN = /Show hidden|Mostrar ocultos/i;
const BOOKMARK_FILTER_GROUP = /Filter by bookmark|Filtrar por bookmark/i;

function bookmarkFilterGroup() {
	return within(screen.getByRole('group', { name: BOOKMARK_FILTER_GROUP }));
}

describe('AcerolaFilterPanel', () => {
	const mockBookmarks: Category[] = [
		{ id: 1, name: 'Favoritos', color: 0xff0000 },
		{ id: 2, name: 'Lendo', color: 0x00ff00 }
	];

	function baseData() {
		return {
			sortBy: 'title' as const,
			sortOrder: 'asc' as const,
			showHidden: false,
			metadataSource: 'all' as const,
			bookmarkFilter: 'all' as const,
			bookmarks: mockBookmarks
		};
	}

	const mockOnApply = vi.fn();
	const mockOnClose = vi.fn();

	function clickShowHiddenToggle() {
		const toggle = screen.getByText(SHOW_HIDDEN).closest('button');
		if (!toggle) throw new Error('could not find the show hidden toggle button');
		return fireEvent.click(toggle);
	}

	it('renders nothing when closed', () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: false },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
	});

	it('renders the panel with the current sort/filter state when open', () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		expect(screen.getByRole('dialog')).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: TITLE_SORT })).toHaveAttribute('aria-checked', 'true');
		expect(screen.getByRole('button', { name: 'Favoritos' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Lendo' })).toBeInTheDocument();
	});

	it('flips sort order when clicking the already-selected sort option', async () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		await fireEvent.click(screen.getByRole('radio', { name: TITLE_SORT }));
		await fireEvent.click(screen.getByRole('button', { name: APPLY }));

		expect(mockOnApply).toHaveBeenCalledWith(
			expect.objectContaining({ sortBy: 'title', sortOrder: 'desc' })
		);
	});

	it('selects a different sort option and applies it with ascending order', async () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		await fireEvent.click(screen.getByRole('radio', { name: CHAPTER_COUNT_SORT }));
		await fireEvent.click(screen.getByRole('button', { name: APPLY }));

		expect(mockOnApply).toHaveBeenCalledWith(
			expect.objectContaining({ sortBy: 'chapterCount', sortOrder: 'asc' })
		);
	});

	it('toggles the show hidden switch and applies it', async () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		await clickShowHiddenToggle();
		await fireEvent.click(screen.getByRole('button', { name: APPLY }));

		expect(mockOnApply).toHaveBeenCalledWith(expect.objectContaining({ showHidden: true }));
	});

	it('selects a bookmark category filter and applies it', async () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Favoritos' }));
		await fireEvent.click(screen.getByRole('button', { name: APPLY }));

		expect(mockOnApply).toHaveBeenCalledWith(expect.objectContaining({ bookmarkFilter: 1 }));
	});

	it('disables Reset when nothing changed and no filters are active', () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		expect(screen.getByRole('button', { name: RESET })).toBeDisabled();
	});

	it('enables Reset after a change and restores defaults when clicked', async () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		await clickShowHiddenToggle();
		const resetButton = screen.getByRole('button', { name: RESET });
		expect(resetButton).not.toBeDisabled();

		await fireEvent.click(resetButton);
		expect(resetButton).toBeDisabled();
	});

	it('calls onClose when the backdrop is clicked', async () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		await fireEvent.click(screen.getByRole('presentation'));

		expect(mockOnClose).toHaveBeenCalled();
	});

	it('calls onClose when Escape is pressed while open', async () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		await fireEvent.keyDown(window, { key: 'Escape' });

		expect(mockOnClose).toHaveBeenCalled();
	});

	it('renders only the default bookmark filter options when there are no bookmarks', () => {
		render(AcerolaFilterPanel, {
			props: {
				state: { open: true },
				data: { ...baseData(), bookmarks: [] },
				events: { onApply: mockOnApply, onClose: mockOnClose }
			}
		});

		const group = bookmarkFilterGroup();
		expect(group.getByRole('button', { name: ALL })).toBeInTheDocument();
		expect(group.getByRole('button', { name: NO_BOOKMARK })).toBeInTheDocument();
		expect(screen.queryByRole('button', { name: 'Favoritos' })).not.toBeInTheDocument();
	});
});
