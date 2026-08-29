import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import DocsIndexPage from './+page.svelte';

describe('docs/+page', () => {
	it('renders the sidebar sections with links to each doc', () => {
		render(DocsIndexPage);

		// getSidebar('pt-br') roda contra o conteúdo markdown real do repo — só garantimos
		// que pelo menos uma seção com pelo menos um link de doc aparece, sem acoplar o
		// teste ao conjunto exato de artigos (que muda conforme a documentação evolui).
		const links = screen.getAllByRole('link');
		expect(links.length).toBeGreaterThan(0);
		expect(links[0]).toHaveAttribute('href', expect.stringContaining('/docs/'));
	});
});
