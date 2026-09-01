import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { debug } from '@tauri-apps/plugin-log';
import { toast } from 'svelte-sonner';
import { HOME_COMMANDS } from '$lib/contracts/home/home.commands';
import { HOME_EVENTS } from '$lib/contracts/home/home.events';
import { METADATA_COMMANDS } from '$lib/contracts/metadata/metadata.commands';
import type {
	ComicSummaryPayload,
	MetadataSource,
	SortBy,
	SortOrder
} from '$lib/contracts/home/home.payloads';
import type { ErrorPayload } from '$lib/contracts/shared/shared.payloads';
import { resolveErrorMessage } from '$lib/contracts/errors/errors.i18n';
import { notificationStore } from '$lib/components/acerola-notification/acerola-notification.svelte';
import { m } from '$lib/paraglide/messages';
import { error } from '@tauri-apps/plugin-log';

const { notify } = notificationStore;

let comics = $state<ComicSummaryPayload | undefined>(undefined);
let loading = $state(false);
let sortBy = $state<SortBy>('title');
let sortOrder = $state<SortOrder>('asc');
let showHidden = $state(false);
let metadataSource = $state<MetadataSource>('all');

let fetchQueued = false;
let lastSearch: string | undefined = undefined;

export function _resetComicSummaryState() {
	comics = undefined;
	loading = false;
	sortBy = 'title';
	sortOrder = 'asc';
	showHidden = false;
	metadataSource = 'all';
	fetchQueued = false;
	lastSearch = undefined;
}

export function useComicSummary() {
	async function fetch(search?: string): Promise<void> {
		if (loading) {
			fetchQueued = true;
			lastSearch = search;
			return;
		}
		loading = true;
		fetchQueued = false;

		return new Promise<void>(async (resolve) => {
			const unlisten = await listen<ComicSummaryPayload>(HOME_EVENTS.homeData, (event) => {
				comics = event.payload;

				debug(
					`[useComicSummary] total=${event.payload.total} fetchedAt=${event.payload.fetchedAt} payload=${JSON.stringify(event.payload.comics.slice(0, 3))}`
				);

				unlisten();
				unlistenErr();
				resolve();
			});

			const unlistenErr = await listen<ErrorPayload>(HOME_EVENTS.homeError, (event) => {
				const description = resolveErrorMessage(event.payload);

				notify.error(m['hooks.comic_summary.error.title'](), {
					description,
					duration: 0
				});

				toast.error(description);
				unlisten();
				unlistenErr();
				resolve();
			});

			await invoke(HOME_COMMANDS.getComicSummarySorted, {
				search,
				sortBy,
				sortOrder,
				showHidden,
				metadataSource: metadataSource === 'all' ? null : metadataSource
			});
		}).finally(() => {
			loading = false;
			if (fetchQueued) {
				fetch(lastSearch);
			}
		});
	}

	async function updateVisibility(ids: (string | number)[], hidden: boolean): Promise<number> {
		try {
			// IDs são hashes i64 que podem passar de Number.MAX_SAFE_INTEGER — mandar como
			// string preserva precisão (Number(id) arredondaria e atualizaria o quadrinho errado).
			const count = await invoke<number>(HOME_COMMANDS.updateComicsVisibility, {
				ids: ids.map(String),
				hidden
			});
			if (hidden && !showHidden && comics) {
				const idStrs = new Set(ids.map(String));
				comics = {
					...comics,
					comics: comics.comics.filter((c) => !idStrs.has(String(c.relations.directoryId))),
					total: comics.total - count
				};
			} else {
				await fetch(lastSearch);
			}
			return count;
		} catch (err) {
			error(`Failed to update comics visibility: ${err}`);
			throw err;
		}
	}

	async function deleteComics(ids: (string | number)[]): Promise<number> {
		try {
			// IDs são hashes i64 que podem passar de Number.MAX_SAFE_INTEGER — mandar como
			// string preserva precisão (Number(id) arredondaria e não bateria com nenhum
			// registro real, era exatamente o bug: "0 quadrinho(s) excluído(s)").
			const count = await invoke<number>(HOME_COMMANDS.deleteComics, {
				ids: ids.map(String)
			});
			// Remove deleted comics from the local state
			if (comics) {
				const idStrs = new Set(ids.map(String));
				comics = {
					...comics,
					comics: comics.comics.filter((c) => !idStrs.has(String(c.relations.directoryId))),
					total: comics.total - count
				};
			}
			return count;
		} catch (err) {
			error(`Failed to delete comics: ${err}`);
			throw err;
		}
	}

	async function clearMetadata(ids: (string | number)[]): Promise<number> {
		try {
			// IDs são hashes i64 que podem passar de Number.MAX_SAFE_INTEGER — mandar como
			// string preserva precisão (Number(id) arredondaria e apagaria o quadrinho errado).
			const count = await invoke<number>(METADATA_COMMANDS.clearComicsMetadataBatch, {
				comicIds: ids.map(String)
			});
			await fetch(lastSearch);
			return count;
		} catch (err) {
			error(`Failed to clear comics metadata: ${err}`);
			throw err;
		}
	}

	function setSorting(newSortBy: SortBy, newSortOrder: SortOrder): void {
		sortBy = newSortBy;
		sortOrder = newSortOrder;
	}

	function setFilters(newShowHidden: boolean, newMetadataSource: MetadataSource): void {
		showHidden = newShowHidden;
		metadataSource = newMetadataSource;
	}

	function hasActiveFilters(): boolean {
		return showHidden || metadataSource !== 'all';
	}

	return {
		fetch,
		updateVisibility,
		deleteComics,
		clearMetadata,
		setSorting,
		setFilters,
		hasActiveFilters,
		get comics() {
			return comics;
		},
		get loading() {
			return loading;
		},
		get sortBy() {
			return sortBy;
		},
		get sortOrder() {
			return sortOrder;
		},
		get showHidden() {
			return showHidden;
		},
		get metadataSource() {
			return metadataSource;
		}
	};
}
