import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AcerolaSearchDialog from './acerola-search-dialog.svelte';

// ensurePagefind() short-circuits to status = 'unavailable' whenever import.meta.env.DEV is
// true, which it is by default under Vitest — real/loading/results branches are out of scope
// here (covered by e2e against a built site elsewhere).

describe('AcerolaSearchDialog', () => {
	it('shows the dev-unavailable empty state once opened', async () => {
		render(AcerolaSearchDialog, { props: { open: true } });

		expect(
			await screen.findByText('Busca disponível só no build de produção (`npm run build`).')
		).toBeInTheDocument();
		expect(screen.getByPlaceholderText('Digite para pesquisar...')).toBeInTheDocument();
	});

	it('does not render the dialog content while closed', () => {
		render(AcerolaSearchDialog, { props: { open: false } });

		expect(screen.queryByPlaceholderText('Digite para pesquisar...')).not.toBeInTheDocument();
	});

	it('toggles open on ctrl+k / cmd+k', async () => {
		render(AcerolaSearchDialog, { props: { open: false } });

		expect(screen.queryByPlaceholderText('Digite para pesquisar...')).not.toBeInTheDocument();

		await fireEvent.keyDown(window, { key: 'k', ctrlKey: true });

		expect(await screen.findByPlaceholderText('Digite para pesquisar...')).toBeInTheDocument();

		await fireEvent.keyDown(window, { key: 'k', ctrlKey: true });

		expect(screen.queryByPlaceholderText('Digite para pesquisar...')).not.toBeInTheDocument();
	});
});
