import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import RootLayout from '../+layout.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

vi.mock('$app/stores', async () => {
	const { writable: makeWritable } = await import('svelte/store');
	const store = makeWritable({ url: new URL('http://localhost/home') });
	return { page: store };
});

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// Mapa por nome de evento (não um mock genérico que sempre devolve um unlisten vazio) — pra
// poder simular `sync:files:started`/`sync:files:complete` chegando de verdade e provar que o
// indicador global de sync no header (`headerSync`/`headerPeers`, ver `+layout.svelte`) reage a
// eles. Mesmo padrão de `setupListeners()` em `use-network-sync.test.ts`.
const { mockListenCallbacks } = vi.hoisted(() => ({
	mockListenCallbacks: new Map<string, (event: { payload: unknown }) => void>()
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: (event: string, callback: (event: { payload: unknown }) => void) => {
		mockListenCallbacks.set(event, callback);
		return Promise.resolve(vi.fn());
	}
}));

const { mockMinimize, mockToggleMaximize, mockCloseWin } = vi.hoisted(() => ({
	mockMinimize: vi.fn(),
	mockToggleMaximize: vi.fn(),
	mockCloseWin: vi.fn()
}));

vi.mock('@tauri-apps/api/window', () => ({
	getCurrentWindow: () => ({
		theme: vi.fn().mockResolvedValue('light'),
		setTheme: vi.fn().mockResolvedValue(undefined),
		onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() }),
		minimize: mockMinimize,
		toggleMaximize: mockToggleMaximize,
		close: mockCloseWin
	})
}));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(), debug: vi.fn() }));

// checkStatus() do useOnboarding roda uma única vez, no import do módulo (top-level, não
// dentro do hook) — por isso o LazyStore precisa já devolver "completo" aqui na factory do
// vi.mock (hoisted antes de qualquer import), não em beforeEach: nesse ponto já é tarde.
vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn().mockResolvedValue({
		get: vi.fn().mockResolvedValue(undefined),
		set: vi.fn().mockResolvedValue(undefined),
		delete: vi.fn().mockResolvedValue(undefined),
		save: vi.fn().mockResolvedValue(undefined)
	}),
	LazyStore: class {
		constructor() {
			return {
				// só "onboarding_completed" precisa ser true — o mesmo LazyStore é
				// reaproveitado por use-theme.svelte.ts (chaves "theme"/"mode"), que quebra se
				// receber `true` como valor de tema/modo em vez de undefined (cai no default).
				get: vi.fn((key: string) =>
					Promise.resolve(key === 'onboarding_completed' ? true : undefined)
				),
				set: vi.fn().mockResolvedValue(undefined)
			};
		}
	}
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: [],
		get_package_family_name: 'No package identity',
		// `headerPeers`/`headerSync` (indicador global de sync no header) chamam `startListening()`
		// no `onMount` — sem esses defaults, `usePeerConnection::loadPairedPeers` recebia
		// `undefined` de volta e quebrava tentando iterar (`for...of undefined`), gerando uma
		// unhandled rejection nos outros testes deste arquivo (mesmas fixtures de
		// `use-peer-connection.test.ts`, pra bater com o shape real esperado por cada hook).
		[NETWORK_COMMANDS.getLocalId]: 'local-id',
		[NETWORK_COMMANDS.getLocalAddr]: { id: { id: 'local-id', device_id: null }, addrs: [1, 2] },
		[NETWORK_COMMANDS.getLocalDeviceInfo]: { name: 'Desktop', os: 'windows', version: '1.0' },
		[NETWORK_COMMANDS.getRelayInfo]: {
			acerolaRelayUrl: 'relay-a',
			useAcerolaRelay: true,
			useIrohPublicNetwork: false,
			customRelayUrls: [],
			hasIrohServicesTicket: false
		},
		[NETWORK_COMMANDS.getPairedPeers]: [],
		[NETWORK_COMMANDS.getSyncHistoryLog]: []
	};
	mockInvoke.mockImplementation((cmd: string) =>
		Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined))
	);
}

function childrenSnippet() {
	return createRawSnippet(() => ({
		render: () => `<div data-testid="page-content">Conteúdo da página</div>`
	}));
}

describe('root +layout', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockListenCallbacks.clear();
		setupInvokeMock();
	});

	it('renders the sidebar, header and page content once onboarding is complete', async () => {
		render(RootLayout, { props: { children: childrenSnippet() } });

		expect(await screen.findByTestId('page-content')).toBeInTheDocument();
		expect(screen.getByText('Acerola')).toBeInTheDocument();
	});

	it('calls the native window controls', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });

		await screen.findByTestId('page-content');
		await waitFor(() => expect(mockInvoke).toHaveBeenCalled());

		await user.click(screen.getByRole('button', { name: /minimize|minimizar/i }));
		expect(mockMinimize).toHaveBeenCalled();

		await user.click(screen.getByRole('button', { name: /maximize|maximizar/i }));
		expect(mockToggleMaximize).toHaveBeenCalled();

		await user.click(screen.getByRole('button', { name: /close|fechar/i }));
		expect(mockCloseWin).toHaveBeenCalled();
	});

	it('opens the command palette on ctrl+k and navigates to a section', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });

		await screen.findByTestId('page-content');

		await user.keyboard('{Control>}k{/Control}');

		const historyItem = await screen.findByText(/histórico|history/i);
		await user.click(historyItem);

		expect(mockGoto).toHaveBeenCalledWith('/history');
	});

	// Regressão do problema real relatado: um sync de arquivos rodando sem NENHUM indício
	// visual em lugar nenhum da UI — o usuário só descobria olhando o log do backend. O
	// indicador global no header (`headerSync.activeSession()`) precisa aparecer assim que a
	// sessão começa (`sync:files:started`) e sumir assim que ela termina (`sync:files:complete`),
	// independente de qual página está montada — por isso vive no `+layout.svelte`, não numa
	// tela específica.
	it('shows a global sync indicator in the header while a files session is active, and hides it once it completes', async () => {
		render(RootLayout, { props: { children: childrenSnippet() } });
		await screen.findByTestId('page-content');

		expect(screen.queryByText(/sincronizando|syncing/i)).not.toBeInTheDocument();

		// `startListening()` registra os listeners em série (cada `await listen(...)`) depois
		// de já ter esperado `loadPersistedLog()` — sem isso, disparar o evento logo após
		// `findByTestId('page-content')` corre o risco de ainda não ter o callback registrado
		// no Map, e o teste ficaria esperando um texto que nunca chega.
		await waitFor(() => expect(mockListenCallbacks.has(NETWORK_EVENTS.filesStarted)).toBe(true));
		mockListenCallbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-1' });

		expect(await screen.findByText(/sincronizando|syncing/i)).toBeInTheDocument();

		mockListenCallbacks.get(NETWORK_EVENTS.filesComplete)?.({ payload: 'peer-1' });

		await waitFor(() => {
			expect(screen.queryByText(/sincronizando|syncing/i)).not.toBeInTheDocument();
		});
	});

	it('clicking the global sync indicator navigates to the network page', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });
		await screen.findByTestId('page-content');

		await waitFor(() => expect(mockListenCallbacks.has(NETWORK_EVENTS.comicStarted)).toBe(true));
		mockListenCallbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-2' });
		const indicator = await screen.findByRole('button', { name: /sincronizando|syncing/i });

		await user.click(indicator);

		expect(mockGoto).toHaveBeenCalledWith('/network');
	});
});
