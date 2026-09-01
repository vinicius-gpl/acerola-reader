import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { error } from '@tauri-apps/plugin-log';
import { SvelteSet } from 'svelte/reactivity';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import { m } from '$lib/paraglide/messages';

// Contrato com o `code` opcional que o backend anexa ao payload de `sync:*:error` quando a
// mensagem tem uma tradução estável (ver `emit_busy`/`SESSION_BUSY_TAG` em `transfer.rs`) — o
// `message` cru continua em inglês (texto técnico de log/wire), então sem isso o erro de
// "sessão já em andamento" apareceria sem tradução na UI mesmo com o app todo em pt-BR.
const SYNC_ERROR_MESSAGES: Record<string, () => string> = {
	busy: m['tauri_errors.sync.session_busy.label']
};

function translateSyncMessage(code: string | undefined, message: string): string {
	return (code && SYNC_ERROR_MESSAGES[code]?.()) ?? message;
}

export type TransferLogEntry = {
	id: number;
	/** Vazio quando a origem não é resolvível (ex.: `files:progress`, que não carrega peer id). */
	peerId: string;
	kind: 'history' | 'files' | 'comic';
	status: 'started' | 'progress' | 'complete' | 'error';
	message: string;
	timestamp: number;
	/** Só presente em `kind: 'comic'` — o quadrinho escopado desta sessão (ver
	 *  `comic_handler.rs::COMPLETE_EVENT`/`ERROR_EVENT`, payload `{peerId, comicName}` desde a
	 *  extensão de cover/banner/ComicInfo.xml). Usado pela página do quadrinho pra saber se uma
	 *  sessão concluída afeta o quadrinho atualmente aberto, sem precisar assumir que qualquer
	 *  sync individual é sobre ele. */
	comicName?: string;
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

	// Resolvers de quem está esperando a conclusão *de verdade* de uma sessão (ver
	// `syncComic`) — só é populado por chamadas que pedem essa espera, então settlePending
	// é um no-op seguro pra `history`/`files`/`all`, que ninguém aguarda hoje.
	const pendingSettlement = new Map<
		string,
		{ resolve: (message: string) => void; reject: (message: string) => void }
	>();

	function settlePending(key: string, ok: boolean, message: string) {
		const pending = pendingSettlement.get(key);
		if (!pending) return;
		pendingSettlement.delete(key);
		if (ok) pending.resolve(message);
		else pending.reject(message);
	}

	function markSyncing(peerId: string, kind: SyncKind) {
		const key = syncKey(peerId, kind);
		syncingKeys.add(key);
		// Stryker disable next-line CallExpression: markSyncing só roda depois do guard
		// `kinds.some(isSyncing)` em withSyncGuard confirmar que a chave NÃO está syncing —
		// e `syncingKeys`/`inFlightTimeouts` são sempre atualizados juntos (markSyncing seta
		// os dois, clearSyncing apaga os dois), então `inFlightTimeouts.get(key)` aqui é
		// sempre `undefined`. Remover esta chamada defensiva não muda nenhum comportamento
		// observável.
		clearTimeout(inFlightTimeouts.get(key));
		inFlightTimeouts.set(
			key,
			setTimeout(() => {
				clearSyncing(peerId, kind);
				// Peer sumiu no meio da sessão sem emitir `complete`/`error` — sem isso, um
				// `syncComic` aguardando a conclusão real ficaria pendurado pra sempre.
				settlePending(key, false, 'sync timed out');
			}, IN_FLIGHT_TIMEOUT_MS)
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
	/// `{"peerId": ..., "message": ..., "comicName"?: ...}` (ver
	/// `history_handler.rs`/`file_handler.rs`/`comic_handler.rs`), ao contrário de `started`,
	/// que carrega o `peerId` puro como string.
	function parseErrorPayload(
		payload: string
	): { peerId?: string; message: string; comicName?: string; code?: string } {
		try {
			const parsed = JSON.parse(payload);
			// Stryker disable next-line ConditionalExpression,LogicalOperator: `typeof
			// parsed.message === 'string'` só é verdadeiro quando `parsed` já É um objeto de
			// fato — nenhum primitivo vindo de `JSON.parse` (number, string, boolean) expõe uma
			// propriedade `.message` do tipo string, então remover o check `typeof parsed ===
			// 'object'` nunca muda o resultado pra esses casos. Trocar o `&&` antes dele por `||`
			// também é equivalente: o único valor onde isso importaria é `parsed = null` (typeof
			// null === 'object', mas falsy) — aí o mutante avalia `null.message`, que lança
			// TypeError, capturado pelo catch logo abaixo, caindo no MESMO `return { message:
			// payload }` do fallback normal. A saída observável é idêntica, só o caminho muda.
			// Verificado empiricamente: aplicando cada mutante isoladamente, nenhum teste falha.
			if (parsed && typeof parsed === 'object' && typeof parsed.message === 'string') {
				return {
					peerId: parsed.peerId,
					message: parsed.message,
					comicName: parsed.comicName,
					code: typeof parsed.code === 'string' ? parsed.code : undefined
				};
			}
		} catch {
			// payload antigo/inesperado sem JSON — trata como mensagem crua.
		}
		return { message: payload };
	}

	/// Extrai `{peerId, comicName}` do payload de `sync:comic:complete` — só esse evento carrega
	/// `comicName` (ver `comic_handler.rs::COMPLETE_EVENT`); `sync:history:complete`/
	/// `sync:files:complete` continuam carregando só o `peerId` puro como string, então o
	/// fallback cobre esses dois sem quebrar.
	function parseCompletePayload(payload: string): { peerId: string; comicName?: string } {
		try {
			const parsed = JSON.parse(payload);
			if (parsed && typeof parsed === 'object' && typeof parsed.peerId === 'string') {
				return { peerId: parsed.peerId, comicName: parsed.comicName };
			}
		} catch {
			// `sync:history:complete`/`sync:files:complete`: payload é o peerId cru, não JSON.
		}
		return { peerId: payload };
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
		// Stryker disable next-line ConditionalExpression,StringLiteral: o resultado desta
		// função só é lido em `appendNewEntry`, sempre atrás de `peerId && isOpenStatus(status)`.
		// Toda chamada de `push(...)` com `status: 'progress'` neste arquivo (filesProgress,
		// comicProgress) passa `peerId: ''` de propósito — o protocolo não correlaciona eventos
		// `*:progress` com um peer (ver o comentário de `inFlightEntryId` acima). Ou seja, sempre
		// que `status === 'progress'` chega aqui, `peerId` já é falsy e curto-circuita
		// `appendNewEntry` antes de isOpenStatus importar — forçar o branch 'progress' pra
		// sempre-false ou trocar a string nunca muda nada observável. Verificado empiricamente.
		return status === 'started' || status === 'progress';
	}

	function updateInFlightEntry(
		index: number,
		key: string,
		status: TransferLogEntry['status'],
		message: string,
		comicName?: string
	) {
		log[index] = {
			...log[index],
			status,
			message,
			timestamp: Date.now(),
			comicName: comicName ?? log[index].comicName
		};
		if (isTerminalStatus(status)) inFlightEntryId.delete(key);
	}

	function appendNewEntry(
		peerId: string,
		kind: SyncKind,
		key: string,
		status: TransferLogEntry['status'],
		message: string,
		comicName?: string
	) {
		const entry: TransferLogEntry = {
			id: nextId++,
			peerId,
			kind,
			status,
			message,
			timestamp: Date.now(),
			comicName
		};
		log = [entry, ...log].slice(0, MAX_LOG_ENTRIES);
		if (peerId && isOpenStatus(status)) inFlightEntryId.set(key, entry.id);
	}

	function push(
		peerId: string,
		kind: SyncKind,
		status: TransferLogEntry['status'],
		message: string,
		comicName?: string
	) {
		const key = syncKey(peerId, kind);
		const existingId = peerId ? inFlightEntryId.get(key) : undefined;
		const existingIndex =
			// Stryker disable next-line ConditionalExpression: toda entrada de `log` recebe
			// `id: nextId++` (sempre um number) em `appendNewEntry` — nenhuma entrada jamais tem
			// `id === undefined`. Então quando `existingId` é genuinamente `undefined`, forçar a
			// condição pra `true` ainda roda `findIndex((entry) => entry.id === undefined)`, que
			// sempre retorna `-1` — idêntico ao literal `-1` do branch `false`. Verificado
			// empiricamente.
			existingId !== undefined ? log.findIndex((entry) => entry.id === existingId) : -1;

		if (existingIndex !== -1) {
			updateInFlightEntry(existingIndex, key, status, message, comicName);
		} else {
			appendNewEntry(peerId, kind, key, status, message, comicName);
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
				const { peerId, message, code } = parseErrorPayload(event.payload);
				if (peerId) clearSyncing(peerId, 'history');
				push(peerId ?? '', 'history', 'error', translateSyncMessage(code, message));
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
				const { peerId, message, code } = parseErrorPayload(event.payload);
				if (peerId) clearSyncing(peerId, 'files');
				push(peerId ?? '', 'files', 'error', translateSyncMessage(code, message));
			}),
			await listen<string>(NETWORK_EVENTS.comicStarted, (event) =>
				push(event.payload, 'comic', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.comicProgress, (event) =>
				push('', 'comic', 'progress', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.comicComplete, (event) => {
				const { peerId, comicName } = parseCompletePayload(event.payload);
				clearSyncing(peerId, 'comic');
				push(peerId, 'comic', 'complete', peerId, comicName);
				settlePending(syncKey(peerId, 'comic'), true, peerId);
			}),
			await listen<string>(NETWORK_EVENTS.comicError, (event) => {
				const { peerId, message, comicName, code } = parseErrorPayload(event.payload);
				const translated = translateSyncMessage(code, message);
				if (peerId) clearSyncing(peerId, 'comic');
				push(peerId ?? '', 'comic', 'error', translated, comicName);
				if (peerId) settlePending(syncKey(peerId, 'comic'), false, translated);
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
		// Sem isso, um `await syncComic(...)` cujo componente desmontou no meio da sessão
		// ficaria pendurado pra sempre — os listeners que resolveriam essa promise acabaram
		// de ser desregistrados acima.
		pendingSettlement.forEach(({ reject }) => reject('sync cancelled: listener stopped'));
		pendingSettlement.clear();
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
	///
	/// Ao contrário de `syncHistory`/`syncFiles`/`syncAll`, a promise retornada só resolve
	/// (ou rejeita) quando a sessão termina de verdade (`sync:comic:complete`/`error`, ou o
	/// timeout de `markSyncing`) — o `invoke` do comando Tauri resolve assim que a conexão é
	/// só enfileirada, muito antes do handshake/transferência acontecer, então usar aquela
	/// promise direto pra decidir sucesso/erro (como a tela do quadrinho fazia antes) dispara
	/// o toast de sucesso cedo demais e nunca mostra o de erro se a sessão falhar depois.
	async function syncComic(peerId: string, addrs: number[], comicName: string): Promise<string> {
		if (isSyncing(peerId, 'comic')) {
			// Mesmo texto traduzido do `code: "busy"` que o backend manda — este guard é
			// local (nem chega a abrir conexão), mas é o mesmo erro do ponto de vista do
			// usuário, então usa a mesma mensagem em vez de um texto cru em inglês.
			throw new Error(m['tauri_errors.sync.session_busy.label']());
		}

		const key = syncKey(peerId, 'comic');
		const settlement = new Promise<string>((resolve, reject) => {
			pendingSettlement.set(key, { resolve, reject });
		});

		try {
			await withSyncGuard(peerId, ['comic'], NETWORK_COMMANDS.syncComic, {
				peerId,
				addrs,
				comicName
			});
		} catch (err) {
			pendingSettlement.delete(key);
			throw err;
		}

		return settlement;
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
