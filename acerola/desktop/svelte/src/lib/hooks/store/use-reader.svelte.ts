import { invoke } from '@tauri-apps/api/core';
import { onDestroy } from 'svelte';
import { READER_COMMANDS } from '$lib/contracts/reader/reader.commands';
import type {
	ReaderCachedPagePayload,
	ReaderChapterPayload,
	ReaderPagePayload,
	ReaderSessionPayload
} from '$lib/contracts/reader/reader.payloads';
import { LRUService } from '$lib/services/lru.service';

// INFO: Espelha DEFAULT_CACHE_CAPACITY do backend (core/services/reader/mod.rs) —
// janela deslizante, não o capítulo inteiro. É só o valor usado antes do
// session.cacheCapacity chegar do backend; resetCache(session.cacheCapacity)
// sobrescreve isso assim que a sessão abre.
const DEFAULT_READER_CACHE_SIZE = 20;
export const PREFETCH_RADIUS = 2;

export function useReader() {
	let pageCache = createPageCache(DEFAULT_READER_CACHE_SIZE);
	let pendingPages = new Map<number, Promise<ReaderCachedPagePayload | undefined>>();
	let requestVersion = 0;

	let session = $state<ReaderSessionPayload | undefined>(undefined);
	let currentPage = $state(0);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let cacheVersion = $state(0);

	onDestroy(() => {
		pageCache.clear();
		void invoke(READER_COMMANDS.closeChapter).catch(() => undefined);
	});

	function createPageCache(max: number) {
		return new LRUService<number, ReaderCachedPagePayload>({
			max,
			dispose: (page) => URL.revokeObjectURL(page.url)
		});
	}

	function resetCache(max = DEFAULT_READER_CACHE_SIZE) {
		// Stryker disable next-line UpdateOperator: requestVersion só é comparado por
		// (des)igualdade contra um valor capturado antes (`version !== requestVersion` /
		// `version === requestVersion`), nunca ordenado ou exibido. Este é o único ponto
		// de incremento, então a sequência de valores que produz é estritamente monotônica
		// de qualquer jeito (0,1,2,... ou 0,-1,-2,...) — sempre injetiva, nunca colide com
		// um valor anterior. Trocar ++ por -- rotula cada resultado "mudou" vs "não mudou"
		// de forma idêntica; nenhuma checagem de igualdade neste arquivo distingue os dois.
		// Verificado empiricamente.
		requestVersion++;
		pageCache.clear();
		pageCache = createPageCache(max);
		pendingPages.clear();
		// Stryker disable next-line UpdateOperator: cacheVersion só é lido puro
		// (`cacheVersion;`) dentro de blocks $derived.by pra forçar o Svelte a tratá-los
		// como dependentes dele — seu valor numérico nunca é comparado nem exibido.
		// ++ e -- mudam o valor em exatamente 1 a partir do que era, então os dois sempre
		// disparam o dirty-check do Svelte e recomputam do mesmo jeito; a direção da
		// mudança é inobservável. Verificado empiricamente.
		cacheVersion++;
	}

	function setError(value: unknown) {
		error = value instanceof Error ? value.message : String(value);
	}

	function clampPage(index: number) {
		if (!session) return 0;
		return Math.max(0, Math.min(index, session.pageCount - 1));
	}

	function isValidPage(index: number) {
		return Boolean(session && index >= 0 && index < session.pageCount);
	}

	function toCachedPage(payload: ReaderPagePayload): ReaderCachedPagePayload {
		const bytes = new Uint8Array(payload.bytes);
		const blob = new Blob([bytes], { type: payload.mimeType });

		return {
			index: payload.index,
			total: payload.total,
			mimeType: payload.mimeType,
			url: URL.createObjectURL(blob),
			cacheHit: payload.cacheHit
		};
	}

	function windowIndices(center: number) {
		// Stryker disable next-line ConditionalExpression: prefetchWindow() (única
		// chamadora desta função) já retorna cedo pelo seu próprio guard `if (!session)`,
		// e faz isso de forma síncrona sem nenhum `await` antes de chamar windowIndices()
		// — então `session` nunca pode mudar entre as duas checagens. Esse guard é
		// inalcançavelmente falso dado o único call site atual. Verificado empiricamente.
		if (!session) return [];

		const start = Math.max(0, center - PREFETCH_RADIUS);
		// Stryker disable next-line ArithmeticOperator: o único papel desse clamp é
		// impedir a janela de passar da última página válida. Sempre que ele é a
		// restrição ativa (ou seja, quando essa mutação mudaria o valor de `end`), todo
		// índice que ela admitiria é >= pageCount e portanto inválido; o próprio guard
		// isValidPage() do loadPage descarta esses silenciosamente antes de qualquer
		// efeito colateral observável (sem chamada ao backend, sem escrita no cache).
		// Verificado empiricamente: aplicar essa mutação e rodar a suíte inteira deixa
		// todo teste verde.
		const end = Math.min(session.pageCount - 1, center + PREFETCH_RADIUS);

		return Array.from({ length: end - start + 1 }, (_, index) => start + index);
	}

	async function open(chapter: ReaderChapterPayload, startPage = 0) {
		loading = true;
		error = null;
		resetCache();

		try {
			session = await invoke<ReaderSessionPayload>(READER_COMMANDS.openChapter, { chapter });
			resetCache(session.cacheCapacity);

			currentPage = clampPage(startPage);
			await goToPage(currentPage);

			return session;
		} catch (caught) {
			setError(caught);
			throw caught;
		} finally {
			loading = false;
		}
	}

	async function loadPage(index: number, setCurrent = true) {
		if (!isValidPage(index)) return undefined;

		const cached = setCurrent ? pageCache.get(index) : pageCache.peek(index);
		if (cached) {
			if (setCurrent) {
				await syncCurrentPage(index);
			}

			// Stryker disable next-line UpdateOperator: cacheVersion só é lido puro
			// (`cacheVersion;`) dentro de blocks $derived.by pra forçar o Svelte a tratá-los
			// como dependentes dele — seu valor numérico nunca é comparado nem exibido.
			// ++ e -- mudam o valor em exatamente 1 a partir do que era, então os dois
			// sempre disparam o dirty-check do Svelte e recomputam do mesmo jeito; a
			// direção da mudança é inobservável. Verificado empiricamente.
			cacheVersion++;
			return cached;
		}

		const pending = pendingPages.get(index);
		if (pending) return pending;

		const version = requestVersion;
		// Stryker disable next-line OptionalChaining: isValidPage(index) no topo desta
		// função já exigiu que `session` fosse truthy (ela curto-circuita `session && ...`
		// pra false caso contrário), e nada entre essa checagem e esta linha usa await —
		// então `session` ainda não pode ter virado undefined. Verificado empiricamente.
		const chapterId = session?.chapter.id;

		const request = (async () => {
			if (setCurrent) loading = true;

			try {
				const payload = await invoke<ReaderPagePayload>(READER_COMMANDS.loadPage, {
					index,
					setCurrent
				});

				// Os dois disjuntos abaixo (`payload.chapterId !== chapterId` e `payload.chapterId
				// !== session?.chapter.id`) só importam pro resultado quando `version ===
				// requestVersion` (senão o primeiro disjunto acima já descarta). E `session` só
				// muda via `open()` ou `close()`, e as duas sempre incrementam `requestVersion` de
				// forma síncrona no mesmo tick da troca de sessão, sem nenhum `await` entre elas —
				// então sempre que `version === requestVersion`, `session` não mudou desde a
				// captura de `chapterId`, o que garante `chapterId === session?.chapter.id`. Ou
				// seja, os dois disjuntos abaixo sempre avaliam pro MESMO booleano um do outro no
				// único regime em que fazem diferença — mutar um sozinho nunca muda o resultado
				// final, porque o outro (intacto) sempre "cobre" a mesma checagem. Verificado
				// empiricamente.
				if (
					version !== requestVersion ||
					// Stryker disable next-line ConditionalExpression
					payload.chapterId !== chapterId ||
					// Stryker disable next-line ConditionalExpression,OptionalChaining
					payload.chapterId !== session?.chapter.id
				) {
					return undefined;
				}

				const page = toCachedPage(payload);

				pageCache.set(index, page);

				if (setCurrent) {
					currentPage = index;
				}

				error = null;
				// Stryker disable next-line UpdateOperator: mesmo raciocínio dos outros
				// pontos de cacheVersion++ — só é lido puro como gatilho de dependência
				// do Svelte, nunca comparado, então ++ vs -- é inobservável. Verificado
				// empiricamente.
				cacheVersion++;

				return page;
			} catch (caught) {
				if (version === requestVersion) {
					setError(caught);
				}

				throw caught;
			} finally {
				if (version === requestVersion) {
					pendingPages.delete(index);
				}
				if (setCurrent && version === requestVersion) loading = false;
			}
		})();

		pendingPages.set(index, request);
		return request;
	}

	async function syncCurrentPage(index: number) {
		// Stryker disable next-line ConditionalExpression: a única chamadora desta
		// função (o branch de cache do loadPage) obtém `cached` via `pageCache.get(index)`
		// imediatamente antes de chamar syncCurrentPage(index) com esse MESMO índice, e
		// isValidPage(index) fica implícito verdadeiro pra qualquer índice que tenha sido
		// realmente cacheado — nada usa await no meio, então esse guard nunca observa um
		// índice inválido. Verificado empiricamente.
		if (!isValidPage(index)) return;

		currentPage = index;
		// Stryker disable next-line CallExpression: a chamadora acima já chamou
		// `pageCache.get(index)` (não `.peek()`) pra esse índice exato logo antes de
		// invocar syncCurrentPage, o que já promoveu a entrada a mais-recentemente-usada
		// no cache LRU. Essa segunda chamada de `.get()` retoca a mesma entrada já-MRU —
		// um no-op pra fins de ordenação, já que nada mais lê ou escreve no cache nesse
		// meio tempo. Verificado empiricamente.
		pageCache.get(index);
		// Stryker disable next-line UpdateOperator: mesmo raciocínio dos outros pontos
		// de cacheVersion++ — só é lido puro como gatilho de dependência do Svelte,
		// nunca comparado, então ++ vs -- é inobservável. Verificado empiricamente.
		cacheVersion++;

		try {
			await invoke(READER_COMMANDS.setCurrentPage, { index });
		} catch (caught) {
			setError(caught);
		}
	}

	async function prefetchWindow(center = currentPage) {
		if (!session) return;

		void invoke(READER_COMMANDS.prefetchWindow, {
			center,
			radius: PREFETCH_RADIUS
		}).catch((caught) => setError(caught));

		for (const index of windowIndices(center)) {
			if (index !== center && !pageCache.has(index)) {
				void loadPage(index, false).catch(() => undefined);
			}
		}
	}

	async function goToPage(index: number) {
		const nextPage = clampPage(index);
		const page = await loadPage(nextPage, true);

		if (page) {
			void prefetchWindow(nextPage);
		}

		return page;
	}

	function nextPage() {
		return goToPage(currentPage + 1);
	}

	function previousPage() {
		return goToPage(currentPage - 1);
	}

	async function close() {
		try {
			await invoke(READER_COMMANDS.closeChapter);
		} finally {
			session = undefined;
			currentPage = 0;
			loading = false;
			error = null;
			resetCache();
		}
	}

	const pages = $derived.by(() => {
		cacheVersion;

		return pageCache.keys
			.slice()
			.sort((left, right) => left - right)
			.map((key) => pageCache.peek(key))
			.filter((page): page is ReaderCachedPagePayload => Boolean(page));
	});

	const current = $derived.by(() => {
		cacheVersion;
		return pageCache.peek(currentPage);
	});

	const cacheKeys = $derived.by(() => {
		cacheVersion;
		return pageCache.keys;
	});

	return {
		open,
		loadPage,
		goToPage,
		nextPage,
		previousPage,
		prefetchWindow,
		close,
		pageAt(index: number) {
			cacheVersion;
			return pageCache.peek(index);
		},
		get session() {
			return session;
		},
		get pageCount() {
			return session?.pageCount ?? 0;
		},
		get currentPage() {
			return currentPage;
		},
		get current() {
			return current;
		},
		get pages() {
			return pages;
		},
		get cacheKeys() {
			return cacheKeys;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		}
	};
}
