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
					prev: null,
					next: {
						locale: 'pt-br',
						slug: 'next-doc',
						component: FakeDoc,
						frontmatter: { title: 'Próximo Artigo', section: 'Docs' }
					}
				}
			}
		});

		expect(screen.getByTestId('doc-body')).toHaveTextContent('Conteúdo do artigo');
		// The prev/next nav renders both above and below the doc body (`position="top"` and
		// the default `"bottom"`), so its links legitimately appear twice.
		expect(screen.getAllByText('Próximo Artigo')).toHaveLength(2);
	});
});
