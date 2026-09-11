import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'svelte-sonner';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';
import { LIBRARY_EVENTS } from '$lib/contracts/library/chapter.events';
import type {
	ChapterPayload,
	ChapterFilePayload,
	ChapterPagePayload
} from '$lib/contracts/library/chapter.payloads';
import type { ErrorPayload } from '$lib/contracts/shared/shared.payloads';
import { resolveErrorMessage } from '$lib/contracts/errors/errors.i18n';
import { notificationStore } from '$lib/components/acerola-notification/acerola-notification.svelte';
import { LRUService } from '$lib/services/lru.service';
import { onMount } from 'svelte';
import { m } from '$lib/paraglide/messages';

const { notify } = notificationStore;

// INFO: A lista de capítulos é buscada inteira de uma vez — nem RAM (metadados
// são leves) nem DOM (a renderização é virtualizada por posição de scroll)
// dependem mais de paginação. Um comic com milhares de capítulos ainda fica
// muito abaixo desse teto; ele só existe pra dar um limite explícito à query.
const FETCH_ALL_PAGE_SIZE = 1_000_000;

// INFO: Espelha o cache do backend (ChapterCacheService, chapter_cache.rs) —
// mesma chave (comic + volume + ordenação + busca), mesma capacidade. Reabrir
// um volume/ordenação já visto nesta sessão não depende nem de round-trip de
// IPC: aplica na hora, sem o período em branco que causava o "clica duas
// vezes" — bem diferente do LRU antigo daqui, que cacheava blocos de
// paginação de uma única busca (removido junto com a paginação client-side).
const CHAPTER_CACHE_CAPACITY = 32;

type SortBy = 'number_asc' | 'number_desc' | 'modified_asc' | 'modified_desc';

function buildCacheKey(
	comicDirectoryId: string,
	sortBy: SortBy,
	volumeId: string | null,
	searchQuery: string | null
) {
	return `${comicDirectoryId}|${volumeId ?? ''}|${sortBy}|${searchQuery ?? ''}`;
}

export function useComicChapters() {
	const chapterCache = new LRUService<string, ChapterPayload>({ max: CHAPTER_CACHE_CAPACITY });

	let items = $state<ChapterFilePayload[] | undefined>(undefined);
	let loading = $state(false);
	let failed = false;
	let currentKey: string | null = null;

	let metadata = $state<
		(Omit<ChapterPayload, 'archive'> & { archive: Omit<ChapterPagePayload, 'items'> }) | undefined
	>(undefined);

	function applyPayload(payload: ChapterPayload) {
		items = payload.archive.items;
		failed = false;

		metadata = {
			showVolumeHeaders: payload.showVolumeHeaders,
			hasVolumeStructure: payload.hasVolumeStructure,
			effectiveViewMode: payload.effectiveViewMode,
			archive: {
				page: payload.archive.page,
				pageSize: payload.archive.pageSize,
				total: payload.archive.total,
				volumes: payload.archive.volumes,
				volumeSections: payload.archive.volumeSections
			}
		};

		loading = false;
	}

	function clear(keepMetadata = false) {
		items = undefined;
		failed = false;
		if (!keepMetadata) {
			metadata = undefined;
		}
		loading = false;
	}

	// INFO: `clear()` sozinho não basta pra forçar um dado fresco — o próximo `fetch()`
	// pra essa mesma chave (comic + volume + ordenação + busca) ainda acha o payload
	// antigo no `chapterCache` e aplica ele sem round-trip nenhum. Usado depois de uma
	// mutação real (rescan leve/completo), onde o conteúdo por trás da mesma chave mudou —
	// diferente de só trocar de volume/ordenação, onde reaproveitar o cache é o
	// comportamento certo.
	function invalidate() {
		chapterCache.clear();
	}

	onMount(() => {
		let unlistenChapters: (() => void) | undefined;
		let unlistenError: (() => void) | undefined;

		const setupListeners = async () => {
			unlistenChapters = await listen<ChapterPayload>(LIBRARY_EVENTS.comicChapters, (event) => {
				const payload = event.payload;

				applyPayload(payload);

				if (currentKey) {
					chapterCache.set(currentKey, payload);
				}
			});

			unlistenError = await listen<ErrorPayload>(LIBRARY_EVENTS.comicChaptersError, (event) => {
				const errorMessage = resolveErrorMessage(event.payload);

				notify.error(m['hooks.comic_chapters.error.load'](), {
					description: errorMessage
				});
				toast.error(errorMessage);

				loading = false;
			});
		};

		setupListeners();

		return () => {
			unlistenChapters?.();
			unlistenError?.();
		};
	});

	async function fetch(
		comicDirectoryId: string,
		sortBy: SortBy,
		volumeId: string | null = null,
		searchQuery: string | null = null
	) {
		if (failed) return;
		if (items !== undefined) return;
		if (loading) return;

		const key = buildCacheKey(comicDirectoryId, sortBy, volumeId, searchQuery);
		const cached = chapterCache.get(key);

		if (cached) {
			applyPayload(cached);
			return;
		}

		loading = true;
		currentKey = key;

		try {
			await invoke(LIBRARY_COMMANDS.getComicChapters, {
				comicDirectoryFk: comicDirectoryId,
				volumeId,
				page: 0,
				pageSize: FETCH_ALL_PAGE_SIZE,
				sortBy,
				searchQuery: searchQuery || null
			});
		} catch (error) {
			const errorMessage = error as string;

			notify.error(m['hooks.comic_chapters.error.request'](), {
				description: errorMessage
			});
			toast.error(errorMessage);

			failed = true;
			loading = false;
		}
	}

	const chapters = $derived.by(() => {
		if (!metadata) return undefined;

		// INFO: `items` fica undefined enquanto uma nova busca está em voo
		// (clear(true) sempre limpa items, mesmo preservando metadata) — cair
		// pra [] aqui em vez de retornar undefined evita que archive.volumes
		// suma nesse meio-tempo. Sem isso, expandir um volume fazia a lista de
		// volumes inteira desmontar e remontar a cada clique (nada estava
		// realmente quebrado, só piscava e engolia o primeiro clique).
		return {
			...metadata,
			archive: {
				...metadata.archive,
				items: items ?? []
			}
		};
	});

	return {
		fetch,
		clear,
		invalidate,
		get chapters() {
			return chapters;
		},
		get loading() {
			return loading;
		}
	};
}
