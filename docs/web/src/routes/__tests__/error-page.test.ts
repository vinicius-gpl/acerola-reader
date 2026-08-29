import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ErrorPage from '../+error.svelte';

describe('+error page', () => {
	it('renders the 404 message with a link back home', () => {
		render(ErrorPage);

		expect(screen.getByText('404')).toBeInTheDocument();
		const homeLink = screen.getByRole('link');
		expect(homeLink).toHaveAttribute('href', '/');
	});
});
