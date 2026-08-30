import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AcerolaComicHeroBanner from './acerola-comic-hero-banner.svelte';

describe('AcerolaComicHeroBanner', () => {
	it('renders banner image when url is provided', () => {
		const { container } = render(AcerolaComicHeroBanner, {
			props: { data: { banner: 'https://test.com/banner.jpg' } }
		});
		const img = container.querySelector('img');
		expect(img).toBeInTheDocument();
		expect(img?.getAttribute('src')).toBe('https://test.com/banner.jpg');
	});

	it('renders icon placeholder when banner is not provided', () => {
		const { container } = render(AcerolaComicHeroBanner, { props: { data: { banner: null } } });
		const img = container.querySelector('img');
		expect(img).not.toBeInTheDocument();
	});

	it('displays the real rating when provided', () => {
		render(AcerolaComicHeroBanner, { props: { data: { banner: null, rating: 9.8 } } });
		expect(screen.getByText('9.8')).toBeInTheDocument();
	});

	it('hides rating badge when there is no rating, but shows total chapters', () => {
		render(AcerolaComicHeroBanner, {
			props: { data: { banner: null, rating: null, chapterCount: 10 } }
		});
		expect(screen.queryByText('9.8')).not.toBeInTheDocument();
		expect(screen.getByText('10 Caps')).toBeInTheDocument();
	});
});
