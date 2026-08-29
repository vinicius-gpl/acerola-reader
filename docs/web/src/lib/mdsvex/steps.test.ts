import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Steps from './steps.svelte';

describe('Steps (mdsvex)', () => {
	it('renders its children inside the steps container', () => {
		const children = createRawSnippet(() => ({
			render: () => `<ol><li data-testid="step-1">Step one</li></ol>`
		}));

		render(Steps, { props: { children } });

		expect(screen.getByTestId('step-1')).toBeInTheDocument();
	});
});
