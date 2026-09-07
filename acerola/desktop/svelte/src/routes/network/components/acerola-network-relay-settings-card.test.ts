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
		hasIrohServicesTicket: false,
		...overrides
	};
}

function events() {
	return {
		onToggleAcerolaRelay: vi.fn().mockResolvedValue(undefined),
		onToggleIrohPublicNetwork: vi.fn().mockResolvedValue(undefined),
		onAddCustomRelayUrl: vi.fn().mockResolvedValue(undefined),
		onRemoveCustomRelayUrl: vi.fn().mockResolvedValue(undefined),
		onSetIrohServicesTicket: vi.fn().mockResolvedValue(undefined),
		onClearIrohServicesTicket: vi.fn().mockResolvedValue(undefined),
		onRestart: vi.fn().mockResolvedValue(undefined)
	};
}

async function expandCard() {
	await fireEvent.click(screen.getByRole('button', { expanded: false }));
}

function acerolaRelayCardButton() {
	return screen.getByRole('button', { name: /Use Acerola's relay|Usar o relay do Acerola/i });
}

function customRelaysCardButton() {
	return screen.getByRole('button', { name: /Your own relays|Seus relays próprios/i });
}

function irohCardButton() {
	return screen.getByRole('button', {
		name: /Use Iroh Services \(own account\)|Usar a Iroh Services \(conta própria\)/i
	});
}

/// O gerenciamento de ticket fica atrás de um toggle próprio dentro do card 3 (ver
/// `ticketExpanded` no componente) — pré-expandido só quando ainda não há ticket salvo.
async function expandTicketSection() {
	await fireEvent.click(
		screen.getByRole('button', {
			name: /Ticket configured|No ticket configured yet|Ticket configurado|Nenhum ticket configurado/i
		})
	);
}

async function expandCustomRelaysSection() {
	await fireEvent.click(customRelaysCardButton());
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

	it('toggles the acerola relay card', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		await fireEvent.click(acerolaRelayCardButton());

		expect(handlers.onToggleAcerolaRelay).toHaveBeenCalledWith(false);
	});

	it('shows a check badge on the acerola relay card only while it is active', async () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useAcerolaRelay: true }), events: events() }
		});
		await expandCard();

		expect(acerolaRelayCardButton().querySelector('svg.lucide-check')).toBeInTheDocument();
	});

	it('toggles the iroh public network card when a ticket is configured', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: true }), events: handlers }
		});
		await expandCard();

		await fireEvent.click(irohCardButton());

		expect(handlers.onToggleIrohPublicNetwork).toHaveBeenCalledWith(true);
	});

	it('disables the cards while a toggle is still applying and ignores a second click on it', async () => {
		const handlers = events();
		let resolveToggle: (() => void) | undefined;
		handlers.onToggleAcerolaRelay.mockImplementation(
			() =>
				new Promise<void>((resolve) => {
					resolveToggle = resolve;
				})
		);
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: true }), events: handlers }
		});
		await expandCard();

		await fireEvent.click(acerolaRelayCardButton());

		expect(handlers.onToggleAcerolaRelay).toHaveBeenCalledTimes(1);
		expect(acerolaRelayCardButton()).toBeDisabled();
		// O card Iroh só ficaria habilitado aqui se não fosse pelo guard de `restarting`
		// (ticket já configurado, então não é a razão de estar desabilitado).
		expect(irohCardButton()).toBeDisabled();

		// Um segundo clique enquanto a primeira mudança ainda não terminou (clique duplo, ou
		// mexer em outra fonte de relay) não pode disparar uma segunda restart em paralelo —
		// nem pelo atributo `disabled` (bloqueado nativamente), nem pela guarda em
		// `runRestartingAction` se o clique chegasse a acontecer de outra forma.
		await fireEvent.click(acerolaRelayCardButton());
		expect(handlers.onToggleAcerolaRelay).toHaveBeenCalledTimes(1);

		resolveToggle?.();
	});

	it('disables the iroh public network card without a configured ticket', async () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: false }), events: events() }
		});
		await expandCard();

		expect(irohCardButton()).toBeDisabled();
	});

	it('saves an iroh services ticket', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		// Sem ticket configurado, a seção já vem pré-expandida — não precisa clicar pra abrir.
		const input = screen.getByPlaceholderText(/services\.iroh\.computer/i);
		await fireEvent.input(input, { target: { value: 'services-fake-ticket' } });
		await fireEvent.click(screen.getByRole('button', { name: /Save ticket|Salvar ticket/i }));

		expect(handlers.onSetIrohServicesTicket).toHaveBeenCalledWith('services-fake-ticket');
	});

	it('shows an error when saving an invalid ticket fails', async () => {
		const handlers = events();
		handlers.onSetIrohServicesTicket.mockRejectedValueOnce(new Error('invalid ticket'));
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		const input = screen.getByPlaceholderText(/services\.iroh\.computer/i);
		await fireEvent.input(input, { target: { value: 'not-a-valid-ticket' } });
		await fireEvent.click(screen.getByRole('button', { name: /Save ticket|Salvar ticket/i }));

		expect(await screen.findByText(/[Ii]nvalid ticket|[Tt]icket inválido/i)).toBeInTheDocument();
	});

	it('removes a configured iroh services ticket', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: true }), events: handlers }
		});
		await expandCard();
		// Ticket já configurado — a seção vem recolhida por padrão, precisa expandir primeiro.
		await expandTicketSection();

		await fireEvent.click(screen.getByRole('button', { name: /Remove ticket|Remover ticket/i }));

		expect(handlers.onClearIrohServicesTicket).toHaveBeenCalled();
	});

	it('adds a custom relay url', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();
		await expandCustomRelaysSection();

		const input = screen.getByPlaceholderText(/your-relay\.example\.com|seu-relay\.exemplo\.com/i);
		await fireEvent.input(input, { target: { value: 'https://relay-a.test.local' } });
		await fireEvent.click(
			screen.getByRole('button', { name: /Add custom relay|Adicionar relay próprio/i })
		);

		expect(handlers.onAddCustomRelayUrl).toHaveBeenCalledWith('https://relay-a.test.local');
	});

	it('rejects an invalid custom relay url without calling the handler', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();
		await expandCustomRelaysSection();

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
		await expandCustomRelaysSection();

		await fireEvent.click(
			screen.getByRole('button', { name: /Remove custom relay|Remover relay próprio/i })
		);

		expect(handlers.onRemoveCustomRelayUrl).toHaveBeenCalledWith('https://relay-a.test.local');
	});

	it('shows a check badge on the custom relays card once at least one url is configured', async () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: {
				data: data({ customRelayUrls: ['https://relay-a.test.local'] }),
				events: events()
			}
		});
		await expandCard();

		expect(customRelaysCardButton().querySelector('svg.lucide-check')).toBeInTheDocument();
	});

	it('disables the custom relay input while the iroh public network is active', async () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useIrohPublicNetwork: true }), events: events() }
		});
		await expandCard();
		await expandCustomRelaysSection();

		expect(
			screen.getByPlaceholderText(/your-relay\.example\.com|seu-relay\.exemplo\.com/i)
		).toBeDisabled();
	});

	it('restarts the p2p module', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		await fireEvent.click(
			screen.getByRole('button', { name: /Restart P2P module|Reiniciar módulo P2P/i })
		);

		expect(handlers.onRestart).toHaveBeenCalled();
	});

	it('shows an error when restarting fails', async () => {
		const handlers = events();
		handlers.onRestart.mockRejectedValueOnce(new Error('restart failed'));
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCard();

		await fireEvent.click(
			screen.getByRole('button', { name: /Restart P2P module|Reiniciar módulo P2P/i })
		);

		expect(
			await screen.findByText(/Couldn't restart the P2P module|Não foi possível reiniciar/i)
		).toBeInTheDocument();
	});
});
