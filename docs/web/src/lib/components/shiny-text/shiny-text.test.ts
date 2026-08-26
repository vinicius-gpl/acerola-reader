import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ShinyText from './shiny-text.svelte';

describe('ShinyText', () => {
	it('renders the given text with the default animation speed', () => {
		render(ShinyText, { props: { text: 'Acerola' } });

		const span = screen.getByText('Acerola');
		expect(span).toBeInTheDocument();
		expect(span).toHaveClass('shiny-text');
		expect(span.style.getPropertyValue('--shiny-text-duration')).toBe('6s');
	});

	it('applies a custom speed and extra class names', () => {
		render(ShinyText, { props: { text: 'Fast', speed: 2, class: 'extra-class' } });

		const span = screen.getByText('Fast');
		expect(span).toHaveClass('shiny-text', 'extra-class');
		expect(span.style.getPropertyValue('--shiny-text-duration')).toBe('2s');
	});
});
