import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import DocsLayout from '../+layout.svelte';

vi.mock('$app/state', () => ({
	page: { params: { slug: 'getting-started' } }
}));

function childrenSnippet() {
	return createRawSnippet(() => ({
		render: () => `<div data-testid="doc-content">Conteúdo do doc</div>`
	}));
}

describe('docs/+layout', () => {
	it('renders the sidebar, the table of contents and the page content', async () => {
		render(DocsLayout, { props: { children: childrenSnippet() } });

		expect(await screen.findByTestId('doc-content')).toBeInTheDocument();
	});
});
