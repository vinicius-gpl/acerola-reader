import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AcerolaShinyText from './acerola-shiny-text.svelte';

describe('AcerolaShinyText', () => {
	it('renders the given text with the default animation speed', () => {
		render(AcerolaShinyText, { props: { text: 'Acerola' } });

		const span = screen.getByText('Acerola');
		expect(span).toBeInTheDocument();
		expect(span).toHaveClass('shiny-text');
		expect(span.style.getPropertyValue('--shiny-text-duration')).toBe('6s');
	});

	it('applies a custom speed and extra class names', () => {
		render(AcerolaShinyText, { props: { text: 'Fast', speed: 2, class: 'extra-class' } });

		const span = screen.getByText('Fast');
		expect(span).toHaveClass('shiny-text', 'extra-class');
		expect(span.style.getPropertyValue('--shiny-text-duration')).toBe('2s');
	});
});
