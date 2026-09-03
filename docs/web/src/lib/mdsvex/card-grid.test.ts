import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import CardGrid from './card-grid.svelte';

describe('CardGrid (mdsvex)', () => {
	it('renders its children inside the grid container', () => {
		const children = createRawSnippet(() => ({
			render: () => `<div data-testid="grid-child">Card</div>`
		}));

		render(CardGrid, { props: { children } });

		expect(screen.getByTestId('grid-child')).toBeInTheDocument();
	});

	it('renders every card and animates them without throwing when actual [data-slot=card] children are present', () => {
		const children = createRawSnippet(() => ({
			render: () =>
				`<div>` +
				`<div data-slot="card" data-testid="card-1">One</div>` +
				`<div data-slot="card" data-testid="card-2">Two</div>` +
				`</div>`
		}));

		expect(() => render(CardGrid, { props: { children } })).not.toThrow();

		expect(screen.getByTestId('card-1')).toBeInTheDocument();
		expect(screen.getByTestId('card-2')).toBeInTheDocument();
	});

	it('does not throw when unmounted right after mounting (gsap context cleanup)', () => {
		const children = createRawSnippet(() => ({
			render: () => `<div data-slot="card">Card</div>`
		}));

		const { unmount } = render(CardGrid, { props: { children } });

		expect(() => unmount()).not.toThrow();
	});
});
