import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Callout from './callout.svelte';

function bodySnippet(text: string) {
	return createRawSnippet(() => ({
		render: () => `<p>${text}</p>`
	}));
}

describe('Callout (mdsvex)', () => {
	it('defaults to the note type and its type as title when no title is given', () => {
		render(Callout, { props: { children: bodySnippet('Some warning text') } });

		expect(screen.getByText('note')).toBeInTheDocument();
		expect(screen.getByText('Some warning text')).toBeInTheDocument();
	});

	it('renders a custom title and honors the type prop', () => {
		render(Callout, {
			props: { type: 'danger', title: 'Heads up', children: bodySnippet('Destructive action') }
		});

		expect(screen.getByText('Heads up')).toBeInTheDocument();
		expect(screen.getByText('Destructive action')).toBeInTheDocument();
	});
});
