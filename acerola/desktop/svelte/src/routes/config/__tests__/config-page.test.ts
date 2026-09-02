import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
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
		mockInvoke.mockResolvedValue(undefined);
		mockListen.mockResolvedValue(vi.fn());
		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		});
	});

	it('renders every category row collapsed by default', () => {
		render(ConfigPage);

		expect(screen.getByText(/^configuração dos arquivos$/i)).toBeInTheDocument();
		expect(screen.getByText(/^biblioteca$/i)).toBeInTheDocument();
		expect(screen.getByText(/^aparência$/i)).toBeInTheDocument();
		expect(screen.getByText(/^configuração de metadados$/i)).toBeInTheDocument();
		expect(screen.getByText(/^marcadores$/i)).toBeInTheDocument();

		expect(screen.queryByText(/catppuccin/i)).not.toBeInTheDocument();
	});

	it('expands a category inline instead of navigating', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^aparência$/i));

		expect(await screen.findByText(/catppuccin/i)).toBeInTheDocument();
		expect(mockGoto).not.toHaveBeenCalled();

		// aria-expanded no cabeçalho muda de forma síncrona — a remoção do conteúdo em si passa
		// por transition:slide, cujo outro pode não completar em jsdom (sem layout real), então
		// o sinal confiável aqui é o atributo, não o conteúdo.
		await user.click(screen.getByText(/^aparência$/i));

		expect(screen.getByText(/^aparência$/i).closest('button')).toHaveAttribute(
			'aria-expanded',
			'false'
		);
	});

	it('keeps more than one category expanded at the same time', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^aparência$/i));
		await user.click(screen.getByText(/^biblioteca$/i));

		expect(await screen.findByText(/catppuccin/i)).toBeInTheDocument();
		expect(screen.getByText(/^templates de nomenclatura$/i)).toBeInTheDocument();
	});

	it('still navigates to the templates route from within library', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^biblioteca$/i));
		await user.click(await screen.findByText(/^templates de nomenclatura$/i));

		expect(mockGoto).toHaveBeenCalledWith('/config/templates');
	});
});
