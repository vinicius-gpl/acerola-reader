import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import FakeDoc from '../__fixtures__/fake-doc.svelte';
import SlugPage from '../+page.svelte';

describe('docs/[...slug]/+page (component)', () => {
	it('renders the resolved doc component and the prev/next nav', () => {
		render(SlugPage, {
			props: {
				data: {
					Doc: FakeDoc,
					frontmatter: { title: 'Getting Started', section: 'Docs', description: 'Intro doc' },
					raw: '# Getting Started',
					prev: null,
					next: {
						locale: 'pt-br',
						slug: 'next-doc',
						component: FakeDoc,
						frontmatter: { title: 'Próximo Artigo', section: 'Docs' },
						raw: ''
					}
				}
			}
		});

		expect(screen.getByTestId('doc-body')).toHaveTextContent('Conteúdo do artigo');
		// A navegação de anterior/próximo aparece tanto acima quanto abaixo do corpo
		// do artigo (`position="top"` e o padrão `"bottom"`), então os links dela
		// legitimamente aparecem duas vezes.
		expect(screen.getAllByText('Próximo Artigo')).toHaveLength(2);
	});
});
