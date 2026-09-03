import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaComicPreferences from './acerola-comic-preferences.svelte';

describe('AcerolaComicPreferences', () => {
	function defaultProps() {
		return {
			state: {
				volumeViewMode: 'cover' as const,
				bookmarkId: null,
				externalSyncEnabled: true
			},
			events: {
				onVolumeViewModeChange: vi.fn(),
				onBookmarkChange: vi.fn(),
				onExternalSyncChange: vi.fn()
			}
		};
	}

	async function openCategory(user: ReturnType<typeof userEvent.setup>, name: RegExp) {
		await user.click(screen.getByText(name));
	}

	function categoryButton(name: RegExp) {
		return screen.getByText(name).closest('button')!;
	}

	it('renders every category collapsed by default', () => {
		render(AcerolaComicPreferences, { props: defaultProps() });

		expect(screen.getByText(/^leitura$/i)).toBeInTheDocument();
		expect(screen.getByText(/^sincronização$/i)).toBeInTheDocument();
		expect(screen.getByText(/^avançado$/i)).toBeInTheDocument();
		expect(screen.queryByText(/atribua um marcador/i)).not.toBeInTheDocument();
	});

	it('expands a category inline when clicked', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);

		expect(screen.getByText(/atribua um marcador/i)).toBeInTheDocument();
	});

	it('collapses the category again on a second click', async () => {
		// aria-expanded no botão do cabeçalho muda de forma síncrona — a remoção do conteúdo em
		// si passa por transition:slide, cujo outro pode não completar em jsdom (sem layout
		// real), então o sinal confiável aqui é o atributo, não o conteúdo.
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);
		expect(categoryButton(/^leitura$/i)).toHaveAttribute('aria-expanded', 'true');

		await openCategory(user, /^leitura$/i);
		expect(categoryButton(/^leitura$/i)).toHaveAttribute('aria-expanded', 'false');
	});

	it('keeps more than one category expanded at the same time', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);
		await openCategory(user, /^sincronização$/i);

		expect(screen.getByText(/atribua um marcador/i)).toBeInTheDocument();
		expect(screen.getByText(/sincronização externa/i)).toBeInTheDocument();
	});

	it('hides volume preference when there is no volume structure', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);

		expect(screen.queryByText('Destaque do Volume')).not.toBeInTheDocument();
		expect(screen.queryByRole('radio', { name: 'Capa' })).not.toBeInTheDocument();
	});

	it('displays volume preference when volume structure exists', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, {
			props: {
				...defaultProps(),
				data: { hasVolumeStructure: true }
			}
		});

		await openCategory(user, /^leitura$/i);

		expect(screen.getByText('Destaque do Volume')).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: 'Capa' })).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: 'Banner' })).toBeInTheDocument();
	});

	it('changes volume highlight when clicking banner', async () => {
		const user = userEvent.setup();
		const props = defaultProps();
		render(AcerolaComicPreferences, {
			props: {
				...props,
				data: { hasVolumeStructure: true }
			}
		});

		await openCategory(user, /^leitura$/i);
		await user.click(screen.getByRole('radio', { name: 'Banner' }));

		expect(props.events.onVolumeViewModeChange).toHaveBeenCalledWith('banner');
	});
});
