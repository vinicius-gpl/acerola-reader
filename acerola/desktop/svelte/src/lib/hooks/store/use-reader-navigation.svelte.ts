import { browser } from '$app/environment';
import { replaceState } from '$app/navigation';
import { page } from '$app/state';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ReaderChapterPayload } from '$lib/contracts/reader/reader.payloads';
import { LIBRARY_EVENTS } from '$lib/contracts/library/chapter.events';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';
import { SESSION_KEYS } from '$lib/constants/session-keys';
import type { ChapterPayload } from '$lib/contracts/library/chapter.payloads';
import { onMount } from 'svelte';

export type ReaderNavigationState = {
	chapter?: ReaderChapterPayload;
	startPage?: number;
	chapterIndex?: number;
	totalChapters?: number;
	chapterScope?: string;
	comicDirectoryId?: string;
	sortBy?: 'number_asc' | 'number_desc' | 'modified_asc' | 'modified_desc';
};

export function useReaderNavigation() {
	let initializing = $state(false);

	// Mantém o rastro da navegação pendente para que o listener saiba o que fazer
	let pendingAction = $state<'load' | 'navigate' | null>(null);
	let pendingTargetIndex = $state<number | null>(null);

	const resolveInitialState = (): ReaderNavigationState => {
		const pageState = (page.state ?? {}) as ReaderNavigationState;

		if (!browser) return pageState;

		if (Object.keys(pageState).length > 0 && pageState.chapter) {
			sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(pageState));
			return pageState;
		}

		const saved = sessionStorage.getItem(SESSION_KEYS.readerState);
		if (saved) return JSON.parse(saved);

		return pageState;
	};

	let state = $state<ReaderNavigationState>(resolveInitialState());

	$effect(() => {
		const pageState = (page.state ?? {}) as ReaderNavigationState;

		if (Object.keys(pageState).length > 0 && pageState.chapter) {
			state = pageState;
			sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(pageState));
		}
	});

	const chapter = $derived(state.chapter);

	const chapterIndex = $derived.by(() => {
		const value = state.chapterIndex;
		// Stryker disable next-line ConditionalExpression: forçar `typeof value === 'number'` pra
		// `true` é equivalente — `Number.isFinite` nunca retorna `true` pra um argumento que não é
		// number (ele não faz coerção), então pra qualquer `value` não-number o operando da direita
		// já avalia pra `false` sozinho. Os dois branches são comportamentalmente idênticos pra
		// qualquer entrada possível.
		return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : undefined;
	});

	const totalChapters = $derived.by(() => {
		const value = state.totalChapters;
		// Stryker disable next-line ConditionalExpression: mesma equivalência do clamp de
		// `chapterIndex` acima — `Number.isFinite` nunca retorna `true` pra um argumento que
		// não é number, então forçar `typeof value === 'number'` pra `true` não muda o
		// resultado pra nenhuma entrada.
		return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : undefined;
	});

	const chaptersRemaining = $derived.by(() => {
		if (chapterIndex === undefined || totalChapters === undefined) return undefined;
		return Math.max(0, totalChapters - chapterIndex - 1);
	});

	// Stryker disable next-line ConditionalExpression: forçar `chaptersRemaining !== undefined` pra
	// `true` é equivalente — quando `chaptersRemaining` é realmente `undefined`,
	// `chaptersRemaining > 0` vira `undefined > 0`, que já é `false`, então os dois branches
	// concordam pra qualquer entrada.
	const hasNextChapter = $derived(chaptersRemaining !== undefined && chaptersRemaining > 0);
	// Stryker disable next-line ConditionalExpression: mesma equivalência de cima — `chapterIndex > 0`
	// já é `false` sempre que `chapterIndex` é `undefined`, então forçar o operando da esquerda pra
	// `true` não muda o resultado pra nenhuma entrada.
	const hasPreviousChapter = $derived(chapterIndex !== undefined && chapterIndex > 0);

	onMount(() => {
		let unlistenChapters: (() => void) | undefined;
		let unlistenError: (() => void) | undefined;

		const setupListeners = async () => {
			unlistenChapters = await listen<ChapterPayload>(LIBRARY_EVENTS.comicChapters, (event) => {
				const payload = event.payload;

				if (pendingAction === 'load') {
					const items = payload.archive.items;
					const idx = items.findIndex((item) => item.id === state.chapter?.id);

					if (idx !== -1) {
						state.chapterIndex = idx;
						state.totalChapters = payload.archive.total ?? items.length;
						sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(state));
					}

					pendingAction = null;
					return;
				}

				const isPendingNavigate = pendingAction === 'navigate';

				// `pendingTargetIndex` fica inline na condição (em vez de extraído pra uma
				// variável nomeada como `isPendingNavigate`) de propósito: o TypeScript só
				// estreita `pendingTargetIndex` de `number | null` pra `number` dentro deste
				// bloco através do `!== null` inline aqui — indiretar por um boolean separado
				// quebra esse narrowing e `chapterIndex: pendingTargetIndex` mais abaixo (que
				// espera `number | undefined`) para de compilar.
				//
				// Stryker disable next-line ConditionalExpression: `pendingTargetIndex` só é
				// setado pra um número não-nulo na mesma instrução que seta `pendingAction =
				// 'navigate'` (ver `navigateToRelativeChapter` abaixo), e todo caminho de
				// conclusão reseta os dois juntos. Então sempre que `isPendingNavigate` é true,
				// `pendingTargetIndex !== null` já é garantidamente true também — não existe
				// estado alcançável onde `isPendingNavigate` é true e o valor real dessa
				// checagem é false, então forçá-la pra `true` nunca muda o resultado.
				if (isPendingNavigate && pendingTargetIndex !== null) {
					const nextChapterData = payload.archive.items[0];

					if (!nextChapterData) {
						initializing = false;
						pendingAction = null;
						pendingTargetIndex = null;
						return;
					}

					const readerChapter: ReaderChapterPayload = {
						id: nextChapterData.id,
						name: nextChapterData.name,
						path: nextChapterData.path,
						chapterSort: nextChapterData.chapterSort ?? '',
						volumeId: nextChapterData.volumeId ?? null,
						volumeName: nextChapterData.volumeName ?? null,
						isSpecial: nextChapterData.isSpecial ?? false,
						lastModified: nextChapterData.lastModified ?? 0
					};

					const newState: ReaderNavigationState = {
						chapter: readerChapter,
						comicDirectoryId: state.comicDirectoryId,
						chapterIndex: pendingTargetIndex,
						totalChapters: state.totalChapters,
						chapterScope: state.chapterScope,
						sortBy: state.sortBy
					};

					state = newState;
					replaceState(page.url, newState);

					initializing = false;
					pendingAction = null;
					pendingTargetIndex = null;
				}
			});

			unlistenError = await listen<any>(LIBRARY_EVENTS.comicChaptersError, () => {
				initializing = false;
				pendingAction = null;
				pendingTargetIndex = null;
			});
		};

		void setupListeners();

		return () => {
			unlistenChapters?.();
			unlistenError?.();
		};
	});

	async function loadContext() {
		if (!chapter || !state.comicDirectoryId) return;

		pendingAction = 'load';

		const sortBy = state.sortBy ?? 'number_asc';

		try {
			await invoke(LIBRARY_COMMANDS.getComicChapters, {
				comicDirectoryFk: state.comicDirectoryId,
				volumeId: null,
				page: 0,
				pageSize: 99999,
				sortBy: sortBy,
				searchQuery: null
			});
		} catch {
			pendingAction = null;
		}
	}

	async function goToNextChapter() {
		// Stryker disable next-line ConditionalExpression: `hasNextChapter` é `$derived` de
		// `chaptersRemaining`, que só é definido quando `chapterIndex` também é — então
		// `hasNextChapter === true` já garante `chapterIndex !== undefined`. Não existe estado
		// alcançável onde `hasNextChapter` é true e `chapterIndex` é undefined, então forçar essa
		// checagem pra `false` nunca muda o resultado.
		const chapterIndexIsMissing = chapterIndex === undefined;

		if (!hasNextChapter || chapterIndexIsMissing || !state.comicDirectoryId) return;
		await navigateToRelativeChapter(chapterIndex + 1);
	}

	async function goToPreviousChapter() {
		// Stryker disable next-line ConditionalExpression: `hasPreviousChapter` é `$derived` de
		// `chapterIndex !== undefined && chapterIndex > 0`, então `hasPreviousChapter === true` já
		// garante `chapterIndex !== undefined`. Não existe estado alcançável onde `hasPreviousChapter`
		// é true e `chapterIndex` é undefined, então forçar essa checagem pra `false` nunca muda o
		// resultado.
		const chapterIndexIsMissing = chapterIndex === undefined;

		if (!hasPreviousChapter || chapterIndexIsMissing || !state.comicDirectoryId) return;
		await navigateToRelativeChapter(chapterIndex - 1);
	}

	async function navigateToRelativeChapter(targetIndex: number) {
		initializing = true;
		pendingAction = 'navigate';
		pendingTargetIndex = targetIndex;

		const sortBy = state.sortBy ?? 'number_asc';

		try {
			await invoke(LIBRARY_COMMANDS.getComicChapters, {
				comicDirectoryFk: state.comicDirectoryId,
				volumeId: null,
				page: targetIndex,
				pageSize: 1,
				sortBy: sortBy,
				searchQuery: null
			});
		} catch {
			initializing = false;
			pendingAction = null;
			pendingTargetIndex = null;
		}
	}

	return {
		get state() {
			return state;
		},
		get initializing() {
			return initializing;
		},
		set initializing(value) {
			initializing = value;
		},
		get chapter() {
			return chapter;
		},
		get chapterIndex() {
			return chapterIndex;
		},
		get totalChapters() {
			return totalChapters;
		},
		get chaptersRemaining() {
			return chaptersRemaining;
		},
		get hasNextChapter() {
			return hasNextChapter;
		},
		get hasPreviousChapter() {
			return hasPreviousChapter;
		},
		loadContext,
		goToNextChapter,
		goToPreviousChapter
	};
}
