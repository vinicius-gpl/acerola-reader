import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import DocLayout from './doc-layout.svelte';

const { mockMermaidRun, mockMermaidInitialize } = vi.hoisted(() => ({
	// O mermaid.run() real substitui o conteúdo de cada nó por um `<svg>`
	// renderizado — o botão de zoom só aparece depois que ele existe, então o
	// mock também precisa simular essa parte.
	mockMermaidRun: vi.fn().mockImplementation(async ({ nodes }: { nodes: HTMLElement[] }) => {
		for (const node of nodes) node.innerHTML = '<svg><title>diagram</title></svg>';
	}),
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

	it('adds inline pan/zoom controls to a rendered diagram, starting at 100%', async () => {
		render(DocLayout, {
			props: {
				children: bodySnippet('<pre class="mermaid" data-testid="diagram">graph TD; A-->B;</pre>')
			}
		});

		expect(await screen.findByText('100%')).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Aumentar zoom' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Diminuir zoom' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Redefinir' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Mover para cima' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Mover para baixo' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Mover para a esquerda' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Mover para a direita' })).toBeInTheDocument();
	});

	it('zooms the diagram in and out in place, and resets back to 100%', async () => {
		render(DocLayout, {
			props: {
				children: bodySnippet('<pre class="mermaid" data-testid="diagram">graph TD; A-->B;</pre>')
			}
		});

		await screen.findByText('100%');
		const diagram = screen.getByTestId('diagram');
		const svg = diagram.querySelector('svg') as SVGElement;

		await fireEvent.click(screen.getByRole('button', { name: 'Aumentar zoom' }));
		expect(screen.getByText('125%')).toBeInTheDocument();
		expect(svg.style.transform).toBe('translate(0px, 0px) scale(1.25)');

		await fireEvent.click(screen.getByRole('button', { name: 'Diminuir zoom' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Diminuir zoom' }));
		expect(screen.getByText('75%')).toBeInTheDocument();
		expect(svg.style.transform).toBe('translate(0px, 0px) scale(0.75)');

		await fireEvent.click(screen.getByRole('button', { name: 'Redefinir' }));
		expect(screen.getByText('100%')).toBeInTheDocument();
		expect(svg.style.transform).toBe('translate(0px, 0px) scale(1)');
	});

	it('pans the diagram with the direction buttons, and resets the position too', async () => {
		render(DocLayout, {
			props: {
				children: bodySnippet('<pre class="mermaid" data-testid="diagram">graph TD; A-->B;</pre>')
			}
		});

		await screen.findByText('100%');
		const diagram = screen.getByTestId('diagram');
		const svg = diagram.querySelector('svg') as SVGElement;

		await fireEvent.click(screen.getByRole('button', { name: 'Mover para a direita' }));
		expect(svg.style.transform).toBe('translate(-60px, 0px) scale(1)');

		await fireEvent.click(screen.getByRole('button', { name: 'Mover para baixo' }));
		expect(svg.style.transform).toBe('translate(-60px, -60px) scale(1)');

		await fireEvent.click(screen.getByRole('button', { name: 'Redefinir' }));
		expect(svg.style.transform).toBe('translate(0px, 0px) scale(1)');
	});
});
