<script module lang="ts">
	import type { MetadataSource, SortBy, SortOrder } from '$lib/contracts/home/home.payloads';
	import type { Category } from '$lib/contracts/bookmarks/bookmarks.payloads';

	/** 'all' shows every comic, 'none' shows comics without a bookmark, a number filters by category id. */
	export type BookmarkFilter = 'all' | 'none' | number;

	export type FilterPanelProps = {
		state?: { open?: boolean };
		data: {
			sortBy: SortBy;
			sortOrder: SortOrder;
			showHidden: boolean;
			metadataSource: MetadataSource;
			bookmarkFilter: BookmarkFilter;
			bookmarks: Category[];
		};
		events: {
			onApply: (params: {
				sortBy: SortBy;
				sortOrder: SortOrder;
				showHidden: boolean;
				metadataSource: MetadataSource;
				bookmarkFilter: BookmarkFilter;
			}) => void;
			onClose: () => void;
		};
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import { fly, fade } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import ArrowDown from '@lucide/svelte/icons/arrow-down';
	import X from '@lucide/svelte/icons/x';
	import Check from '@lucide/svelte/icons/check';
	import Eye from '@lucide/svelte/icons/eye';
	import Bookmark from '@lucide/svelte/icons/bookmark';
	import AcerolaSwitch from '$lib/components/acerola-switch/acerola-switch.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import { m } from '$lib/paraglide/messages';

	let { state: controlState, data, events }: FilterPanelProps = $props();

	let localSortBy = $state<SortBy>('title');
	let localSortOrder = $state<SortOrder>('asc');
	let localShowHidden = $state(false);
	let localMetadataSource = $state<MetadataSource>('all');
	let localBookmarkFilter = $state<BookmarkFilter>('all');
	let wasOpen = false;

	$effect(() => {
		const isOpen = Boolean(controlState?.open);
		if (isOpen && !wasOpen) {
			localSortBy = data.sortBy;
			localSortOrder = data.sortOrder;
			localShowHidden = data.showHidden;
			localMetadataSource = data.metadataSource;
			localBookmarkFilter = data.bookmarkFilter;
		}
		wasOpen = isOpen;
	});

	const sortOptions = $derived<{ value: SortBy; labelKey: string }[]>([
		{ value: 'title', labelKey: m['pages.home.filter_panel.sort_options.title']() },
		{
			value: 'chapterCount',
			labelKey: m['pages.home.filter_panel.sort_options.chapter_count']()
		},
		{ value: 'lastUpdated', labelKey: m['pages.home.filter_panel.sort_options.last_updated']() }
	]);

	const metadataSources = $derived<{ value: MetadataSource; label: string }[]>([
		{ value: 'all', label: m['pages.home.filter_panel.metadata_sources.all']() },
		{ value: 'comicinfo', label: m['pages.home.filter_panel.metadata_sources.comicinfo']() },
		{ value: 'mangadex', label: m['pages.home.filter_panel.metadata_sources.mangadex']() },
		{ value: 'anilist', label: m['pages.home.filter_panel.metadata_sources.anilist']() },
		{ value: 'no_metadata', label: m['pages.home.filter_panel.metadata_sources.no_metadata']() }
	]);

	function selectSort(value: SortBy) {
		if (localSortBy === value) {
			localSortOrder = localSortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			localSortBy = value;
			localSortOrder = 'asc';
		}
	}

	function handleApply() {
		events.onApply({
			sortBy: localSortBy,
			sortOrder: localSortOrder,
			showHidden: localShowHidden,
			metadataSource: localMetadataSource,
			bookmarkFilter: localBookmarkFilter
		});
	}

	function handleReset() {
		localSortBy = 'title';
		localSortOrder = 'asc';
		localShowHidden = false;
		localMetadataSource = 'all';
		localBookmarkFilter = 'all';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && controlState?.open) {
			events.onClose();
		}
	}

	const hasChanges = $derived(
		localSortBy !== data.sortBy ||
			localSortOrder !== data.sortOrder ||
			localShowHidden !== data.showHidden ||
			localMetadataSource !== data.metadataSource ||
			localBookmarkFilter !== data.bookmarkFilter
	);

	const hasActiveFilters = $derived(
		localShowHidden || localMetadataSource !== 'all' || localBookmarkFilter !== 'all'
	);
</script>

<svelte:window onkeydown={handleKeydown} />

{#if controlState?.open}
	<!-- Backdrop -->
	<div
		role="presentation"
		class="fixed inset-x-0 top-8 bottom-0 z-40 bg-background/40 backdrop-blur-sm"
		transition:fade={{ duration: 200 }}
		onclick={events.onClose}
	></div>

	<!-- Panel -->
	<div
		role="dialog"
		aria-modal="true"
		aria-label={m['pages.home.filter_panel.aria_label']()}
		class="fixed top-8 right-0 bottom-0 z-50 flex w-full max-w-full flex-col border-l border-border/60 bg-background/95 shadow-2xl backdrop-blur-xl sm:max-w-sm"
		transition:fly={{ x: 400, duration: 300, easing: cubicOut }}
	>
		<!-- Header -->
		<div class="flex items-center justify-between border-b border-border/40 px-6 py-5">
			<div class="flex items-center gap-3">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/15 text-primary">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<line x1="21" y1="4" x2="14" y2="4"></line>
						<line x1="10" y1="4" x2="3" y2="4"></line>
						<line x1="21" y1="12" x2="12" y2="12"></line>
						<line x1="8" y1="12" x2="3" y2="12"></line>
						<line x1="21" y1="20" x2="16" y2="20"></line>
						<line x1="12" y1="20" x2="3" y2="20"></line>
						<line x1="14" y1="2" x2="14" y2="6"></line>
						<line x1="8" y1="10" x2="8" y2="14"></line>
						<line x1="16" y1="18" x2="16" y2="22"></line>
					</svg>
				</div>
				<div>
					<h2 class="font-bold tracking-tight text-base text-foreground">
						{m['pages.home.filter_panel.aria_label']()}
					</h2>
					{#if hasActiveFilters}
						<span class="text-[10px] font-semibold tracking-wider text-primary uppercase"
							>{m['pages.home.filter_panel.active_filters']()}</span
						>
					{/if}
				</div>
			</div>
			<AcerolaButtonIcon
				events={{ onClick: events.onClose }}
				ui={{
					variant: 'ghost',
					tone: 'muted',
					class: 'h-8 w-8',
					title: m['pages.home.filter_panel.close_aria'](),
					'aria-label': m['pages.home.filter_panel.close_aria']()
				}}
			>
				<X size={18} />
			</AcerolaButtonIcon>
		</div>

		<!-- Content - scrollable -->
		<div class="flex-1 space-y-7 overflow-y-auto px-6 py-5">
			<!-- Sort Section -->
			<section>
				<p class="mb-3 text-[11px] font-bold tracking-widest text-primary uppercase">
					{m['pages.home.filter_panel.sort_section_title']()}
				</p>
				<div
					class="space-y-1.5"
					role="radiogroup"
					aria-label={m['pages.home.filter_panel.sort_radiogroup_aria']()}
				>
					{#each sortOptions as option}
						{@const isSelected = localSortBy === option.value}
						<button
							type="button"
							role="radio"
							aria-checked={isSelected}
							class="group flex w-full items-center justify-between rounded-xl px-3.5 py-3 text-sm font-medium transition-all duration-200 focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none {isSelected
								? 'bg-primary/15 text-foreground ring-1 ring-primary/30'
								: 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
							onclick={() => selectSort(option.value)}
						>
							<span class="truncate">{option.labelKey}</span>
							{#if isSelected}
								<div class="ml-2 flex shrink-0 items-center gap-1.5">
									<div
										class="flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-[10px] font-bold text-primary"
									>
										{#if localSortOrder === 'asc'}
											<ArrowUp size={10} />
											<span>A–Z</span>
										{:else}
											<ArrowDown size={10} />
											<span>Z–A</span>
										{/if}
									</div>
									<Check size={14} class="text-primary" />
								</div>
							{/if}
						</button>
					{/each}
				</div>
			</section>

			<!-- Divider -->
			<div class="h-px bg-border/40"></div>

			<!-- Filter Section -->
			<section>
				<p class="mb-3 text-[11px] font-bold tracking-widest text-primary uppercase">
					{m['pages.home.filter_panel.filter_section_title']()}
				</p>

				<!-- Show Hidden Toggle Row -->
				<button
					type="button"
					role="switch"
					aria-checked={localShowHidden}
					class="flex w-full cursor-pointer items-center justify-between rounded-xl px-3.5 py-3 text-left transition-colors hover:bg-muted/40 focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none"
					onclick={() => (localShowHidden = !localShowHidden)}
				>
					<div class="flex items-center gap-3">
						<div
							class="flex h-8 w-8 items-center justify-center rounded-lg bg-muted/60 text-muted-foreground"
						>
							<Eye size={15} />
						</div>
						<div>
							<p class="text-sm font-medium text-foreground">
								{m['pages.home.filter_panel.show_hidden.title']()}
							</p>
							<p class="text-xs text-muted-foreground">
								{m['pages.home.filter_panel.show_hidden.desc']()}
							</p>
						</div>
					</div>
					<div class="pointer-events-none">
						<AcerolaSwitch state={{ checked: localShowHidden }} ui={{ size: 'sm' }} />
					</div>
				</button>
			</section>

			<!-- Metadata Source Section -->
			<section>
				<p class="mb-3 text-[11px] font-bold tracking-widest text-primary uppercase">
					{m['pages.home.filter_panel.metadata_sources_title']()}
				</p>
				<div
					class="flex flex-wrap gap-2"
					role="group"
					aria-label={m['pages.home.filter_panel.metadata_sources_aria']()}
				>
					{#each metadataSources as source}
						{@const isSelected = localMetadataSource === source.value}
						<button
							type="button"
							aria-pressed={isSelected}
							class="rounded-xl border px-3.5 py-2 text-sm font-semibold transition-all duration-200 focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none active:scale-95 {isSelected
								? 'border-primary/50 bg-primary/20 text-primary shadow-sm shadow-primary/20'
								: 'border-border/60 bg-muted/30 text-muted-foreground hover:border-border hover:bg-muted/60 hover:text-foreground'}"
							onclick={() => (localMetadataSource = source.value)}
						>
							{source.label}
						</button>
					{/each}
				</div>
			</section>

			<!-- Bookmark Filter Section -->
			<section>
				<p class="mb-3 text-[11px] font-bold tracking-widest text-primary uppercase">
					{m['pages.home.filter_panel.bookmark_filter_title']()}
				</p>
				<div
					class="flex flex-wrap gap-2"
					role="group"
					aria-label={m['pages.home.filter_panel.bookmark_filter_aria']()}
				>
					<button
						type="button"
						aria-pressed={localBookmarkFilter === 'all'}
						class="rounded-xl border px-3.5 py-2 text-sm font-semibold transition-all duration-200 focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none active:scale-95 {localBookmarkFilter ===
						'all'
							? 'border-primary/50 bg-primary/20 text-primary shadow-sm shadow-primary/20'
							: 'border-border/60 bg-muted/30 text-muted-foreground hover:border-border hover:bg-muted/60 hover:text-foreground'}"
						onclick={() => (localBookmarkFilter = 'all')}
					>
						{m['pages.home.filter_panel.bookmark_filter.all']()}
					</button>
					<button
						type="button"
						aria-pressed={localBookmarkFilter === 'none'}
						class="flex items-center gap-1.5 rounded-xl border px-3.5 py-2 text-sm font-semibold transition-all duration-200 focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none active:scale-95 {localBookmarkFilter ===
						'none'
							? 'border-primary/50 bg-primary/20 text-primary shadow-sm shadow-primary/20'
							: 'border-border/60 bg-muted/30 text-muted-foreground hover:border-border hover:bg-muted/60 hover:text-foreground'}"
						onclick={() => (localBookmarkFilter = 'none')}
					>
						<Bookmark size={12} class="opacity-70" />
						{m['pages.home.filter_panel.bookmark_filter.none']()}
					</button>
					{#each data.bookmarks as category (category.id)}
						{@const isSelected = localBookmarkFilter === category.id}
						{@const hexColor = '#' + (category.color & 0xffffff).toString(16).padStart(6, '0')}
						<button
							type="button"
							aria-pressed={isSelected}
							class="flex items-center gap-1.5 rounded-xl border px-3.5 py-2 text-sm font-semibold transition-all duration-200 focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none active:scale-95 {isSelected
								? 'border-primary/50 bg-primary/20 text-primary shadow-sm shadow-primary/20'
								: 'border-border/60 bg-muted/30 text-muted-foreground hover:border-border hover:bg-muted/60 hover:text-foreground'}"
							onclick={() => (localBookmarkFilter = category.id)}
						>
							<span class="h-2.5 w-2.5 shrink-0 rounded-full" style="background-color: {hexColor};"
							></span>
							{category.name}
						</button>
					{/each}
				</div>
			</section>
		</div>

		<!-- Footer Actions -->
		<div class="border-t border-border/40 px-6 py-5">
			<div class="flex gap-3">
				<button
					type="button"
					class="flex-1 rounded-xl border border-border/60 bg-muted/30 px-4 py-2.5 text-sm font-semibold text-muted-foreground transition-all hover:bg-muted/60 hover:text-foreground focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none active:scale-[0.98] disabled:opacity-40"
					onclick={handleReset}
					disabled={!hasChanges && !hasActiveFilters}
				>
					{m['pages.home.filter_panel.reset']()}
				</button>
				<button
					type="button"
					class="flex-1 rounded-xl bg-primary px-4 py-2.5 text-sm font-bold text-primary-foreground shadow-md shadow-primary/30 transition-all hover:bg-primary/90 hover:shadow-lg hover:shadow-primary/40 focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none active:scale-[0.98]"
					onclick={handleApply}
				>
					{m['pages.home.filter_panel.apply']()}
				</button>
			</div>
		</div>
	</div>
{/if}
