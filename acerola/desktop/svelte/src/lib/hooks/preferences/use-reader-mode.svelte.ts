import { STORE_FILE, STORE_KEYS } from '$lib/constants/store-plugin';
import { load } from '@tauri-apps/plugin-store';
import type { ReaderMode } from '../../../routes/reader/hooks/use-reader-zoom.svelte';

export function useReaderMode() {
	let readerMode = $state<ReaderMode>('vertical');

	async function loadReaderMode() {
		const store = await load(STORE_FILE);
		readerMode = (await store.get<ReaderMode>(STORE_KEYS.readerMode)) ?? 'vertical';
	}

	async function saveReaderMode(value: ReaderMode) {
		const store = await load(STORE_FILE);
		readerMode = value;

		await store.set(STORE_KEYS.readerMode, value);
		await store.save();
	}

	return {
		loadReaderMode,
		saveReaderMode,

		get readerMode() {
			return readerMode;
		},

		set readerMode(value: ReaderMode) {
			readerMode = value;
		}
	};
}
