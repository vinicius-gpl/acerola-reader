import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Card from './card.svelte';
import GithubIcon from '$lib/icons/github.svelte';

function bodySnippet(text: string) {
	return createRawSnippet(() => ({
		render: () => `<p>${text}</p>`
	}));
}

describe('Card (mdsvex)', () => {
	it('renders the title without a link when href is not provided', () => {
		render(Card, { props: { title: 'Getting Started', children: bodySnippet('Body text') } });

		expect(screen.getByText('Getting Started')).toBeInTheDocument();
		expect(screen.getByText('Body text')).toBeInTheDocument();
		expect(screen.queryByRole('link')).not.toBeInTheDocument();
	});

	it('wraps the card in a link when href is provided', () => {
		render(Card, {
			props: {
				title: 'Getting Started',
				href: '/docs/getting-started',
				children: bodySnippet('Body text')
			}
		});

		expect(screen.getByRole('link')).toHaveAttribute('href', '/docs/getting-started');
	});

	it('does not open internal links in a new tab', () => {
		render(Card, {
			props: { title: 'Getting Started', href: '/docs/getting-started' }
		});

		expect(screen.getByRole('link')).not.toHaveAttribute('target');
	});

	it('opens external links in a new tab without leaking a referrer', () => {
		render(Card, {
			props: { title: 'Storybook', href: 'https://storybook-web.acerola-comic.com' }
		});

		const link = screen.getByRole('link');
		expect(link).toHaveAttribute('target', '_blank');
		expect(link).toHaveAttribute('rel', 'noopener noreferrer');
	});

	it('renders no content block when no children are given', () => {
		const { container } = render(Card, { props: { title: 'Getting Started' } });

		expect(container.querySelector('[data-slot="card-content"]')).not.toBeInTheDocument();
	});

	it('renders the icon before the title when provided', () => {
		const { container } = render(Card, { props: { title: 'Getting Started', icon: GithubIcon } });

		const titleEl = screen.getByText('Getting Started').closest('[data-slot="card-title"]');
		expect(titleEl?.querySelector('svg')).toBeInTheDocument();
	});

	it('omits the icon element entirely when none is provided', () => {
		const { container } = render(Card, { props: { title: 'Getting Started' } });

		expect(container.querySelector('svg')).not.toBeInTheDocument();
	});

	it('merges a custom class onto the card root alongside the base classes', () => {
		const { container } = render(Card, {
			props: { title: 'Getting Started', class: 'my-custom-card' }
		});

		expect(container.querySelector('[data-slot="card"]')).toHaveClass('my-custom-card');
	});
});
