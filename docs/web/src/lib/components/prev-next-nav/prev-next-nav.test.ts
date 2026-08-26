import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import type { DocEntry } from '$lib/content/docs';
import PrevNextNav from './prev-next-nav.svelte';

function makeDoc(slug: string, title: string): DocEntry {
	return {
		locale: 'pt-br',
		slug,
		component: {} as never,
		frontmatter: { title, section: 'Docs', order: 1 }
	};
}

describe('PrevNextNav', () => {
	it('renders both links when prev and next are given', () => {
		render(PrevNextNav, {
			props: {
				prev: makeDoc('getting-started', 'Primeiros passos'),
				next: makeDoc('architecture', 'Arquitetura')
			}
		});

		const prevLink = screen.getByRole('link', { name: /Anterior.*Primeiros passos/s });
		const nextLink = screen.getByRole('link', { name: /Próximo.*Arquitetura/s });

		expect(prevLink).toHaveAttribute('href', '/docs/getting-started');
		expect(nextLink).toHaveAttribute('href', '/docs/architecture');
	});

	it('renders only the next link when prev is null', () => {
		render(PrevNextNav, { props: { prev: null, next: makeDoc('architecture', 'Arquitetura') } });

		expect(screen.queryByText('Anterior')).not.toBeInTheDocument();
		expect(screen.getByRole('link', { name: /Arquitetura/ })).toHaveAttribute(
			'href',
			'/docs/architecture'
		);
	});

	it('renders nothing when both prev and next are null', () => {
		const { container } = render(PrevNextNav, { props: { prev: null, next: null } });

		expect(container.querySelector('a')).toBeNull();
	});
});
