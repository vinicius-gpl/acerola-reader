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
});
