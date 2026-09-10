import { render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LandingPage from './+page.svelte';

describe('landing +page', () => {
	beforeEach(() => {
		globalThis.ResizeObserver = class {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof ResizeObserver;
	});

	it('renders the hero title and the get started / github CTAs', () => {
		render(LandingPage);

		expect(screen.getByRole('link', { name: /get started|começar/i })).toHaveAttribute(
			'href',
			'/docs/getting-started'
		);
		expect(screen.getByRole('link', { name: /github/i })).toHaveAttribute(
			'href',
			expect.stringContaining('github.com')
		);
	});

	it('renders a platform card for each supported platform', () => {
		render(LandingPage);

		expect(screen.getAllByText(/android|desktop|relay|p2p/i).length).toBeGreaterThan(0);
	});
});
