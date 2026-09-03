import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AcerolaMicrosoftStoreButton from './acerola-microsoft-store-button.svelte';

describe('AcerolaMicrosoftStoreButton', () => {
	it('links to the Microsoft Store listing, opened in a new tab without leaking a referrer', () => {
		render(AcerolaMicrosoftStoreButton);

		const link = screen.getByRole('link');
		expect(link).toHaveAttribute(
			'href',
			'https://apps.microsoft.com/store/detail/9N9SL3P7F8GC?cid=DevShareMCLPCS'
		);
		expect(link).toHaveAttribute('target', '_blank');
		expect(link).toHaveAttribute('rel', 'noopener noreferrer');
	});

	it('shows the default caption above the "Microsoft Store" brand name', () => {
		render(AcerolaMicrosoftStoreButton);

		expect(screen.getByText('Get it from')).toBeInTheDocument();
		expect(screen.getByText('Microsoft Store')).toBeInTheDocument();
	});

	it('uses a custom caption when provided', () => {
		render(AcerolaMicrosoftStoreButton, { props: { label: 'Baixe na' } });

		expect(screen.getByText('Baixe na')).toBeInTheDocument();
	});
});
