import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Card from './card.svelte';

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
});
