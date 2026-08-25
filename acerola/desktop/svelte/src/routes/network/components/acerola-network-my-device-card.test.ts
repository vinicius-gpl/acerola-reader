import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkMyDeviceCard from './acerola-network-my-device-card.svelte';

vi.mock('svelte-sonner', () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

Object.assign(navigator, {
	clipboard: { writeText: vi.fn().mockResolvedValue(undefined) }
});

describe('AcerolaNetworkMyDeviceCard', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('shows the device name and short id', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'local',
					activeRelay: null,
					isRelayOverridden: false
				}
			}
		});

		expect(screen.getByText('Meu Notebook')).toBeInTheDocument();
	});

	it('disables the copy button when there is no local id yet', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: null,
					localId: null,
					mode: undefined,
					activeRelay: null,
					isRelayOverridden: false
				}
			}
		});

		expect(screen.getByRole('button', { name: /Copy|Copiar/i })).toBeDisabled();
	});

	it('copies the local id to the clipboard when the copy button is clicked', async () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'relay',
					activeRelay: 'relay.acerola.app',
					isRelayOverridden: true
				}
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /Copy|Copiar/i }));

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith('abcdefghijklmnop');
	});

	it('shows the active relay address', () => {
		render(AcerolaNetworkMyDeviceCard, {
			props: {
				data: {
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnop',
					mode: 'relay',
					activeRelay: 'relay.acerola.app',
					isRelayOverridden: false
				}
			}
		});

		expect(screen.getByText(/relay\.acerola\.app/)).toBeInTheDocument();
	});
});
