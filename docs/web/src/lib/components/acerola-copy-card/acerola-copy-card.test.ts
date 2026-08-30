import { fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AcerolaCopyCard from './acerola-copy-card.svelte';

let writeText: ReturnType<typeof vi.fn>;

beforeEach(() => {
	vi.useFakeTimers();
	// `@testing-library/user-event` lazily defines `navigator.clipboard` as a
	// getter-only property once it initializes — redefining it here (configurable,
	// before any `userEvent.setup()` call) is what lets us swap in a fresh spy per
	// test instead of fighting that getter. Uses `fireEvent`, not `user.click`, for
	// the same reason.
	writeText = vi.fn().mockResolvedValue(undefined);
	Object.defineProperty(navigator, 'clipboard', {
		value: { writeText },
		configurable: true,
		writable: true
	});
});

afterEach(() => {
	vi.useRealTimers();
});

describe('AcerolaCopyCard', () => {
	it('renders the label and value as plain text when no href is given', () => {
		render(AcerolaCopyCard, { props: { label: 'Contato', value: 'contato@acerola-comic.com' } });

		expect(screen.getByText('Contato')).toBeInTheDocument();
		expect(screen.getByText('contato@acerola-comic.com')).toBeInTheDocument();
		expect(screen.queryByRole('link')).not.toBeInTheDocument();
	});

	it('renders the value as a link when href is given', () => {
		render(AcerolaCopyCard, {
			props: {
				label: 'Contato',
				value: 'contato@acerola-comic.com',
				href: 'mailto:contato@acerola-comic.com'
			}
		});

		expect(screen.getByRole('link', { name: 'contato@acerola-comic.com' })).toHaveAttribute(
			'href',
			'mailto:contato@acerola-comic.com'
		);
	});

	it('copies the value to the clipboard and shows feedback that resets after a while', async () => {
		render(AcerolaCopyCard, { props: { label: 'Contato', value: 'contato@acerola-comic.com' } });

		await fireEvent.click(screen.getByRole('button', { name: 'Copy' }));

		expect(writeText).toHaveBeenCalledWith('contato@acerola-comic.com');
		expect(screen.getByRole('button', { name: 'Copied' })).toBeInTheDocument();

		await vi.advanceTimersByTimeAsync(2000);

		expect(screen.getByRole('button', { name: 'Copy' })).toBeInTheDocument();
	});

	it('uses custom copy/copied labels when provided', async () => {
		render(AcerolaCopyCard, {
			props: {
				label: 'Contato',
				value: 'contato@acerola-comic.com',
				copyLabel: 'Copiar',
				copiedLabel: 'Copiado'
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Copiar' }));

		expect(screen.getByRole('button', { name: 'Copiado' })).toBeInTheDocument();
	});
});
