import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SvelteMap, SvelteSet } from 'svelte/reactivity';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import { resolveArtworkPath } from '$lib/utils/artwork.utils';
import type {
	ComicSummary,
	CoverQueryResultPayload,
	LibraryQueryResultPayload
} from '$lib/contracts/network/network.payloads';

const coverKey = (peerId: string, comicName: string) => `${peerId}:${comicName}`;

/// Espelha o payload de erro dos outros eventos `sync:*:error` (ver
/// `use-network-sync.svelte.ts::parseErrorPayload`) — `library:query:error` carrega
/// `{ peerId, message }` no mesmo formato.
function parseErrorPayload(payload: string): { peerId?: string; message: string } {
	try {
		const parsed = JSON.parse(payload);
		// Stryker disable next-line ConditionalExpression,LogicalOperator: `typeof parsed.message
		// === 'string'` só é verdadeiro quando `parsed` já É um objeto de fato — nenhum primitivo
		// vindo de `JSON.parse` (number, string, boolean) expõe uma propriedade `.message` do tipo
		// string, então remover o check `typeof parsed === 'object'` nunca muda o resultado pra
		// esses casos. Trocar o `&&` antes dele por `||` (LogicalOperator) também é equivalente:
		// o único valor onde isso importaria é `parsed = null` (typeof null === 'object', mas
		// falsy) — aí o mutante avalia `null.message`, que lança TypeError, capturado pelo catch
		// logo abaixo, caindo no MESMO `return { message: payload }` do fallback normal. A saída
		// observável é idêntica nos dois casos, só o caminho (exceção vs. curto-circuito) muda.
		// Verificado empiricamente: aplicando cada mutante isoladamente, nenhum teste falha.
		if (parsed && typeof parsed === 'object' && typeof parsed.message === 'string') {
			return { peerId: parsed.peerId, message: parsed.message };
		}
	} catch {
		// payload inesperado sem JSON — trata como mensagem crua.
	}
	// Stryker disable next-line ObjectLiteral: este fallback nunca inclui `peerId` — o objeto
	// retornado só tem a chave `message`. O único consumidor (`libraryQueryError`) descarta o
	// evento inteiro via `if (!peerId) return;` antes de ler `message` nesse caminho, então
	// trocar `{ message: payload }` por `{}` não muda nenhum comportamento observável.
	return { message: payload };
}

/// Consulta e mantém em cache (por `peerId`) a lista de quadrinhos de bibliotecas remotas —
/// usado pra "puxar" um quadrinho individual que só existe em outro dispositivo (ver
/// `sync_comic` em `use-network-sync.svelte.ts`). A consulta é única por clique (não
/// re-dispara a cada tecla do campo de busca): o filtro por título é reativo no frontend sobre
/// o resultado já em cache, mesmo padrão do "Buscar quadrinhos por título" da Home.
export function useRemoteLibrary() {
	const libraries = new SvelteMap<string, ComicSummary[]>();
	const loading = new SvelteSet<string>();
	const errors = new SvelteMap<string, string>();
	const coverPaths = new SvelteMap<string, string>();
	const unlisten: UnlistenFn[] = [];

	// `addrs` do último `queryRemoteLibrary` por peer — reaproveitado pra disparar
	// `queryRemoteCover` em paralelo assim que a lista chega, sem o chamador precisar guardar
	// isso de novo. Versão já confirmada (cacheada ou "não mudou") por `"$peerId:$comicName"` —
	// nunca rebaixa a mesma versão duas vezes na mesma sessão do app.
	const addrsByPeer = new Map<string, number[]>();
	const knownCoverVersions = new Map<string, number>();

	async function startListening() {
		unlisten.push(
			await listen<string>(NETWORK_EVENTS.libraryQueryResult, (event) => {
				const payload = JSON.parse(event.payload) as LibraryQueryResultPayload;
				loading.delete(payload.peerId);
				errors.delete(payload.peerId);
				libraries.set(payload.peerId, payload.comics);
				void fetchCoversFor(payload.peerId, payload.comics);
			}),
			await listen<string>(NETWORK_EVENTS.libraryQueryError, (event) => {
				const { peerId, message } = parseErrorPayload(event.payload);
				if (!peerId) return;
				loading.delete(peerId);
				errors.set(peerId, message);
			}),
			await listen<string>(NETWORK_EVENTS.coverQueryResult, (event) => {
				const payload = JSON.parse(event.payload) as CoverQueryResultPayload;
				const key = coverKey(payload.peerId, payload.comicName);
				if (payload.coverVersion !== null) knownCoverVersions.set(key, payload.coverVersion);
				if (payload.status === 'changed' && payload.path) {
					const resolved = resolveArtworkPath(payload.path);
					if (resolved) coverPaths.set(key, resolved);
				}
			}),
			await listen<string>(NETWORK_EVENTS.coverQueryError, () => {
				// best-effort — a lista continua sem thumb pra esse item, sem bloquear o resto.
			})
		);
	}

	function stopListening() {
		unlisten.forEach((fn) => fn());
		unlisten.length = 0;
	}

	async function queryRemoteLibrary(peerId: string, addrs: number[]) {
		addrsByPeer.set(peerId, addrs);
		loading.add(peerId);
		errors.delete(peerId);
		try {
			await invoke(NETWORK_COMMANDS.queryRemoteLibrary, { peerId, addrs });
		} catch (err) {
			loading.delete(peerId);
			errors.set(peerId, String(err));
			throw err;
		}
	}

	/// Dispara `acerola/browse-cover/1` em paralelo pra cada quadrinho da lista — streams são
	/// baratas numa conexão já pooled por `(peer, alpn)`, sem necessidade de serializar.
	async function fetchCoversFor(peerId: string, comics: ComicSummary[]) {
		const addrs = addrsByPeer.get(peerId);
		if (!addrs) return;

		await Promise.all(
			comics.map((comic) => {
				const key = coverKey(peerId, comic.comicName);
				if (knownCoverVersions.get(key) === comic.coverVersion) return Promise.resolve();
				return invoke(NETWORK_COMMANDS.queryRemoteCover, {
					peerId,
					addrs,
					comicName: comic.comicName,
					knownVersion: knownCoverVersions.get(key) ?? null
				}).catch(() => {
					// best-effort, mesmo espírito de coverQueryError acima.
				});
			})
		);
	}

	function comicsFor(peerId: string): ComicSummary[] {
		return libraries.get(peerId) ?? [];
	}

	function isLoading(peerId: string): boolean {
		return loading.has(peerId);
	}

	function errorFor(peerId: string): string | undefined {
		return errors.get(peerId);
	}

	function coverPathFor(peerId: string, comicName: string): string | undefined {
		return coverPaths.get(coverKey(peerId, comicName));
	}

	return {
		startListening,
		stopListening,
		queryRemoteLibrary,
		comicsFor,
		isLoading,
		errorFor,
		coverPathFor
	};
}
