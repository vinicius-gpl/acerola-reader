import { fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import CopyMarkdownButton from './copy-markdown-button.svelte';

let writeText: ReturnType<typeof vi.fn>;

beforeEach(() => {
	vi.useFakeTimers();
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

describe('CopyMarkdownButton', () => {
	it('copies the raw markdown prefixed with a URL comment, and shows feedback that resets', async () => {
		render(CopyMarkdownButton, {
			props: {
				raw: '# Getting Started\n\nSome content.',
				url: 'https://docs.acerola-comic.com/docs/getting-started'
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Copy page as Markdown' }));

		expect(writeText).toHaveBeenCalledWith(
			'<!-- URL: https://docs.acerola-comic.com/docs/getting-started -->\n\n# Getting Started\n\nSome content.'
		);
		expect(screen.getByRole('button', { name: 'Copied' })).toBeInTheDocument();

		await vi.advanceTimersByTimeAsync(2000);

		expect(screen.getByRole('button', { name: 'Copy page as Markdown' })).toBeInTheDocument();
	});

	it('uses custom labels when provided', async () => {
		render(CopyMarkdownButton, {
			props: {
				raw: '# Doc',
				url: 'https://docs.acerola-comic.com/docs/doc',
				label: 'Copiar como Markdown',
				copiedLabel: 'Copiado'
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Copiar como Markdown' }));

		expect(screen.getByRole('button', { name: 'Copiado' })).toBeInTheDocument();
	});
});
