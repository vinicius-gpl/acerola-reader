import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkRelaySettingsCard, {
	type NetworkRelaySettingsCardData
} from './acerola-network-relay-settings-card.svelte';

function data(overrides: Partial<NetworkRelaySettingsCardData> = {}): NetworkRelaySettingsCardData {
	return {
		acerolaRelayUrl: 'https://relay.acerola-comic.com',
		useAcerolaRelay: true,
		useIrohPublicNetwork: false,
		customRelayUrls: [],
		irohRelayUrls: [],
		...overrides
	};
}

function events() {
	return {
		onToggleAcerolaRelay: vi.fn(),
		onToggleIrohPublicNetwork: vi.fn(),
		onAddCustomRelayUrl: vi.fn(),
		onRemoveCustomRelayUrl: vi.fn(),
		onAddIrohRelayUrl: vi.fn(),
		onRemoveIrohRelayUrl: vi.fn()
	};
}

async function expandCard() {
	await fireEvent.click(screen.getByRole('button', { expanded: false }));
}

describe('AcerolaNetworkRelaySettingsCard', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('shows the mDNS-only summary when no relay source is active', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useAcerolaRelay: false }), events: events() }
		});

		expect(screen.getByText(/mDNS|Só rede local/i)).toBeInTheDocument();
	});

	it('shows an active-sources summary count', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: {
				data: data({ customRelayUrls: ['https://relay-a.test.local'] }),
				events: events()
			}
		});

		expect(screen.getByText(/2/)).toBeInTheDocument();
	});

	it('shows the iroh public network summary when active', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useIrohPublicNetwork: true }), events: events() }
		});

		expect(screen.getByText(/Iroh/i)).toBeInTheDocument();
	});

	it('toggles the acerola relay switch', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		const switches = screen.getAllByRole('switch');
		await fireEvent.click(switches[0]);

		expect(handlers.onToggleAcerolaRelay).toHaveBeenCalledWith(false);
	});

	it('toggles the iroh public network switch', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		const switches = screen.getAllByRole('switch');
		await fireEvent.click(switches[1]);

		expect(handlers.onToggleIrohPublicNetwork).toHaveBeenCalledWith(true);
	});

	it('adds a custom relay url', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		const input = screen.getByPlaceholderText(/your-relay\.example\.com|seu-relay\.exemplo\.com/i);
		await fireEvent.input(input, { target: { value: 'https://relay-a.test.local' } });
		await fireEvent.click(screen.getByRole('button', { name: /Add custom relay|Adicionar relay próprio/i }));

		expect(handlers.onAddCustomRelayUrl).toHaveBeenCalledWith('https://relay-a.test.local');
	});

	it('rejects an invalid custom relay url without calling the handler', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		const input = screen.getByPlaceholderText(/your-relay\.example\.com|seu-relay\.exemplo\.com/i);
		await fireEvent.input(input, { target: { value: 'not-a-url' } });
		await fireEvent.click(
			screen.getByRole('button', { name: /Add custom relay|Adicionar relay próprio/i })
		);

		expect(handlers.onAddCustomRelayUrl).not.toHaveBeenCalled();
		expect(screen.getByText(/valid URL|URL válida/i)).toBeInTheDocument();
	});

	it('removes a custom relay url', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, {
			props: {
				data: data({ customRelayUrls: ['https://relay-a.test.local'] }),
				events: handlers
			}
		});
		await expandCard();

		await fireEvent.click(
			screen.getByRole('button', { name: /Remove custom relay|Remover relay próprio/i })
		);

		expect(handlers.onRemoveCustomRelayUrl).toHaveBeenCalledWith('https://relay-a.test.local');
	});

	it('disables the custom/iroh inputs while the iroh public network is active', async () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useIrohPublicNetwork: true }), events: events() }
		});
		await expandCard();

		expect(
			screen.getByPlaceholderText(/your-relay\.example\.com|seu-relay\.exemplo\.com/i)
		).toBeDisabled();
		expect(
			screen.getByPlaceholderText(/iroh-relay\.example\.com|iroh-relay\.exemplo\.com/i)
		).toBeDisabled();
	});
});
