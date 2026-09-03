import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import PlatformCard from './platform-card.svelte';
import GithubIcon from '$lib/icons/github.svelte';

function ctaSnippet(text: string) {
	return createRawSnippet(() => ({
		render: () => `<a href="#">${text}</a>`
	}));
}

describe('PlatformCard (mdsvex)', () => {
	it('renders the title', () => {
		render(PlatformCard, { props: { title: 'Windows' } });

		expect(screen.getByText('Windows')).toBeInTheDocument();
	});

	it('renders the description when provided', () => {
		render(PlatformCard, {
			props: { title: 'Windows', description: 'Install from the Microsoft Store.' }
		});

		expect(screen.getByText('Install from the Microsoft Store.')).toBeInTheDocument();
	});

	it('omits the description when not provided', () => {
		const { container } = render(PlatformCard, { props: { title: 'Windows' } });

		expect(container.querySelector('[data-slot="card-description"]')).not.toBeInTheDocument();
	});

	it('renders the icon when provided', () => {
		const { container } = render(PlatformCard, { props: { title: 'Windows', icon: GithubIcon } });

		expect(container.querySelector('svg')).toBeInTheDocument();
	});

	it('renders the cta content passed as children', () => {
		render(PlatformCard, {
			props: { title: 'Android', children: ctaSnippet('Download APK') }
		});

		expect(screen.getByRole('link', { name: 'Download APK' })).toBeInTheDocument();
	});

	it('omits the icon wrapper and cta area entirely when only title is given', () => {
		const { container } = render(PlatformCard, { props: { title: 'Windows' } });

		expect(container.querySelector('svg')).not.toBeInTheDocument();
		expect(container.querySelector('[data-slot="card-content"]')).not.toBeInTheDocument();
	});
});
