import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaCopyButton from './acerola-copy-button.svelte';

describe('AcerolaCopyButton', () => {
	it('renders button with label and responds to click', async () => {
		const user = userEvent.setup();
		const writeTextMock = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue(undefined);

		render(AcerolaCopyButton, {
			props: {
				value: 'https://example.com',
				label: 'Copiar Link',
				copiedLabel: 'Link Copiado'
			}
		});

		const btn = screen.getByRole('button', { name: 'Copiar Link' });
		expect(btn).toBeInTheDocument();

		await user.click(btn);

		expect(writeTextMock).toHaveBeenCalledWith('https://example.com');
		expect(screen.getByText('Link Copiado')).toBeInTheDocument();
	});

	it('disables button when value is empty or disabled prop is true', () => {
		render(AcerolaCopyButton, {
			props: {
				value: '',
				label: 'Copiar'
			}
		});

		const btn = screen.getByRole('button', { name: 'Copiar' });
		expect(btn).toBeDisabled();
	});
});
