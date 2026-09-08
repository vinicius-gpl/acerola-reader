import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CONTEXT_KEYS } from '$lib/constants/context-keys';
import HistoryPage from '../+page.svelte';

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

const { mockInvoke, mockListen } = vi.hoisted(() => ({
	mockInvoke: vi.fn(),
	mockListen: vi.fn()
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: any) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// HistoryPage chama peers.startListening()/sync.startListening() no onMount, que dependem
// de `listen()` (Tauri IPC real, indisponível fora de uma webview) — sem mock aqui, a
// promise rejeitada vaza como unhandled rejection e derruba a suíte mesmo com os asserts
// passando. O `mockResolvedValue` é reaplicado a cada teste (ver beforeEach) porque
// `vi.resetAllMocks()` limpa a config do mock — sem reaplicar, `listen()` volta a
// resolver `undefined`, e `unlisten.push(await listen(...))` empurra `undefined` pro
// array, quebrando o `unlisten.forEach((fn) => fn())` do stopListening.
vi.mock('@tauri-apps/api/event', () => ({
	listen: (...args: unknown[]) => mockListen(...args)
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

vi.mock('$lib/assets/placeholder/placeholder_manga.svg?component', () => ({
	default: () => ''
}));

const mockHistoryData = [
	{
		comicDirectoryId: '1',
		chapterArchiveId: '10',
		lastPage: 5,
		isCompleted: false,
		updatedAt: 1600000000,
		comicName: 'Comic 1',
		comicCover: '/path/to/cover1.jpg',
		chapterName: '1',
		folderName: 'Comic 1',
		chapterPath: '/path/to/chapter1.cbz',
		chapterSort: '1',
		isSpecial: false,
		lastModified: 0
	},
	{
		comicDirectoryId: '2',
		chapterArchiveId: '20',
		lastPage: 12,
		isCompleted: true,
		updatedAt: 1500000000,
		comicName: 'Comic 2',
		comicCover: null,
		chapterName: '2',
		folderName: 'Comic 2',
		chapterPath: '/path/to/chapter2.cbz',
		chapterSort: '2',
		isSpecial: false,
		lastModified: 0
	}
];

// onMount desta pagina dispara history.fetch() junto com peers.startListening() e
// sync.startListening(), que chamam invoke() concorrentemente (get_paired_peers,
// get_sync_history_log, get_network_status). mockResolvedValueOnce só olha a ordem de
// chamada, não o comando — sob concorrência real qualquer um desses podia "roubar" o
// valor pensado pra history_get_all. Roteando por nome de comando isso deixa de importar.
function setupInvokeMock(overrides: Record<string, unknown> = {}) {
	const defaults: Record<string, unknown> = {
		get_paired_peers: [],
		get_sync_history_log: []
	};
	mockInvoke.mockImplementation((cmd: string) =>
		Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined))
	);
}

// `sync` agora vem do contexto compartilhado com `+layout.svelte` (ver
// `CONTEXT_KEYS.networkSync`), não de uma instância própria da página — sem prover esse
// contexto aqui, `getContext(...)` devolve `undefined` e o primeiro acesso a `sync.log`
// (dentro do `$effect` da página) explode.
function renderHistoryPage() {
	return render(HistoryPage, {
		context: new Map([
			[
				CONTEXT_KEYS.networkSync,
				{
					log: [],
					isSyncing: () => false,
					lastSyncedAt: () => undefined,
					activeSession: () => undefined,
					activeProgressMessage: () => undefined,
					syncHistory: vi.fn(),
					syncFiles: vi.fn(),
					syncComic: vi.fn(),
					syncAll: vi.fn(),
					startListening: vi.fn(),
					stopListening: vi.fn()
				}
			]
		])
	});
}

describe('HistoryPage', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		mockListen.mockResolvedValue(() => {});
	});

	it('renders empty state when there is no history', async () => {
		// Renderiza o empty state quando não houver histórico
		setupInvokeMock({ history_get_all: [] });
		renderHistoryPage();

		await waitFor(() => {
			expect(mockInvoke).toHaveBeenCalledWith('history_get_all', undefined);
		});

		expect(screen.getByText(/Nenhum histórico encontrado/i)).toBeInTheDocument();
	});

	it('renders history items successfully', async () => {
		// Renderiza os itens de histórico com sucesso
		setupInvokeMock({ history_get_all: mockHistoryData });
		renderHistoryPage();

		await waitFor(() => {
			expect(screen.getByText('Comic 1')).toBeInTheDocument();
			expect(screen.getByText('Comic 2')).toBeInTheDocument();
		});
	});

	it('clears history when clicking the button', async () => {
		// Limpa o histórico ao clicar no botão de limpar
		const user = userEvent.setup();
		setupInvokeMock({ history_get_all: mockHistoryData });
		renderHistoryPage();

		await waitFor(() => {
			expect(screen.getByText('Comic 1')).toBeInTheDocument();
		});

		// bits-ui AlertDialog.Trigger envolve o elemento filho em seu próprio <button>, portanto dois botões
		// com o mesmo nome acessível são renderizados. Pegamos o gatilho mais externo (índice 0).
		const clearButtons = screen.getAllByRole('button', { name: /Limpar Histórico/i });
		await user.click(clearButtons[0]);

		// Após abrir o diálogo, clica no botão de ação de confirmação ("Sim, limpar tudo")
		const confirmButton = await screen.findByRole('button', { name: /Sim, limpar tudo/i });
		await user.click(confirmButton);

		expect(mockInvoke).toHaveBeenCalledWith('history_clear', undefined);

		await waitFor(() => {
			expect(screen.queryByText('Comic 1')).not.toBeInTheDocument();
		});
	});
});
