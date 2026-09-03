import { render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AcerolaApkDownloadButton from './acerola-apk-download-button.svelte';

const originalFetch = global.fetch;

afterEach(() => {
	global.fetch = originalFetch;
	vi.restoreAllMocks();
});

function mockApkStatus(available: boolean) {
	global.fetch = vi.fn().mockResolvedValue({
		ok: true,
		json: () => Promise.resolve({ available })
	}) as unknown as typeof fetch;
}

describe('AcerolaApkDownloadButton', () => {
	beforeEach(() => {
		mockApkStatus(true);
	});

	it('links to the latest APK build while the availability check is pending or confirms it is up', () => {
		render(AcerolaApkDownloadButton);

		expect(screen.getByRole('link')).toHaveAttribute(
			'href',
			'https://binary.acerola-comic.com/android/latest.apk'
		);
	});

	it('shows the default caption above "APK"', () => {
		render(AcerolaApkDownloadButton);

		expect(screen.getByText('Get the')).toBeInTheDocument();
		expect(screen.getByText('APK')).toBeInTheDocument();
	});

	it('uses a custom caption when provided', () => {
		render(AcerolaApkDownloadButton, { props: { label: 'Baixe o' } });

		expect(screen.getByText('Baixe o')).toBeInTheDocument();
	});

	it('swaps to a GitHub Releases fallback link once the check reports the build is unavailable', async () => {
		mockApkStatus(false);
		render(AcerolaApkDownloadButton);

		await waitFor(() =>
			expect(screen.getByRole('link')).toHaveAttribute(
				'href',
				'https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/releases/latest'
			)
		);

		expect(screen.getByRole('link')).toHaveAttribute('target', '_blank');
		expect(screen.getByRole('link')).toHaveAttribute('rel', 'noopener noreferrer');
		expect(screen.getByText('GitHub Releases')).toBeInTheDocument();
	});

	it('uses a custom fallback caption when provided', async () => {
		mockApkStatus(false);
		render(AcerolaApkDownloadButton, { props: { fallbackLabel: 'Indisponível — tente' } });

		expect(await screen.findByText('Indisponível — tente')).toBeInTheDocument();
	});

	it('keeps the direct download link when the availability check itself fails', async () => {
		global.fetch = vi.fn().mockRejectedValue(new Error('network error')) as unknown as typeof fetch;
		render(AcerolaApkDownloadButton);

		// Não há um estado assíncrono visível pra esperar (o fallback nunca é ativado),
		// então dá um tick pra deixar a promise rejeitada do onMount assentar antes de
		// checar que o link continua o de download direto.
		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(screen.getByRole('link')).toHaveAttribute(
			'href',
			'https://binary.acerola-comic.com/android/latest.apk'
		);
	});
});
