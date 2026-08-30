<script lang="ts">
	import { browser } from '$app/environment';
	import { goto, replaceState } from '$app/navigation';
	import { page } from '$app/state';
	import type { ReaderChapterPayload } from '$lib/contracts/reader/reader.payloads';
	import { useReader } from '$lib/hooks/store/use-reader.svelte';
	import { useReaderMode } from '$lib/hooks/preferences/use-reader-mode.svelte';
	import { m } from '$lib/paraglide/messages';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { LIBRARY_EVENTS } from '$lib/contracts/library/chapter.events';
	import { LIBRARY_COMMANDS } from '$lib/contracts/library/chapter.commands';
	import { SESSION_KEYS } from '$lib/constants/session-keys';
	import { onMount, tick, untrack } from 'svelte';
	import { useReaderNavigation } from '$lib/hooks/store/use-reader-navigation.svelte';
	import { useHistory } from '$lib/hooks/store/use-history.svelte';
	import ReaderCommandPalette from './components/acerola-reader-command-palette.svelte';
	import ReaderFooter from './components/acerola-reader-footer.svelte';
	import ReaderPages from './components/acerola-reader-pages.svelte';
	import ReaderShell from './components/acerola-reader-shell.svelte';
	import ReaderToolbar from './components/acerola-reader-toolbar.svelte';
	import ReaderViewport from './components/acerola-reader-viewport.svelte';
	import {
		isReaderEditableTarget,
		type ReaderMode,
		useReaderZoom
	} from './hooks/use-reader-zoom.svelte';

	const reader = useReader();
	const zoom = useReaderZoom();
	const readerModeStore = useReaderMode();
	const history = useHistory();

	let observer: IntersectionObserver | null = null;
	let visibleRects = new Map<number, DOMRectReadOnly>();
	let pageNodes = new Map<number, HTMLElement>();
	let visiblePages = $state<number[]>([]);
	let openFailed = $state(false);
	let readingMode = $state<ReaderMode>('vertical');
	let modeSwitchPage = $state<number | null>(null);
	let commandOpen = $state(false);
	let commandValue = $state('');

	const nav = useReaderNavigation();

	function updateReadingMode(mode: ReaderMode) {
		readingMode = mode;
		void readerModeStore.saveReaderMode(mode);
	}

	const chapterProgressLabel = $derived.by(() => {
		if (nav.chapterIndex === undefined || nav.totalChapters === undefined) {
			return m['pages.reader.progress.chapter_unavailable']();
		}

		return m['pages.reader.progress.chapter_count']({
			current: nav.chapterIndex + 1,
			total: nav.totalChapters
		});
	});

	const chaptersRemainingLabel = $derived.by(() => {
		if (nav.chaptersRemaining === undefined) {
			return m['pages.reader.progress.remaining_unavailable']();
		}

		return nav.chaptersRemaining === 1
			? m['pages.reader.progress.remaining_one']()
			: m['pages.reader.progress.remaining_many']({ count: nav.chaptersRemaining });
	});

	const pageProgressPercent = $derived(
		reader.pageCount > 0 ? Math.round(((reader.currentPage + 1) / reader.pageCount) * 100) : 0
	);

	const pageProgressWidth = $derived(`${pageProgressPercent}%`);
	const readerScopeLabel = $derived(nav.state.chapterScope ?? chapterProgressLabel);

	const readerSubtitle = $derived(
		reader.session
			? m['pages.reader.progress.subtitle']({
					scope: readerScopeLabel,
					current: reader.currentPage + 1,
					total: reader.pageCount
				})
			: undefined
	);

	const isPaginatedMode = $derived(readingMode !== 'webtoon');
	const pageControlsDisabled = $derived(isPaginatedMode && zoom.isZoomed);
	const canPreviousPage = $derived(Boolean(reader.session) && reader.currentPage > 0);

	const canNextPage = $derived(
		Boolean(reader.session) && reader.currentPage < reader.pageCount - 1
	);

	const modeLabel = $derived.by(() => {
		if (readingMode === 'horizontal') return m['pages.reader.modes.horizontal.full']();
		if (readingMode === 'webtoon') return m['pages.reader.modes.webtoon']();
		return m['pages.reader.modes.vertical.full']();
	});

	function leaveReader() {
		if (window.history.length > 1) {
			window.history.back();
			return;
		}

		goto('/home');
	}

	function openCommandPalette() {
		commandValue = '';
		commandOpen = true;
	}

	function handleKeydown(event: KeyboardEvent) {
		const key = event.key.toLowerCase();
		const commandKey = event.ctrlKey || event.metaKey;

		if (commandKey && key === 'k') {
			event.preventDefault();
			commandOpen ? (commandOpen = false) : openCommandPalette();
			return;
		}

		if (commandOpen && key === 'escape') {
			event.preventDefault();
			commandOpen = false;
			return;
		}

		if (isReaderEditableTarget(event.target)) return;

		if (key === 'arrowright' && canNextPage && !pageControlsDisabled) {
			event.preventDefault();
			void goToReaderPage(reader.currentPage + 1);
			return;
		}

		if (key === 'arrowleft' && canPreviousPage && !pageControlsDisabled) {
			event.preventDefault();
			void goToReaderPage(reader.currentPage - 1);
			return;
		}

		if (commandKey && (event.key === '+' || event.key === '=')) {
			event.preventDefault();
			zoom.zoomIn();
			return;
		}

		if (commandKey && event.key === '-') {
			event.preventDefault();
			zoom.zoomOut();
			return;
		}

		if (commandKey && event.key === '0') {
			event.preventDefault();
			zoom.resetZoom();
			return;
		}

		if (key === 'z') {
			event.preventDefault();
			zoom.toggleZoomMode();
		}
	}

	function visiblePageOrder() {
		const horizontal = readingMode === 'horizontal';
		const viewportCenter = horizontal ? window.innerWidth / 2 : window.innerHeight / 2;

		return Array.from(visibleRects.entries())
			.sort(([, leftRect], [, rightRect]) => {
				const leftCenter = horizontal
					? leftRect.left + leftRect.width / 2
					: leftRect.top + leftRect.height / 2;

				const rightCenter = horizontal
					? rightRect.left + rightRect.width / 2
					: rightRect.top + rightRect.height / 2;

				return Math.abs(leftCenter - viewportCenter) - Math.abs(rightCenter - viewportCenter);
			})
			.map(([pageIndex]) => pageIndex);
	}

	function scrollPageIntoView(pageIndex: number, behavior: ScrollBehavior = 'smooth') {
		const node = pageNodes.get(pageIndex);
		if (!node) return;

		node.scrollIntoView({
			behavior,
			block: readingMode === 'webtoon' ? 'start' : 'center',
			inline: readingMode === 'horizontal' ? 'center' : 'nearest'
		});
	}

	async function goToReaderPage(pageIndex: number) {
		if (!reader.session || reader.pageCount === 0) return;

		const targetPage = Math.max(0, Math.min(pageIndex, reader.pageCount - 1));
		const distance = Math.abs(reader.currentPage - targetPage);

		await reader.goToPage(targetPage);
		zoom.resetPan();

		await tick();
		// Se for um salto muito grande, não use scroll suave para evitar disparar o observer em dezenas de páginas intermediárias
		scrollPageIntoView(targetPage, distance > 3 ? 'auto' : 'smooth');
	}

	async function start() {
		if (!nav.chapter) {
			nav.initializing = false;
			return;
		}

		await nav.loadContext();

		try {
			await reader.open(nav.chapter, nav.state.startPage ?? 0);
			nav.state.startPage = undefined; // Reseta a página inicial para que as próximas aberturas comecem do 0
			await tick();

			// Pequeno atraso para garantir que o DOM está pronto e o IntersectionObserver pegue o estado inicial
			setTimeout(() => {
				scrollPageIntoView(reader.currentPage, 'auto');

				// Outro tick para permitir que o observer registre o pulo antes de exibir a interface
				setTimeout(() => {
					nav.initializing = false;
				}, 50);
			}, 50);
		} catch {
			openFailed = true;
			nav.initializing = false;
		}
	}

	function retryOpen() {
		openFailed = false;
		nav.initializing = true;
		void start();
	}

	let currentChapterId = $state<string | null>(null);

	$effect(() => {
		if (nav.chapter && nav.chapter.id !== currentChapterId) {
			untrack(() => {
				currentChapterId = nav.chapter!.id;
				nav.initializing = true;
				void start();
			});
		}
	});

	let scrollTimeoutId: number;

	onMount(() => {
		void readerModeStore.loadReaderMode().then(() => {
			readingMode = readerModeStore.readerMode;
		});

		observer = new IntersectionObserver(
			(entries) => {
				let changed = false;

				for (const entry of entries) {
					const pageIndex = Number((entry.target as HTMLElement).dataset.page);
					if (!Number.isFinite(pageIndex)) continue;

					if (!entry.isIntersecting) {
						visibleRects.delete(pageIndex);
						changed = true;
						continue;
					}

					visibleRects.set(pageIndex, entry.boundingClientRect);
					changed = true;
				}

				if (changed) {
					window.clearTimeout(scrollTimeoutId);
					scrollTimeoutId = window.setTimeout(() => {
						visiblePages = visiblePageOrder();
					}, 50);
				}
			},
			{ threshold: 0.01 }
		);

		return () => observer?.disconnect();
	});

	$effect(() => {
		const mode = readingMode;
		if (!reader.session) return;

		untrack(() => {
			const targetPage = reader.currentPage;
			modeSwitchPage = targetPage;

			zoom.forceResetZoom();

			visibleRects.clear();
			visiblePages = [];

			void tick().then(() => {
				if (mode === readingMode) {
					scrollPageIntoView(targetPage, 'auto');

					window.setTimeout(() => {
						if (mode === readingMode && modeSwitchPage === targetPage) {
							modeSwitchPage = null;
						}
					}, 120);
				}
			});
		});
	});

	$effect(() => {
		if (!reader.session || visiblePages.length === 0) return;
		if (zoom.isZoomed) return;

		if (modeSwitchPage !== null) {
			const targetPage = modeSwitchPage;

			untrack(() => {
				void reader.loadPage(targetPage, false).catch(() => undefined);
			});
			return;
		}

		const targetPage = visiblePages[0];

		untrack(() => {
			if (targetPage !== reader.currentPage) {
				void reader.goToPage(targetPage);
			}

			for (const pageIndex of visiblePages) {
				void reader.loadPage(pageIndex, false).catch(() => undefined);
			}
		});
	});

	function trackPage(node: HTMLElement, pageIndex: number) {
		node.dataset.page = pageIndex.toString();
		pageNodes.set(pageIndex, node);
		observer?.observe(node);

		return {
			destroy() {
				observer?.unobserve(node);
				visibleRects.delete(pageIndex);

				if (pageNodes.get(pageIndex) === node) {
					pageNodes.delete(pageIndex);
				}
			}
		};
	}

	$effect(() => {
		const currentPage = reader.currentPage;
		const pageCount = reader.pageCount;
		const chapterId = nav.chapter?.id;
		const comicDirectoryId = nav.state.comicDirectoryId;

		if (!reader.session || !chapterId || !comicDirectoryId || pageCount === 0) return;

		untrack(() => {
			const isCompleted = (currentPage + 1) / pageCount >= 0.7;
			void history.updateReading(
				comicDirectoryId.toString(),
				chapterId.toString(),
				currentPage,
				isCompleted
			);
		});
	});
</script>

<svelte:window onkeydown={handleKeydown} onresize={zoom.clampPan} />

{#if nav.initializing}
	<div
		class="fixed inset-0 z-100 flex flex-col items-center justify-center bg-base/95 backdrop-blur-xl transition-opacity duration-300"
	>
		<div class="flex flex-col items-center gap-4">
			<div
				class="h-10 w-10 animate-spin rounded-full border-4 border-primary border-t-transparent"
			></div>
			<p class="animate-pulse text-sm font-black tracking-widest text-primary uppercase">
				{m['pages.reader.fallback.loading']()}
			</p>
		</div>
	</div>
{/if}

<ReaderShell>
	{#snippet toolbar()}
		<ReaderToolbar
			data={{
				title: nav.chapter?.name ?? m['pages.reader.fallback.chapter_unavailable'](),
				subtitle: readerSubtitle,
				zoomLevel: zoom.zoomLevel,
				zoomMode: zoom.zoomMode,
				isPaginatedMode,
				pageControlsDisabled,
				canPreviousPage,
				canNextPage
			}}
			state={{ readingMode }}
			events={{
				onBack: leaveReader,
				onReadingModeChange: updateReadingMode,
				onToggleQuickZoom: zoom.toggleQuickZoom,
				onToggleZoomMode: zoom.toggleZoomMode,
				onOpenCommandPalette: openCommandPalette,
				onPreviousPage: () => goToReaderPage(reader.currentPage - 1),
				onNextPage: () => goToReaderPage(reader.currentPage + 1)
			}}
		/>
	{/snippet}

	{#snippet viewport()}
		<ReaderViewport data={{ mode: readingMode }} context={{ zoom }}>
			<ReaderPages
				data={{
					mode: readingMode,
					pageCount: reader.pageCount,
					currentPage: reader.currentPage,
					openFailed,
					chapterAvailable: Boolean(nav.chapter)
				}}
				services={{
					pageAt: reader.pageAt,
					loadPage: reader.loadPage,
					trackPage
				}}
				events={{ onRetry: retryOpen }}
			/>
		</ReaderViewport>
	{/snippet}

	{#snippet footer()}
		<ReaderFooter
			data={{
				pageProgressPercent,
				pageProgressWidth,
				chapterProgressLabel,
				modeLabel,
				zoomStatusLabel: zoom.zoomStatusLabel,
				chaptersRemainingLabel,
				hasNextChapter: nav.hasNextChapter,
				hasPreviousChapter: nav.hasPreviousChapter
			}}
			state={{ readingMode }}
			events={{
				onReadingModeChange: updateReadingMode,
				onNextChapter: nav.goToNextChapter,
				onPreviousChapter: nav.goToPreviousChapter
			}}
		/>
	{/snippet}

	{#snippet command()}
		<ReaderCommandPalette
			data={{ zoomMode: zoom.zoomMode }}
			state={{
				open: commandOpen,
				value: commandValue,
				readingMode
			}}
			events={{
				onOpenChange: (open) => (commandOpen = open),
				onValueChange: (value) => (commandValue = value),
				onReadingModeChange: updateReadingMode,
				onToggleZoomMode: zoom.toggleZoomMode,
				onZoomIn: zoom.zoomIn,
				onZoomOut: zoom.zoomOut,
				onResetZoom: zoom.resetZoom
			}}
		/>
	{/snippet}
</ReaderShell>
