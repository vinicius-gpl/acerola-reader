import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import type { DocEntry } from '$lib/content/docs';
import AcerolaPrevNextNav from './acerola-prev-next-nav.svelte';

function makeDoc(slug: string, title: string): DocEntry {
	return {
		locale: 'pt-br',
		slug,
		component: {} as never,
		raw: '',
		frontmatter: { title, section: 'Docs', order: 1 }
	};
}

describe('AcerolaPrevNextNav', () => {
	it('renders both links when prev and next are given', () => {
		render(AcerolaPrevNextNav, {
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
		render(AcerolaPrevNextNav, {
			props: { prev: null, next: makeDoc('architecture', 'Arquitetura') }
		});

		expect(screen.queryByText('Anterior')).not.toBeInTheDocument();
		expect(screen.getByRole('link', { name: /Arquitetura/ })).toHaveAttribute(
			'href',
			'/docs/architecture'
		);
	});

	it('renders nothing when both prev and next are null', () => {
		const { container } = render(AcerolaPrevNextNav, { props: { prev: null, next: null } });

		expect(container.querySelector('a')).toBeNull();
	});
});
