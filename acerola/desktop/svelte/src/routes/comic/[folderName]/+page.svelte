<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaToggleGroup from '$lib/components/acerola-toggle-group/acerola-toggle-group.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';

	import { ToggleGroupItem } from '$lib/components/ui/toggle-group/index.js';

	import { useVolumeViewMode } from '$lib/hooks/preferences/use-volume-view-mode.svelte';
	import { useComicChapters } from '$lib/hooks/store/use-comic-chapters.svelte';
	import { useBookmarks } from '$lib/hooks/store/use-bookmarks.svelte';
	import { useChapterSelection } from '$lib/hooks/store/use-chapter-selection.svelte';
	import { useHistory } from '$lib/hooks/store/use-history.svelte';
	import { usePeerConnection } from '$lib/hooks/store/use-peer-connection.svelte';
	import { useNetworkSync, type SyncDirection } from '$lib/hooks/store/use-network-sync.svelte';

	import { useComicContext } from '$lib/state/comic-context.svelte';
	import { useMetadataSync } from '$lib/hooks/store/use-metadata-sync.svelte';

	import { resolveArtworkPath, resolveBanner, resolveCover } from '$lib/utils/artwork.utils';
	import type { ReaderChapterPayload } from '$lib/contracts/reader/reader.payloads';
	import { invoke } from '@tauri-apps/api/core';
	import { HOME_COMMANDS } from '$lib/contracts/home/home.commands';
	import { HISTORY_COMMANDS } from '$lib/contracts/history/history.commands';
	import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';

	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import ArrowUpDown from '@lucide/svelte/icons/arrow-up-down';
	import Check from '@lucide/svelte/icons/check';

	import { onMount, untrack } from 'svelte';
	import { fade } from 'svelte/transition';
	import { m } from '$lib/paraglide/messages';
	import { extractErrorMessage } from '$lib/utils/error.utils';
	import { toastAsync } from '$lib/utils/toast-async.utils';
	import { slidingIndicator } from '$lib/utils/sliding-indicator.utils';

	import ComicChapterList, { type Chapter } from './components/acerola-comic-chapter-list.svelte';
	import ComicHeroBanner from './components/acerola-comic-hero-banner.svelte';
	import ComicMetadataPanel from './components/acerola-comic-metadata-panel.svelte';
	import ComicPreferences from './components/acerola-comic-preferences.svelte';
	import ComicVolumeList, {
		type VolumeChapter
	} from './components/acerola-comic-volume-list.svelte';

	let { data } = $props();

	const activeComic = useComicContext();
	const chapterStore = useComicChapters();

	// INFO: Tamanho do bloco só pra virtualização de renderização — não tem
	// mais relação com quantos capítulos são buscados do backend (isso agora
	// vem tudo de uma vez, ver use-comic-chapters.svelte.ts).
	const RENDER_CHUNK_SIZE = 25;

	const volumeViewPreference = useVolumeViewMode();
	const bookmarkStore = useBookmarks();
	const metadataSync = useMetadataSync();
	const chapterSelection = useChapterSelection();
	const historyActions = useHistory();
	const peers = usePeerConnection();
	const p2pSync = useNetworkSync();

	let expandedVolumeId = $state<string | null>(null);
	let currentBookmarkId = $state<number | null>(null);

	let activeTab = $state('content');
	let sortBy = $state<'number_asc' | 'number_desc' | 'modified_asc' | 'modified_desc'>(
		'number_asc'
	);
	let searchQuery = $state('');
	let showSortMenu = $state(false);

	// Sobrescrever cover.* mantém o mesmo path no disco, então o back-end não muda a URL
	// resolvida — sem isso o <img> nunca re-renderiza e o cache do protocolo asset:// serve
	// os bytes antigos. Bumped localmente logo após um regenerate bem-sucedido nesta página.
	let coverCacheBust = $state(0);
	let volumeCoverCacheBust = $state(0);
	// Bumped pelo efeito de sync P2P abaixo — lido (não escrito) pelo efeito de busca de
	// capítulos, só pra forçar um re-fetch quando um sync relevante a este quadrinho termina.
	let syncRefreshTrigger = $state(0);

	function bustCache(url: string | null, bust: number): string | null {
		if (!url || !bust) return url;
		return url + (url.includes('?') ? '&' : '?') + 'v=' + bust;
	}

	const onBack = () => window.history.back();

	function toReaderChapter(chapter: Chapter | VolumeChapter): ReaderChapterPayload | null {
		if (!chapter.path) return null;

		return {
			id: chapter.id,
			name: chapter.name ?? chapter.title,
			path: chapter.path,
			chapterSort: chapter.chapterSort ?? '',
			volumeId: chapter.volumeId ?? null,
			volumeName: chapter.volumeName ?? null,
			isSpecial: chapter.isSpecial ?? false,
			lastModified: chapter.lastModified ?? 0
		};
	}

	function getReaderProgress(chapter: Chapter | VolumeChapter) {
		const currentManga = manga;
		if (!currentManga) return {};

		const volume = chapter.volumeId
			? currentManga.volumes.find((item) => item.id === chapter.volumeId)
			: null;

		return {
			chapterIndex: chapter.chapterIndex,
			totalChapters: volume?.totalChapters ?? currentManga.chaptersCount,
			chapterScope: volume?.title ?? currentManga.title,
			sortBy: sortBy
		};
	}

	function openReader(chapter: Chapter | VolumeChapter) {
		const readerChapter = toReaderChapter(chapter);
		if (!readerChapter) return;

		goto('/reader', {
			state: {
				chapter: readerChapter,
				comicDirectoryId:
					activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId,
				...getReaderProgress(chapter)
			}
		});
	}

	let readingHistory = $state<any | null>(null);
	let readChapters = $state<string[]>([]);
	let isHistoryLoading = $state(false);
	let isSyncing = $state(false);

	async function handleMarkRead(chapter: Chapter | VolumeChapter) {
		const id = comicId;
		if (!id) return;

		await historyActions.markChapterRead(id.toString(), chapter.id);
		if (!readChapters.includes(chapter.id)) {
			readChapters = [...readChapters, chapter.id];
		}
	}

	async function handleMarkUnread(chapter: Chapter | VolumeChapter) {
		const id = comicId;
		if (!id) return;

		await historyActions.unmarkChapterRead(id.toString(), chapter.id);
		readChapters = readChapters.filter((readId) => readId !== chapter.id);
	}

	function handleEnterSelection(chapter: Chapter | VolumeChapter) {
		chapterSelection.selectSingle(chapter.id);
	}

	function handleToggleSelect(chapter: Chapter | VolumeChapter) {
		chapterSelection.toggleSelection(chapter.id);
	}

	async function handleSelectAllChapters() {
		const id = comicId;
		if (!id) return;

		const ids = await invoke<string[]>(LIBRARY_COMMANDS.getComicChapterIds, {
			comicDirectoryFk: id.toString()
		});
		chapterSelection.selectAll(ids);
	}

	async function handleBatchMarkRead() {
		const id = comicId;
		if (!id) return;

		const ids = chapterSelection.selectedIdsArray;
		await historyActions.markChaptersReadBatch(id.toString(), ids);
		const newlyRead = ids.filter((chapterId) => !readChapters.includes(chapterId));
		readChapters = [...readChapters, ...newlyRead];
		chapterSelection.exitSelectionMode();
	}

	async function handleBatchMarkUnread() {
		const id = comicId;
		if (!id) return;

		const ids = chapterSelection.selectedIdsArray;
		await historyActions.unmarkChaptersReadBatch(id.toString(), ids);
		readChapters = readChapters.filter((readId) => !ids.includes(readId));
		chapterSelection.exitSelectionMode();
	}

	async function handleSyncMangadex() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id || !manga?.title) return;
		try {
			await toastAsync(() => metadataSync.syncMangadex(manga!.title, id.toString()), {
				loading: m['pages.comic.toast.sync.start_mangadex'](),
				success: m['pages.comic.toast.sync.success'](),
				error: (err) => m['pages.comic.toast.mangadex_error']({ msg: extractErrorMessage(err) })
			});
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleSyncAnilist() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id || !manga?.title) return;
		try {
			await toastAsync(() => metadataSync.syncAnilist(manga!.title, id.toString()), {
				loading: m['pages.comic.toast.sync.start_anilist'](),
				success: m['pages.comic.toast.sync.success'](),
				error: (err) => m['pages.comic.toast.anilist_error']({ msg: extractErrorMessage(err) })
			});
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleSyncComicInfo() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(() => metadataSync.syncComicInfo(id.toString()), {
				loading: m['pages.comic.toast.sync.start_comic_info'](),
				success: m['pages.comic.toast.sync.success'](),
				error: (err) => m['pages.comic.toast.comic_info.error']({ msg: extractErrorMessage(err) })
			});
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleRescanComic() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;

		try {
			await toastAsync(() => invoke(HOME_COMMANDS.rescanComic, { id: id.toString() }), {
				loading: m['pages.comic.toast.sync.start_rescan'](),
				success: m['pages.comic.toast.sync.rescan_success'](),
				error: (err) => m['pages.comic.toast.rescan_error']({ msg: extractErrorMessage(err) })
			});
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleClearMetadata() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(() => metadataSync.clearMetadata(id.toString()), {
				loading: m['pages.comic.toast.sync.start_clear_metadata'](),
				success: m['pages.comic.toast.sync.clear_metadata_success'](),
				error: (err) =>
					m['pages.comic.toast.clear_metadata_error']({ msg: extractErrorMessage(err) })
			});
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleRegenerateCover() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(() => invoke(HOME_COMMANDS.regenerateComicCover, { id: id.toString() }), {
				loading: m['pages.comic.toast.sync.start_regenerate_cover'](),
				success: m['pages.comic.toast.sync.regenerate_cover_success'](),
				error: (err) =>
					m['pages.comic.toast.regenerate_cover_error']({ msg: extractErrorMessage(err) })
			});
			coverCacheBust = Date.now();
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleRegenerateVolumeCovers() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(() => invoke(HOME_COMMANDS.regenerateVolumeCovers, { id: id.toString() }), {
				loading: m['pages.comic.toast.sync.start_regenerate_volume_covers'](),
				success: m['pages.comic.toast.sync.regenerate_volume_covers_success'](),
				error: (err) =>
					m['pages.comic.toast.regenerate_volume_covers_error']({ msg: extractErrorMessage(err) })
			});
			volumeCoverCacheBust = Date.now();
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleDeepRescanComic() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(() => invoke(HOME_COMMANDS.deepRescanComic, { id: id.toString() }), {
				loading: m['pages.comic.toast.sync.start_deep_rescan'](),
				success: m['pages.comic.toast.sync.success'](),
				error: (err) => m['pages.comic.toast.deep_rescan_error']({ msg: extractErrorMessage(err) })
			});
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	async function handleExternalSyncChange(value: boolean) {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(
				() => invoke(HOME_COMMANDS.toggleComicExternalSync, { id: id.toString(), enabled: value }),
				{
					loading: m['pages.comic.toast.sync.start_toggle'](),
					success: value
						? m['pages.comic.toast.sync.enabled']()
						: m['pages.comic.toast.sync.disabled'](),
					error: (err) =>
						m['pages.comic.toast.sync.toggle_error']({ msg: extractErrorMessage(err) })
				}
			);
			if (manga) {
				manga.metadata.externalSync = value;
			}
			await invalidateAll();
		} catch {
			// Erro já foi mostrado pelo toastAsync acima.
		}
	}

	const pairedPeersForUi = $derived(
		peers.pairedPeers.map((peer) => ({
			peerId: peer.peerId,
			label: peers.peerLabel(peer.peerId),
			addrs: peer.addrs
		}))
	);
	const syncingPeerIds = $derived(
		peers.pairedPeers
			.filter((peer) => p2pSync.isSyncing(peer.peerId, 'comic'))
			.map((peer) => peer.peerId)
	);

	async function handleSyncToDevice(peerId: string, addrs: number[], direction: SyncDirection) {
		if (!manga?.title) return;
		await toastAsync(() => p2pSync.syncComic(peerId, addrs, manga!.title, direction), {
			loading: m['pages.comic.preferences.p2p_sync.toast.start'](),
			success: m['pages.comic.preferences.p2p_sync.toast.success'](),
			error: (err) =>
				m['pages.comic.preferences.p2p_sync.toast.error']({ msg: extractErrorMessage(err) })
		}).catch(() => {
			// Erro já foi mostrado pelo toastAsync acima.
		});
	}

	onMount(() => {
		(async () => {
			await Promise.all([
				volumeViewPreference.loadVolumeViewMode(),
				bookmarkStore.loadBookmarks(),
				peers.startListening(),
				p2pSync.startListening()
			]);
		})();

		return () => {
			peers.stopListening();
			p2pSync.stopListening();
		};
	});

	const comicId = $derived(
		activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId
	);

	$effect(() => {
		const id = comicId;
		if (!id) return;

		untrack(() => {
			isHistoryLoading = true;
			const fetchHistory = invoke(HISTORY_COMMANDS.getComic, { comicId: id.toString() }).catch(
				() => null
			);
			const fetchRead = invoke<string[]>(HISTORY_COMMANDS.getReadChapters, {
				comicId: id.toString()
			}).catch(() => []);
			const fetchBookmark = bookmarkStore.getComicBookmark(id);

			Promise.all([fetchHistory, fetchRead, fetchBookmark]).then(([history, read, bookmark]) => {
				readingHistory = history;
				readChapters = read;
				currentBookmarkId = bookmark?.id ?? null;
				isHistoryLoading = false;
			});
		});
	});

	function handleReadNow() {
		if (readingHistory) {
			goto('/reader', {
				state: {
					comicDirectoryId: readingHistory.comicDirectoryId.toString(),
					startPage: readingHistory.lastPage,
					chapterScope: readingHistory.comicName,
					chapter: {
						id: readingHistory.chapterArchiveId.toString(),
						name: readingHistory.chapterName,
						path: readingHistory.chapterPath,
						chapterSort: readingHistory.chapterSort,
						isSpecial: readingHistory.isSpecial,
						lastModified: readingHistory.lastModified,
						volumeId: null,
						volumeName: null
					}
				}
			});
		} else {
			// Find first chapter available
			if (manga?.pagesData?.[0]?.items?.[0]) {
				openReader(manga.pagesData[0].items[0] as unknown as Chapter);
			}
		}
	}

	$effect(() => {
		if (data.comic) {
			const comic = data.comic;
			untrack(() => {
				activeComic.set(comic, resolveCover(comic.artwork));
			});
		}
	});

	$effect(() => {
		const value = volumeViewPreference.volumeViewMode;

		untrack(() => {
			volumeViewPreference.saveVolumeViewMode(value);
		});
	});

	$effect(() => {
		const volumeId = expandedVolumeId;
		const query = searchQuery;
		const currentSortBy = sortBy;
		const trigger = syncRefreshTrigger;

		const comic = activeComic.item ?? data.comic;
		if (!comic) return;

		untrack(() => {
			void trigger;
			chapterStore.clear(true);
			chapterStore.fetch(comic.relations.directoryId, currentSortBy, volumeId, query || null);
		});
	});

	// Nem `sync:files:complete` (sync em massa, sem escopo de quadrinho) nem
	// `sync:comic:complete` (escopado, `comicName` no payload — ver
	// `comic_handler.rs::COMPLETE_EVENT`) atualizavam esta página antes: capítulos recém-
	// chegados, capa/banner e metadata (ex.: `ComicInfo.xml` reprocessado) ficavam
	// desatualizados até o usuário navegar pra fora e voltar. `sync:files:complete` não carrega
	// quais quadrinhos foram tocados, então trata qualquer sync em massa como potencialmente
	// relevante; `sync:comic:complete` só dispara o refresh se for sobre o quadrinho aberto.
	//
	// A checagem do nome do quadrinho atual (e o disparo em si) precisam ficar dentro do
	// `untrack` — este efeito só pode depender de `p2pSync.log`. Se lesse `activeComic.item`/
	// `manga` fora do untrack, o próprio `invalidateAll()`/re-fetch de capítulos disparado aqui
	// mudaria esses valores e re-executaria o efeito de novo, entrando em loop.
	$effect(() => {
		const entry = p2pSync.log[0];
		if (!entry || entry.status !== 'complete') return;
		if (entry.kind !== 'files' && entry.kind !== 'comic') return;

		untrack(() => {
			// `manga.title` é o mesmo valor que `handleSyncToDevice` manda como `comicName` pro
			// backend (`p2pSync.syncComic(peerId, addrs, manga.title)`) — comparação simétrica
			// com o que `comic_handler.rs` ecoa de volta no payload de conclusão.
			if (entry.kind === 'comic' && entry.comicName && entry.comicName !== manga?.title) return;

			syncRefreshTrigger++;
			coverCacheBust = Date.now();
			invalidateAll();
		});
	});

	const manga = $derived.by(() => {
		const item = activeComic.item ?? data.comic;
		if (!item) return null;

		const chaptersData = chapterStore.chapters;

		const allItems = (chaptersData?.archive.items ?? [])
			.map((comic, index) => ({
				id: comic.id.toString(),
				name: comic.name,
				title: comic.name,
				fileName: comic.name,
				isRead: Array.isArray(readChapters) ? readChapters.includes(comic.id.toString()) : false,
				chapterSort: comic.chapterSort,
				path: comic.path,
				volumeId: comic.volumeId,
				volumeName: comic.volumeName,
				isSpecial: comic.isSpecial,
				lastModified: comic.lastModified,
				chapterIndex: index
			}))
			.filter((comic) => !expandedVolumeId || comic.volumeId === expandedVolumeId);

		// Corta a lista plana em blocos fixos só pra virtualização de renderização
		// (ComicChapterList/ComicVolumeList montam um AcerolaHeroButton por item
		// só quando o bloco está perto do scroll) — não tem relação com a busca,
		// que já veio inteira do backend.
		const pagesData = Array.from(
			{ length: Math.ceil(allItems.length / RENDER_CHUNK_SIZE) || 0 },
			(_, page) => ({
				page,
				items: allItems.slice(page * RENDER_CHUNK_SIZE, (page + 1) * RENDER_CHUNK_SIZE)
			})
		);

		const volumes = (chaptersData?.archive.volumes ?? []).map((volume) => {
			const volCover = bustCache(
				volume.coverUri ? resolveArtworkPath(volume.coverUri) : null,
				volumeCoverCacheBust
			);
			const volBanner = bustCache(
				volume.bannerUri ? resolveArtworkPath(volume.bannerUri) : null,
				volumeCoverCacheBust
			);

			const fallbackCover = bustCache(resolveCover(item.artwork), coverCacheBust);
			const fallbackBanner = bustCache(resolveBanner(item.artwork), coverCacheBust);

			return {
				id: volume.id.toString(),
				title: volume.name,
				totalChapters: volume.chapterCount,
				coverUri: volCover || fallbackCover,
				bannerUri: volBanner || fallbackBanner || fallbackCover,
				hasMore: expandedVolumeId === volume.id.toString() && allItems.length < volume.chapterCount,
				chapters: []
			};
		});

		return {
			id: item.relations.directoryId.toString(),
			title: item.metadata.title || item.filesystem.folderName,
			// Total real de capítulos do quadrinho (soma volumes) — não usar `totalItems`
			// aqui, pois esse valor reflete a paginação/filtro de volume atualmente aberto.
			chaptersCount: item.metadata.chapterCount,
			rating: item.metadata.rating ?? null,
			cover: bustCache(resolveCover(item.artwork), coverCacheBust),
			banner: bustCache(resolveBanner(item.artwork), coverCacheBust),
			pagesData,
			volumes,
			pageSize: RENDER_CHUNK_SIZE,
			metadata: {
				description:
					item.metadata.description || m['pages.comic.metadata.description.unavailable'](),
				author: item.metadata.author || m['pages.comic.metadata.unknown.author'](),
				status: item.metadata.status || m['pages.comic.metadata.unknown.status'](),
				source: item.metadata.activeSource || 'LOCAL',
				externalSync: item.metadata.externalSync,
				genres: []
			}
		};
	});
</script>

{#if manga}
	<div
		in:fade={{ duration: 200 }}
		out:fade={{ duration: 200 }}
		class="text-text fixed inset-x-0 top-8 bottom-0 z-50 flex overflow-hidden bg-base"
	>
		<div class="pointer-events-none absolute inset-0 overflow-hidden">
			<img
				alt={manga.title}
				src={manga.banner || manga.cover}
				referrerpolicy="no-referrer"
				class="h-full w-full scale-150 object-cover opacity-20 blur-[120px]"
			/>

			<!-- NOTE: Aqui é onde os dados são injetados no banner -->
			<div class="absolute inset-0 bg-base/40"></div>
		</div>

		<ComicMetadataPanel
			data={{
				title: manga.title,
				author: manga.metadata.author,
				status: manga.metadata.status,
				source: manga.metadata.source,
				chaptersCount: manga.chaptersCount,
				description: manga.metadata.description,
				cover: manga.cover,
				bookmarkColor: bookmarkStore.bookmarks.find((b) => b.id === currentBookmarkId)?.color,
				bookmarkName: bookmarkStore.bookmarks.find((b) => b.id === currentBookmarkId)?.name
			}}
			state={{
				isResuming: !!readingHistory,
				isLoading: isHistoryLoading
			}}
			events={{
				onBack,
				onReadNow: handleReadNow
			}}
		/>

		<div
			class="scrollbar-hide relative z-10 flex flex-1 flex-col overflow-y-auto [overflow-anchor:none]"
		>
			<div
				class="sticky top-0 z-50 flex h-20 items-center justify-between border-b border-surface/30 bg-base/90 px-6 py-2 backdrop-blur-md lg:hidden"
			>
				<AcerolaButtonIcon events={{ onClick: onBack }} ui={{ size: 'sm' }}>
					<ArrowLeft size={20} />
				</AcerolaButtonIcon>

				<span class="max-w-50 truncate text-sm font-black tracking-widest uppercase">
					{manga.title}
				</span>

				<div class="w-10"></div>
			</div>

			<ComicHeroBanner
				data={{
					banner: manga.banner,
					genres: manga.metadata.genres,
					rating: manga.rating,
					chapterCount: manga.chaptersCount
				}}
			/>

			<div class="mx-auto w-full max-w-5xl space-y-12 p-8 lg:p-16">
				<div
					class="sticky top-0 z-40 -mx-4 flex items-center justify-between border-b border-surface/30 bg-base/5 px-4 backdrop-blur-3xl"
				>
					<div
						class="relative"
						use:slidingIndicator={{
							selector: '[data-state="on"]',
							indicatorClass: 'bottom-0 h-1 rounded-full bg-primary',
							track: ['x', 'width']
						}}
					>
						<AcerolaToggleGroup
							config={{ type: 'single' }}
							state={{ value: activeTab }}
							events={{
								onValueChange: (value) => {
									if (typeof value === 'string') activeTab = value;
								}
							}}
							ui={{ spacing: 4 }}
						>
							{#snippet children()}
								<ToggleGroupItem
									value="content"
									class="border-none bg-transparent py-6 text-sm font-black tracking-[0.2em] uppercase data-[state=on]:bg-transparent data-[state=on]:text-primary"
								>
									{chapterStore.chapters?.hasVolumeStructure
										? m['pages.comic.tabs.volumes']()
										: m['pages.comic.tabs.chapters']()}
								</ToggleGroupItem>

								<ToggleGroupItem
									value="preferences"
									class="border-none bg-transparent py-6 text-sm font-black tracking-[0.2em] uppercase data-[state=on]:bg-transparent data-[state=on]:text-primary"
								>
									{m['pages.comic.tabs.preferences']()}
								</ToggleGroupItem>
							{/snippet}
						</AcerolaToggleGroup>
					</div>

					{#if activeTab === 'content'}
						<div class="flex items-center gap-2 pr-4">
							{#if !chapterStore.chapters?.hasVolumeStructure}
								<input
									type="text"
									bind:value={searchQuery}
									placeholder={m['pages.comic.filter_placeholder']()}
									class="w-40 rounded-full border border-surface/30 bg-surface/20 px-6 py-2 text-[10px] font-black tracking-widest transition-all focus:w-60 focus:ring-2 focus:ring-primary/50 focus:outline-none"
								/>
							{/if}

							<AcerolaPopover
								state={{ open: showSortMenu }}
								events={{ onOpenChange: (open) => (showSortMenu = open) }}
								ui={{
									align: 'end',
									side: 'bottom',
									sideOffset: 8,
									contentClass: 'w-48 p-2 rounded-xl'
								}}
							>
								{#snippet trigger()}
									<AcerolaButton ui={{ variant: 'ghost', size: 'sm', class: 'rounded-lg' }}>
										<ArrowUpDown size={16} />
										{m['pages.comic.sort.button']()}
									</AcerolaButton>
								{/snippet}

								{#snippet content()}
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-lg' }}
										events={{
											onClick: () => {
												sortBy = 'number_asc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'number_asc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										{m['pages.comic.sort.number.asc']()}
									</AcerolaButton>
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-lg' }}
										events={{
											onClick: () => {
												sortBy = 'number_desc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'number_desc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										{m['pages.comic.sort.number.desc']()}
									</AcerolaButton>
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-lg' }}
										events={{
											onClick: () => {
												sortBy = 'modified_desc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'modified_desc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										{m['pages.comic.sort.modified.desc']()}
									</AcerolaButton>
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-lg' }}
										events={{
											onClick: () => {
												sortBy = 'modified_asc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'modified_asc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										{m['pages.comic.sort.modified.asc']()}
									</AcerolaButton>
								{/snippet}
							</AcerolaPopover>
						</div>
					{/if}
				</div>

				{#if activeTab === 'content' && chapterSelection.isSelectionMode}
					<div class="-mx-4 flex items-center gap-2 border-b border-surface/30 px-4 py-3">
						<span class="text-sm font-medium text-muted-foreground">
							{m['pages.comic.selection.selected']({ count: chapterSelection.selectedCount })}
						</span>
						<AcerolaButton
							ui={{ variant: 'ghost', size: 'sm', class: 'rounded-lg' }}
							events={{ onClick: handleSelectAllChapters }}
						>
							{m['pages.comic.selection.select_all']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{ variant: 'secondary', size: 'sm', class: 'rounded-lg' }}
							events={{ onClick: handleBatchMarkRead }}
						>
							{m['pages.comic.selection.mark_read']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{ variant: 'secondary', size: 'sm', class: 'rounded-lg' }}
							events={{ onClick: handleBatchMarkUnread }}
						>
							{m['pages.comic.selection.mark_unread']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{ variant: 'ghost', size: 'sm', class: 'rounded-lg' }}
							events={{ onClick: () => chapterSelection.exitSelectionMode() }}
						>
							{m['pages.comic.selection.cancel']()}
						</AcerolaButton>
					</div>
				{/if}

				<div class="min-h-150">
					{#if activeTab === 'content'}
						{#if chapterStore.chapters?.hasVolumeStructure}
							<ComicVolumeList
								data={{
									volumes: manga.volumes,
									pagesData: manga.pagesData,
									loading: chapterStore.loading,
									pageSize: manga.pageSize,
									viewMode: volumeViewPreference.volumeViewMode,
									isSelectionMode: chapterSelection.isSelectionMode,
									isSelected: chapterSelection.isSelected
								}}
								events={{
									onExpand: (value) => (expandedVolumeId = value),
									onOpenChapter: openReader,
									onToggleSelect: handleToggleSelect,
									onEnterSelection: handleEnterSelection,
									onMarkRead: handleMarkRead,
									onMarkUnread: handleMarkUnread
								}}
							/>
						{:else}
							<ComicChapterList
								data={{
									pagesData: manga.pagesData,
									totalChapters: manga.chaptersCount,
									pageSize: manga.pageSize,
									isSelectionMode: chapterSelection.isSelectionMode,
									isSelected: chapterSelection.isSelected
								}}
								events={{
									onOpenChapter: openReader,
									onToggleSelect: handleToggleSelect,
									onEnterSelection: handleEnterSelection,
									onMarkRead: handleMarkRead,
									onMarkUnread: handleMarkUnread
								}}
							/>
						{/if}
					{:else if activeTab === 'preferences'}
						<ComicPreferences
							data={{
								hasVolumeStructure: chapterStore.chapters?.hasVolumeStructure ?? false,
								bookmarks: bookmarkStore.bookmarks,
								pairedPeers: pairedPeersForUi
							}}
							state={{
								volumeViewMode: volumeViewPreference.volumeViewMode,
								bookmarkId: currentBookmarkId,
								externalSyncEnabled: manga.metadata.externalSync,
								syncingPeerIds
							}}
							events={{
								onVolumeViewModeChange: (value) => (volumeViewPreference.volumeViewMode = value),
								onBookmarkChange: async (value) => {
									const id =
										activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
									if (!id) return;

									if (value) {
										await bookmarkStore.assignToComic(id, value);
									} else {
										await bookmarkStore.removeComicBookmark(id);
									}
									currentBookmarkId = value;
								},
								onExternalSyncChange: handleExternalSyncChange,
								onSyncMangadex: handleSyncMangadex,
								onSyncAnilist: handleSyncAnilist,
								onSyncComicInfo: handleSyncComicInfo,
								onRescanComic: handleRescanComic,
								onDeepRescanComic: handleDeepRescanComic,
								onRegenerateCover: handleRegenerateCover,
								onRegenerateVolumeCovers: handleRegenerateVolumeCovers,
								onClearMetadata: handleClearMetadata,
								onSyncToDevice: handleSyncToDevice
							}}
						/>
					{/if}
				</div>
			</div>
		</div>
	</div>
{:else}
	<div class="text-overlay flex h-screen items-center justify-center bg-base">
		<RefreshCw size={48} class="animate-spin text-primary" />
	</div>
{/if}
