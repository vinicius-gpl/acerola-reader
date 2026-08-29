import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import RootLayout from '../+layout.svelte';

vi.mock('$app/state', () => ({
	page: {
		url: new URL('http://localhost/docs/getting-started'),
		params: {}
	}
}));

function childrenSnippet() {
	return createRawSnippet(() => ({
		render: () => `<div data-testid="page-content">Conteúdo</div>`
	}));
}

describe('root +layout (docs/web)', () => {
	beforeEach(() => {
		document.head.innerHTML = '';
	});

	it('renders the top nav and the page content', async () => {
		render(RootLayout, { props: { children: childrenSnippet() } });

		expect(await screen.findByTestId('page-content')).toBeInTheDocument();
	});

	it('opens the search dialog from the top nav button', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });

		const searchButtons = screen.getAllByRole('button', { name: /pesquisar|search/i });
		await user.click(searchButtons[0]);

		expect(
			await screen.findByPlaceholderText(/digite para pesquisar|type to search/i)
		).toBeInTheDocument();
	});

	it('injects the JSON-LD structured data script into the document head', async () => {
		render(RootLayout, { props: { children: childrenSnippet() } });

		const script = document.head.querySelector('script[type="application/ld+json"]');
		expect(script).toBeTruthy();
		const data = JSON.parse(script!.textContent ?? '{}');
		expect(data['@graph']).toBeInstanceOf(Array);
	});
});
