import { invoke } from '@tauri-apps/api/core';
import { error } from '@tauri-apps/plugin-log';
import { METADATA_COMMANDS } from '$lib/contracts/metadata/metadata.commands';
import type { ComicMetadataEvent } from '$lib/contracts/metadata/metadata.payloads';
import { load } from '@tauri-apps/plugin-store';
import { STORE_FILE, STORE_KEYS } from '$lib/constants/store-plugin';
import { isSyncingAll } from '$lib/state/metadata-sync-all.svelte';

export function useMetadataSync() {
	let isSyncing = $state(false);

	async function syncMangadex(title: string, comicId: string): Promise<ComicMetadataEvent | null> {
		isSyncing = true;
		try {
			const store = await load(STORE_FILE);
			const language = (await store.get<string>(STORE_KEYS.metadataLanguage)) || 'pt-br';
			const generateComicInfo = (await store.get<boolean>(STORE_KEYS.comicInfoPreference)) ?? false;
			const result = await invoke<ComicMetadataEvent>(METADATA_COMMANDS.syncMangadex, {
				title,
				comicId,
				language,
				generateComicInfo
			});
			return result;
		} catch (err) {
			error(`Failed to sync MangaDex: ${JSON.stringify(err)}`);
			throw err;
		} finally {
			isSyncing = false;
		}
	}

	async function syncAnilist(title: string, comicId: string): Promise<ComicMetadataEvent | null> {
		isSyncing = true;
		try {
			const store = await load(STORE_FILE);
			const language = (await store.get<string>(STORE_KEYS.metadataLanguage)) || 'pt-br';
			const generateComicInfo = (await store.get<boolean>(STORE_KEYS.comicInfoPreference)) ?? false;
			const result = await invoke<ComicMetadataEvent>(METADATA_COMMANDS.syncAnilist, {
				title,
				comicId,
				language,
				generateComicInfo
			});
			return result;
		} catch (err) {
			error(`Failed to sync AniList: ${JSON.stringify(err)}`);
			throw err;
		} finally {
			isSyncing = false;
		}
	}

	async function syncComicInfo(comicId: string): Promise<ComicMetadataEvent | null> {
		isSyncing = true;
		try {
			const result = await invoke<ComicMetadataEvent>(METADATA_COMMANDS.syncComicInfo, {
				comicId
			});
			return result;
		} catch (err) {
			error(`Failed to sync ComicInfo.xml: ${JSON.stringify(err)}`);
			throw err;
		} finally {
			isSyncing = false;
		}
	}

	async function clearMetadata(comicId: string): Promise<void> {
		isSyncing = true;
		try {
			await invoke(METADATA_COMMANDS.clearComicMetadata, { comicId });
		} catch (err) {
			error(`Failed to clear comic metadata: ${JSON.stringify(err)}`);
			throw err;
		} finally {
			isSyncing = false;
		}
	}

	return {
		get isSyncing() {
			// Também desabilita enquanto um sync em lote (tela de Config) estiver rodando —
			// o lock de verdade é no backend (`SyncGuard`), isso só evita o usuário clicar e
			// levar um toast de `SyncInProgress` por um comando que já sabíamos que ia falhar.
			return isSyncing || isSyncingAll();
		},
		syncMangadex,
		syncAnilist,
		syncComicInfo,
		clearMetadata
	};
}
