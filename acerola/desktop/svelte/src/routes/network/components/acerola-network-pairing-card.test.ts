import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkPairingCard from './acerola-network-pairing-card.svelte';

vi.mock('svelte-sonner', () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

vi.mock('qrcode', () => ({
	default: {
		toDataURL: vi.fn().mockResolvedValue('data:image/png;base64,fake')
	}
}));

Object.assign(navigator, {
	clipboard: { writeText: vi.fn().mockResolvedValue(undefined) }
});

describe('AcerolaNetworkPairingCard', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('disables the copy button when there is no code yet', () => {
		render(AcerolaNetworkPairingCard, { props: { data: { code: undefined } } });

		expect(screen.getByRole('button', { name: /Copy|Copiar/i })).toBeDisabled();
	});

	it('renders the QR code once generated from the connection code', async () => {
		render(AcerolaNetworkPairingCard, { props: { data: { code: 'connect-me' } } });

		expect(await screen.findByRole('img')).toBeInTheDocument();
	});

	it('copies the connection code to the clipboard', async () => {
		render(AcerolaNetworkPairingCard, { props: { data: { code: 'connect-me' } } });

		await fireEvent.click(screen.getByRole('button', { name: /Copy|Copiar/i }));

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith('connect-me');
	});

	it('toggles the raw code textarea', async () => {
		render(AcerolaNetworkPairingCard, { props: { data: { code: 'connect-me' } } });

		expect(screen.queryByRole('textbox')).not.toBeInTheDocument();

		await fireEvent.click(screen.getByRole('button', { name: /show code|mostrar código/i }));

		expect(screen.getByRole('textbox')).toHaveValue('connect-me');
	});
});
