import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BOOKMARKS_COMMANDS } from '$lib/contracts/bookmarks/bookmarks.commands';
import { _resetBookmarksState } from '$lib/hooks/store/use-bookmarks.svelte';
import ConfigPage from '../+page.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

const { mockInvoke, mockListen } = vi.hoisted(() => ({
	mockInvoke: vi.fn().mockResolvedValue(undefined),
	mockListen: vi.fn().mockResolvedValue(vi.fn())
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: mockInvoke
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: mockListen
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

describe('config +page (accordion)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		_resetBookmarksState();
		// Marcadores agora fica sempre aberta (flat) — `AcerolaBookmarkManager` monta de cara
		// (não só depois de expandir uma categoria) e chama `loadBookmarks()` no `onMount`, então
		// precisa de uma resposta em formato de lista aqui, senão `bookmarks` vira `undefined`.
		mockInvoke.mockImplementation((cmd: string) => {
			if (
				cmd === BOOKMARKS_COMMANDS.getCategories ||
				cmd === BOOKMARKS_COMMANDS.getAllComicCategories
			) {
				return Promise.resolve([]);
			}
			return Promise.resolve(undefined);
		});
		mockListen.mockResolvedValue(vi.fn());
		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		});
	});

	// Aparência e Marcadores ficam sempre abertas (flat, sem clique) — só Arquivos/Biblioteca/
	// Metadados colapsam por padrão. Mesmo mix de UI/UX da tela de Rede (nem toda seção precisa
	// de um clique pra ver o conteúdo).
	it('renders the flat categories already open and the collapsible ones collapsed by default', () => {
		render(ConfigPage);

		expect(screen.getByText(/^configuração dos arquivos$/i)).toBeInTheDocument();
		expect(screen.getByText(/^biblioteca$/i)).toBeInTheDocument();
		expect(screen.getByText(/^aparência$/i)).toBeInTheDocument();
		expect(screen.getByText(/^configuração de metadados$/i)).toBeInTheDocument();
		expect(screen.getByText(/^marcadores$/i)).toBeInTheDocument();

		// Aparência é flat — o grid de temas já vem visível, sem precisar clicar.
		expect(screen.getByText(/catppuccin/i)).toBeInTheDocument();
		// Biblioteca continua colapsável — conteúdo escondido até clicar.
		expect(screen.queryByText(/^templates de nomenclatura$/i)).not.toBeInTheDocument();
	});

	it('expands a collapsible category inline instead of navigating', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^configuração de metadados$/i));

		expect(await screen.findByText(/sincronização com mangadex/i)).toBeInTheDocument();
		expect(mockGoto).not.toHaveBeenCalled();

		// aria-expanded no cabeçalho muda de forma síncrona — a remoção do conteúdo em si passa
		// por transition:slide, cujo outro pode não completar em jsdom (sem layout real), então
		// o sinal confiável aqui é o atributo, não o conteúdo.
		await user.click(screen.getByText(/^configuração de metadados$/i));

		expect(screen.getByText(/^configuração de metadados$/i).closest('button')).toHaveAttribute(
			'aria-expanded',
			'false'
		);
	});

	it('keeps a flat section visible alongside an expanded collapsible category', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^biblioteca$/i));

		// Aparência (flat) nunca deixou de estar visível; Biblioteca (colapsável) agora também está.
		expect(screen.getByText(/catppuccin/i)).toBeInTheDocument();
		expect(await screen.findByText(/^templates de nomenclatura$/i)).toBeInTheDocument();
	});

	it('still navigates to the templates route from within library', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^biblioteca$/i));
		await user.click(await screen.findByText(/^templates de nomenclatura$/i));

		expect(mockGoto).toHaveBeenCalledWith('/config/templates');
	});
});
