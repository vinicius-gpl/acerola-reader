import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { error } from '@tauri-apps/plugin-log';
import { SvelteSet } from 'svelte/reactivity';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';

export type TransferLogEntry = {
	id: number;
	/** Vazio quando a origem não é resolvível (ex.: `files:progress`, que não carrega peer id). */
	peerId: string;
	kind: 'history' | 'files' | 'comic';
	status: 'started' | 'progress' | 'complete' | 'error';
	message: string;
	timestamp: number;
};

type SyncKind = TransferLogEntry['kind'];

/// Espelha `SyncHistoryLogEntry` (`data/models/sync/sync_history_log.rs`, serializado
/// `camelCase`) — uma linha do histórico persistido no SQLite, carregada uma vez ao montar
/// a tela e combinada com os eventos ao vivo da sessão atual.
type PersistedSyncLogEntry = {
	id: number;
	peerId: string;
	kind: string;
	status: string;
	message: string | null;
	createdAt: number;
};

const MAX_LOG_ENTRIES = 200;
// Rede real pode nunca devolver o evento de conclusão/erro (peer sumiu no meio da
// sessão) — sem isso os botões daquele peer ficariam desabilitados pra sempre.
const IN_FLIGHT_TIMEOUT_MS = 60_000;

function syncKey(peerId: string, kind: SyncKind): string {
	return `${peerId}:${kind}`;
}

/// Converte uma linha persistida pro mesmo formato que os eventos ao vivo produzem — pra
/// `describeEntry()` (na página) não precisar saber a diferença. Igual ao `push()` abaixo,
/// `message` carrega o peer id pra status `started`/`complete`/`progress` (usado por
/// `peers.peerLabel(...)`) e o texto de erro de verdade só pra `error`.
function fromPersisted(row: PersistedSyncLogEntry): TransferLogEntry {
	return {
		// Negativo pra nunca colidir com os ids ao vivo (`nextId` começa em 0 e só cresce).
		id: -row.id,
		peerId: row.peerId,
		kind: row.kind as SyncKind,
		status: row.status as TransferLogEntry['status'],
		message: row.status === 'error' ? (row.message ?? '') : row.peerId,
		timestamp: row.createdAt
	};
}

/// Dispara sessões de sync (histórico/arquivos) contra um peer já pareado e mantém um
/// log reativo do progresso, alimentado pelos eventos `sync:history:*`/`sync:files:*` que
/// os protocol handlers do Rust emitem direto (ver `infra::sync::protocol` no backend), mais
/// o histórico persistido carregado uma vez em `startListening()` (sobrevive a restart —
/// ver `get_sync_history_log`/`sync_history_log` no backend).
export function useNetworkSync() {
	let log = $state<TransferLogEntry[]>([]);
	let nextId = 0;
	const unlisten: UnlistenFn[] = [];

	// `loadPersistedLog` pode seguir rodando depois que `stopListening` já rodou (unmount no
	// meio do await) — sem essa flag, a continuação tenta gravar em $state de uma instância já
	// descartada (`useNetworkSync()` é sempre uma instância nova por mount).
	let disposed = false;

	// Chaves `peerId:kind` com uma sessão de sync em andamento — usado só pra desabilitar
	// os botões daquele peer/protocolo e evitar disparar uma segunda sessão concorrente
	// pro mesmo par (peer, ALPN) (ver histórico de erro "stream closed before history
	// manifest" causado por duas sessões simultâneas competindo pelo mesmo stream).
	// Granularidade por `kind`, não só por peer: "Sync All" dispara histórico E arquivos
	// pro mesmo peer, e os dois precisam poder rodar ao mesmo tempo sem se bloquear.
	const syncingKeys = new SvelteSet<string>();
	const inFlightTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	function markSyncing(peerId: string, kind: SyncKind) {
		const key = syncKey(peerId, kind);
		syncingKeys.add(key);
		clearTimeout(inFlightTimeouts.get(key));
		inFlightTimeouts.set(
			key,
			setTimeout(() => clearSyncing(peerId, kind), IN_FLIGHT_TIMEOUT_MS)
		);
	}

	function clearSyncing(peerId: string, kind: SyncKind) {
		const key = syncKey(peerId, kind);
		syncingKeys.delete(key);
		clearTimeout(inFlightTimeouts.get(key));
		inFlightTimeouts.delete(key);
	}

	function isSyncing(peerId: string, kind: SyncKind): boolean {
		return syncingKeys.has(syncKey(peerId, kind));
	}

	/// Extrai o `peerId` do payload de um evento `sync:*:error` — esses eventos carregam
	/// `{"peerId": ..., "message": ...}` (ver `history_handler.rs`/`file_handler.rs`), ao
	/// contrário de `started`/`complete`, que carregam o `peerId` puro como string.
	function parseErrorPayload(payload: string): { peerId?: string; message: string } {
		try {
			const parsed = JSON.parse(payload);
			if (parsed && typeof parsed === 'object' && typeof parsed.message === 'string') {
				return { peerId: parsed.peerId, message: parsed.message };
			}
		} catch {
			// payload antigo/inesperado sem JSON — trata como mensagem crua.
		}
		return { message: payload };
	}

	/// Uma sessão de sync tem UMA linha no log, que transiciona de estado (started -> progress
	/// -> complete/error) em vez de empilhar uma linha nova por evento — sem isso, "started"
	/// fica pra sempre como uma linha separada com spinner girando ao lado da linha "complete"
	/// que já resolveu a mesma sessão. Chave `peerId:kind` (mesma granularidade de
	/// `syncingKeys`). `files:progress` não carrega peer id no payload (ver o comentário do
	/// tipo `TransferLogEntry`), então não dá pra correlacionar com a linha `started` de um
	/// peer específico — continua criando linha própria, é limitação do protocolo, não bug
	/// desta função.
	const inFlightEntryId = new Map<string, number>();

	function isTerminalStatus(status: TransferLogEntry['status']): boolean {
		return status === 'complete' || status === 'error';
	}

	function isOpenStatus(status: TransferLogEntry['status']): boolean {
		return status === 'started' || status === 'progress';
	}

	function updateInFlightEntry(
		index: number,
		key: string,
		status: TransferLogEntry['status'],
		message: string
	) {
		log[index] = { ...log[index], status, message, timestamp: Date.now() };
		if (isTerminalStatus(status)) inFlightEntryId.delete(key);
	}

	function appendNewEntry(
		peerId: string,
		kind: SyncKind,
		key: string,
		status: TransferLogEntry['status'],
		message: string
	) {
		const entry: TransferLogEntry = {
			id: nextId++,
			peerId,
			kind,
			status,
			message,
			timestamp: Date.now()
		};
		log = [entry, ...log].slice(0, MAX_LOG_ENTRIES);
		if (peerId && isOpenStatus(status)) inFlightEntryId.set(key, entry.id);
	}

	function push(
		peerId: string,
		kind: SyncKind,
		status: TransferLogEntry['status'],
		message: string
	) {
		const key = syncKey(peerId, kind);
		const existingId = peerId ? inFlightEntryId.get(key) : undefined;
		const existingIndex =
			existingId !== undefined ? log.findIndex((entry) => entry.id === existingId) : -1;

		if (existingIndex !== -1) {
			updateInFlightEntry(existingIndex, key, status, message);
		} else {
			appendNewEntry(peerId, kind, key, status, message);
		}
	}

	/// Carrega as sessões persistidas (mais recente primeiro) pra dar contexto histórico
	/// assim que a tela abre — antes de qualquer evento ao vivo chegar. Falha em silêncio:
	/// sem histórico persistido, a tela ainda funciona só com os eventos da sessão atual.
	async function loadPersistedLog() {
		try {
			const rows = await invoke<PersistedSyncLogEntry[]>(NETWORK_COMMANDS.getSyncHistoryLog);
			if (disposed) return;
			log = rows.map(fromPersisted);
		} catch (err) {
			if (disposed) return;
			error(`failed to load persisted sync history log: ${err}`);
		}
	}

	async function startListening() {
		await loadPersistedLog();

		unlisten.push(
			await listen<string>(NETWORK_EVENTS.historyStarted, (event) =>
				push(event.payload, 'history', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.historyComplete, (event) => {
				clearSyncing(event.payload, 'history');
				push(event.payload, 'history', 'complete', event.payload);
			}),
			await listen<string>(NETWORK_EVENTS.historyError, (event) => {
				const { peerId, message } = parseErrorPayload(event.payload);
				if (peerId) clearSyncing(peerId, 'history');
				push(peerId ?? '', 'history', 'error', message);
			}),
			await listen<string>(NETWORK_EVENTS.filesStarted, (event) =>
				push(event.payload, 'files', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.filesProgress, (event) =>
				push('', 'files', 'progress', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.filesComplete, (event) => {
				clearSyncing(event.payload, 'files');
				push(event.payload, 'files', 'complete', event.payload);
			}),
			await listen<string>(NETWORK_EVENTS.filesError, (event) => {
				const { peerId, message } = parseErrorPayload(event.payload);
				if (peerId) clearSyncing(peerId, 'files');
				push(peerId ?? '', 'files', 'error', message);
			}),
			await listen<string>(NETWORK_EVENTS.comicStarted, (event) =>
				push(event.payload, 'comic', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.comicProgress, (event) =>
				push('', 'comic', 'progress', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.comicComplete, (event) => {
				clearSyncing(event.payload, 'comic');
				push(event.payload, 'comic', 'complete', event.payload);
			}),
			await listen<string>(NETWORK_EVENTS.comicError, (event) => {
				const { peerId, message } = parseErrorPayload(event.payload);
				if (peerId) clearSyncing(peerId, 'comic');
				push(peerId ?? '', 'comic', 'error', message);
			})
		);
	}

	function stopListening() {
		disposed = true;
		unlisten.forEach((fn) => fn());
		unlisten.length = 0;
		inFlightTimeouts.forEach((timeout) => clearTimeout(timeout));
		inFlightTimeouts.clear();
		inFlightEntryId.clear();
	}

	/// Chama o comando Tauri de sync marcando/limpando a(s) chave(s) `peerId:kind` ao redor
	/// da invocação em si — o clear "de verdade" (sessão concluída/com erro) acontece nos
	/// listeners de evento acima; isto aqui só cobre o caso do comando falhar imediatamente
	/// (peer não resolvido, canal fechado etc.), antes de qualquer evento de rede ser emitido.
	async function withSyncGuard(
		peerId: string,
		kinds: SyncKind[],
		command: string,
		args: Record<string, unknown>
	) {
		if (kinds.some((kind) => isSyncing(peerId, kind))) return;
		kinds.forEach((kind) => markSyncing(peerId, kind));
		try {
			await invoke(command, args);
		} catch (error) {
			kinds.forEach((kind) => clearSyncing(peerId, kind));
			throw error;
		}
	}

	async function syncHistory(peerId: string, addrs: number[]) {
		await withSyncGuard(peerId, ['history'], NETWORK_COMMANDS.syncHistory, { peerId, addrs });
	}

	async function syncFiles(peerId: string, addrs: number[]) {
		await withSyncGuard(peerId, ['files'], NETWORK_COMMANDS.syncFiles, { peerId, addrs });
	}

	async function syncAll(peerId: string, addrs: number[]) {
		await withSyncGuard(peerId, ['history', 'files'], NETWORK_COMMANDS.syncAll, { peerId, addrs });
	}

	/// Sync individual de UM quadrinho (push ou pull, ver `comic_handler.rs` no backend) — o
	/// `comicName` é o mesmo nome (`comic_directory.name`) usado como chave natural em todo o
	/// resto do protocolo de sync de arquivos.
	async function syncComic(peerId: string, addrs: number[], comicName: string) {
		await withSyncGuard(peerId, ['comic'], NETWORK_COMMANDS.syncComic, {
			peerId,
			addrs,
			comicName
		});
	}

	/// Timestamp da última sessão concluída com sucesso pra esse peer (qualquer `kind`), ou
	/// `undefined` se nunca sincronizou — usado pra mostrar "Última sync: ..." por peer.
	function lastSyncedAt(peerId: string): number | undefined {
		return log.find((entry) => entry.peerId === peerId && entry.status === 'complete')?.timestamp;
	}

	return {
		startListening,
		stopListening,
		syncHistory,
		syncFiles,
		syncAll,
		syncComic,
		isSyncing,
		lastSyncedAt,
		get log() {
			return log;
		}
	};
}
