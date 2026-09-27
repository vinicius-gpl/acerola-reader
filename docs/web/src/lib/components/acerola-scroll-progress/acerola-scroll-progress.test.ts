import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AcerolaScrollProgress from './acerola-scroll-progress.svelte';

describe('AcerolaScrollProgress', () => {
	it('renders the scroll progress bar element with aria-hidden', () => {
		render(AcerolaScrollProgress);

		const bar = screen.getByTestId('scroll-progress');
		expect(bar).toBeInTheDocument();
		expect(bar).toHaveAttribute('aria-hidden', 'true');
		expect(bar).toHaveClass('scroll-progress-bar');
	});
});
