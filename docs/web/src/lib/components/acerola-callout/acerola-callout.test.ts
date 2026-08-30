import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import AcerolaCallout from './acerola-callout.svelte';

function bodySnippet(text: string) {
	return createRawSnippet(() => ({
		render: () => `<p>${text}</p>`
	}));
}

describe('AcerolaCallout', () => {
	it('defaults to the note type and its type as title when no title is given', () => {
		render(AcerolaCallout, { props: { children: bodySnippet('Some warning text') } });

		expect(screen.getByText('note')).toBeInTheDocument();
		expect(screen.getByText('Some warning text')).toBeInTheDocument();
	});

	it('renders a custom title and honors the type prop', () => {
		render(AcerolaCallout, {
			props: { type: 'danger', title: 'Heads up', children: bodySnippet('Destructive action') }
		});

		expect(screen.getByText('Heads up')).toBeInTheDocument();
		expect(screen.getByText('Destructive action')).toBeInTheDocument();
	});

	it('only marks the danger type as an assertive live region', () => {
		const { container: dangerContainer } = render(AcerolaCallout, {
			props: { type: 'danger', children: bodySnippet('Destructive action') }
		});
		expect(dangerContainer.querySelector('[role="alert"]')).toBeInTheDocument();

		const { container: noteContainer } = render(AcerolaCallout, {
			props: { children: bodySnippet('Some warning text') }
		});
		expect(noteContainer.querySelector('[role="alert"]')).not.toBeInTheDocument();
	});
});
