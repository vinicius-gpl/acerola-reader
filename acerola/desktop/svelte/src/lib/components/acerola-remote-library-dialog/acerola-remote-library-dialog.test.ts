import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import AcerolaRemoteLibraryDialog, {
	type AcerolaRemoteLibraryDialogProps
} from './acerola-remote-library-dialog.svelte';
import type { ComicSummary } from '$lib/contracts/network/network.payloads';

type DialogData = AcerolaRemoteLibraryDialogProps['data'];

function clickCard(title: string) {
	const heading = screen.getByText(title);
	const card = heading.closest('.group');
	const button = card?.querySelector('button');
	if (!button) throw new Error(`could not find card button for "${title}"`);
	return fireEvent.click(button);
}

describe('AcerolaRemoteLibraryDialog', () => {
	const mockComics: ComicSummary[] = [
		{ comicName: 'One Piece', chapterCount: 1090, coverVersion: 1 },
		{ comicName: 'Berserk', chapterCount: 364, coverVersion: 2 }
	];

	const mockOnOpenChange = vi.fn();
	const mockOnSelectComic = vi.fn();

	function baseData(overrides: Partial<DialogData> = {}): DialogData {
		return {
			peerLabel: 'Notebook do Trabalho',
			comics: mockComics,
			isLoading: false,
			coverPathFor: () => undefined,
			isSyncing: () => false,
			...overrides
		};
	}

	it('renders nothing when open is false', () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: false },
				data: baseData(),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		expect(screen.queryByText('One Piece')).not.toBeInTheDocument();
	});

	it('renders the peer label and the comics grid when open', () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		expect(screen.getByText(/Remote library|Biblioteca remota/i)).toBeInTheDocument();
		expect(screen.getByText('Notebook do Trabalho')).toBeInTheDocument();
		expect(screen.getByText('One Piece')).toBeInTheDocument();
		expect(screen.getByText('Berserk')).toBeInTheDocument();
	});

	it('shows a loading message and hides the grid while loading', () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: true },
				data: baseData({ isLoading: true }),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		expect(
			screen.getByText(/Querying device's library|Consultando biblioteca/i)
		).toBeInTheDocument();
		expect(screen.queryByText('One Piece')).not.toBeInTheDocument();
	});

	it('shows the error message when errorMessage is set', () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: true },
				data: baseData({ errorMessage: 'Falha ao conectar' }),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		expect(screen.getByText('Falha ao conectar')).toBeInTheDocument();
		expect(screen.queryByText('One Piece')).not.toBeInTheDocument();
	});

	it('shows an empty state when there are no comics', () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: true },
				data: baseData({ comics: [] }),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		expect(screen.getByText(/No comics found|Nenhum quadrinho encontrado/i)).toBeInTheDocument();
	});

	it('filters the comics grid by the search query', async () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		const search = screen.getByPlaceholderText(/Search comic|Buscar quadrinho/i);
		await fireEvent.input(search, { target: { value: 'berserk' } });

		expect(screen.queryByText('One Piece')).not.toBeInTheDocument();
		expect(screen.getByText('Berserk')).toBeInTheDocument();
	});

	it('calls onSelectComic when clicking a comic that is not syncing', async () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: true },
				data: baseData(),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		await clickCard('One Piece');

		expect(mockOnSelectComic).toHaveBeenCalledWith('One Piece');
	});

	it('does not call onSelectComic when the comic is currently syncing', async () => {
		render(AcerolaRemoteLibraryDialog, {
			props: {
				state: { open: true },
				data: baseData({ isSyncing: (name: string) => name === 'One Piece' }),
				events: { onOpenChange: mockOnOpenChange, onSelectComic: mockOnSelectComic }
			}
		});

		await clickCard('One Piece');

		expect(mockOnSelectComic).not.toHaveBeenCalled();
	});
});
