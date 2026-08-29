import { render, screen, waitFor } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import DocLayout from './doc-layout.svelte';

const { mockMermaidRun, mockMermaidInitialize } = vi.hoisted(() => ({
	mockMermaidRun: vi.fn().mockResolvedValue(undefined),
	mockMermaidInitialize: vi.fn()
}));

vi.mock('mermaid', () => ({
	default: { initialize: mockMermaidInitialize, run: mockMermaidRun }
}));

function bodySnippet(html: string) {
	return createRawSnippet(() => ({ render: () => html }));
}

describe('DocLayout (mdsvex)', () => {
	it('renders the title, description and children', () => {
		render(DocLayout, {
			props: {
				title: 'Getting Started',
				description: 'How to install Acerola',
				children: bodySnippet('<p data-testid="doc-body">Body</p>')
			}
		});

		expect(screen.getByText('Getting Started')).toBeInTheDocument();
		expect(screen.getByText('How to install Acerola')).toBeInTheDocument();
		expect(screen.getByTestId('doc-body')).toBeInTheDocument();
	});

	it('renders mermaid diagrams found inside the content', async () => {
		render(DocLayout, {
			props: {
				children: bodySnippet('<pre class="mermaid" data-testid="diagram">graph TD; A-->B;</pre>')
			}
		});

		await waitFor(() => expect(mockMermaidRun).toHaveBeenCalled());
		expect(mockMermaidRun).toHaveBeenCalledWith(
			expect.objectContaining({ nodes: expect.arrayContaining([expect.anything()]) })
		);
	});

	it('renders without a title or description when they are omitted', () => {
		render(DocLayout, { props: { children: bodySnippet('<p>Just content</p>') } });

		expect(screen.getByText('Just content')).toBeInTheDocument();
	});
});
