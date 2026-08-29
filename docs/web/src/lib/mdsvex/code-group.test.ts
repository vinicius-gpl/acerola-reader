import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import CodeGroup from './code-group.svelte';

function snippet(text: string) {
	return createRawSnippet(() => ({
		render: () => `<p>${text}</p>`
	}));
}

describe('CodeGroup (mdsvex)', () => {
	it('delegates to Tabs, rendering a trigger and content per item', () => {
		render(CodeGroup, {
			props: {
				items: [{ value: 'npm', label: 'npm', content: snippet('npm install acerola') }]
			}
		});

		expect(screen.getByRole('tab', { name: 'npm' })).toBeInTheDocument();
		expect(screen.getByText('npm install acerola')).toBeInTheDocument();
	});
});
