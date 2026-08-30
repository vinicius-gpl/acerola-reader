<script lang="ts" module>
	export type VolumeChapter = {
		id: string;
		name?: string;
		title: string;
		fileName: string;
		isRead: boolean;
		chapterSort?: string;
		path?: string;
		volumeId?: string | null;
		volumeName?: string | null;
		isSpecial?: boolean;
		lastModified?: number;
		chapterIndex?: number;
	};

	export type Volume = {
		id: string;
		title: string;
		chapters: VolumeChapter[];
		totalChapters: number;
		hasMore: boolean;
		coverUri?: string | null;
		bannerUri?: string | null;
	};

	export type VolumePage = {
		page: number;
		items: VolumeChapter[];
	};

	export type ComicVolumeListProps = {
		data: {
			volumes: Volume[];
			pagesData?: VolumePage[];
			loading?: boolean;
			pageSize?: number;
			viewMode?: 'cover' | 'banner';
			isSelectionMode?: boolean;
			isSelected?: (id: string) => boolean;
		};
		events: {
			onExpand: (volumeId: string | null) => void;
			onOpenChapter?: (chapter: VolumeChapter) => void;
			onToggleSelect?: (chapter: VolumeChapter) => void;
			onEnterSelection?: (chapter: VolumeChapter) => void;
			onMarkRead?: (chapter: VolumeChapter) => void;
			onMarkUnread?: (chapter: VolumeChapter) => void;
		};
	};
</script>

<script lang="ts">
	import { cn } from '$lib/utils/cn.utils';
	import { slide } from 'svelte/transition';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import ComicVolumeButton from './acerola-comic-volume-button.svelte';
	import Folder from '@lucide/svelte/icons/folder';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Check from '@lucide/svelte/icons/check';
	import CheckSquare from '@lucide/svelte/icons/check-square';
	import MoreVertical from '@lucide/svelte/icons/more-vertical';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import { onMount, untrack } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { m } from '$lib/paraglide/messages';

	let { data, events }: ComicVolumeListProps = $props();

	let openMenuId = $state<string | null>(null);

	const volumes = $derived(data.volumes ?? []);
	const pagesData = $derived(data.pagesData ?? []);
	const loading = $derived(data.loading ?? false);
	const pageSize = $derived(data.pageSize ?? 25);
	const viewMode = $derived(data.viewMode ?? 'cover');

	let expandedVolumeId = $state<string | null>(null);

	// INFO: Reativo de propósito — decide o que fica montado no DOM, não só o
	// que dispara fetch. Ver o mesmo raciocínio em acerola-comic-chapter-list.svelte.
	let visiblePages = new SvelteSet<number>();

	const ITEM_HEIGHT = 112;
	const BUTTON_HEIGHT = 100;

	const toggleVolume = (volumeId: string) => {
		if (expandedVolumeId === volumeId) {
			expandedVolumeId = null;
			visiblePages.clear();
		} else {
			expandedVolumeId = volumeId;
		}

		events.onExpand(expandedVolumeId);
	};

	/**
	 * Criado de forma eager (não em onMount): as actions `use:trackPage` dos
	 * blocos do {#each} rodam durante a montagem inicial, antes do onMount do
	 * componente disparar — se o observer fosse criado em onMount, as páginas
	 * já montadas nunca seriam observadas.
	 */
	const observer = new IntersectionObserver(
		(entries) => {
			entries.forEach((entry) => {
				const pageStr = (entry.target as HTMLElement).dataset.page;
				if (!pageStr) return;

				const page = parseInt(pageStr, 10);

				if (entry.isIntersecting) {
					visiblePages.add(page);
				} else {
					visiblePages.delete(page);
				}
			});
		},
		{ rootMargin: '1200px 0px' }
	);

	onMount(() => {
		return () => observer.disconnect();
	});

	function trackPage(node: HTMLElement, page: number) {
		node.dataset.page = page.toString();
		observer.observe(node);
		return {
			destroy() {
				observer.unobserve(node);
			}
		};
	}

	/**
	 * Reciclagem de nós de DOM — mesmo raciocínio de acerola-comic-chapter-list.svelte.
	 * Só um volume fica expandido por vez, então um único pool compartilhado
	 * serve pra qualquer volume que esteja aberto no momento.
	 */
	const POOL_SIZE = 6;

	let slots = $state<Array<{ pageIndex: number | null }>>(
		Array.from({ length: POOL_SIZE }, () => ({ pageIndex: null }))
	);

	const visiblePageList = $derived(Array.from(visiblePages).sort((a, b) => a - b));

	$effect(() => {
		const visible = visiblePageList;

		untrack(() => {
			const currentlyAssigned = new Set(
				slots.map((slot) => slot.pageIndex).filter((page): page is number => page !== null)
			);

			for (const slot of slots) {
				if (slot.pageIndex !== null && !visible.includes(slot.pageIndex)) {
					slot.pageIndex = null;
				}
			}

			const freeSlots = slots.filter((slot) => slot.pageIndex === null);
			const unassignedPages = visible.filter((page) => !currentlyAssigned.has(page));

			for (let i = 0; i < unassignedPages.length && i < freeSlots.length; i++) {
				freeSlots[i].pageIndex = unassignedPages[i];
			}
		});
	});
</script>

<div class="space-y-4">
	{#if volumes && volumes.length > 0}
		{#each volumes as volume}
			<ComicVolumeButton
				data={{
					title: volume.title,
					totalChapters: volume.totalChapters,
					viewMode,
					coverUri: volume.coverUri,
					bannerUri: volume.bannerUri,
					isExpanded: expandedVolumeId === volume.id
				}}
				events={{
					onClick: () => toggleVolume(volume.id)
				}}
			/>

			{#if expandedVolumeId === volume.id}
				{@const totalPages = Math.ceil(volume.totalChapters / pageSize)}

				<div transition:slide class="ml-10 border-l border-surface/30 p-4 pt-0">
					<div class="relative flex w-full flex-col">
						{#if volume.totalChapters > 0}
							{#each Array(totalPages) as _, pageIndex}
								<!-- prettier-ignore -->
								{@const itemsInThisPage = pageIndex === totalPages - 1? volume.totalChapters % pageSize || pageSize: pageSize}

								<!-- Espaçador: só reserva a altura no fluxo do documento e dispara o
								     IntersectionObserver — o conteúdo real vem do pool reciclado abaixo. -->
								<div
									use:trackPage={pageIndex}
									style:height="{itemsInThisPage * ITEM_HEIGHT}px"
								></div>
							{/each}

							{#each slots as slot, slotIndex (slotIndex)}
								{@const blockData =
									slot.pageIndex !== null
										? pagesData.find((page) => page.page === slot.pageIndex)
										: undefined}

								{#each Array(pageSize) as _, itemIndex (itemIndex)}
									{@const chapter = blockData?.items[itemIndex]}

									{#if chapter}
										<div
											class="absolute right-0 left-0 animate-in duration-300 fade-in"
											style:top="{(slot.pageIndex ?? 0) * pageSize * ITEM_HEIGHT +
												itemIndex * ITEM_HEIGHT}px"
											style:height="{BUTTON_HEIGHT}px"
										>
											<AcerolaHeroButton
												data={{
													title: chapter.title,
													description: chapter.fileName
												}}
												events={{
													onClick: () =>
														data.isSelectionMode
															? events.onToggleSelect?.(chapter)
															: events.onOpenChapter?.(chapter)
												}}
												ui={{
													class: chapter.isRead
														? 'h-full flex-nowrap overflow-hidden border-primary/30 bg-primary/10 hover:bg-primary/20'
														: 'h-full flex-nowrap overflow-hidden border-surface/40 bg-mantle/40 hover:bg-surface/30'
												}}
											>
												{#snippet icon()}
													{#if data.isSelectionMode}
														{#if data.isSelected?.(chapter.id)}
															<div
																class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-crust"
															>
																<Check size={16} strokeWidth={3} />
															</div>
														{:else}
															<div class="flex h-6 w-6 items-center justify-center">
																<div
																	class="h-5 w-5 rounded-full border-2 border-muted-foreground"
																></div>
															</div>
														{/if}
													{:else}
														<div class={chapter.isRead ? '' : 'text-primary'}>
															{#if chapter.isRead}
																<div
																	class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-crust"
																>
																	<Check size={16} strokeWidth={3} />
																</div>
															{:else}
																<BookOpen size={24} />
															{/if}
														</div>
													{/if}
												{/snippet}

												{#snippet action()}
													<div class="flex items-center gap-2">
														{#if chapter.isRead}
															<span
																class="rounded-full bg-primary/10 px-3 py-1 text-[10px] font-black tracking-widest text-primary uppercase"
															>
																{m['pages.comic.metadata.completed']
																	? m['pages.comic.metadata.completed']()
																	: 'LIDO'}
															</span>
														{/if}
														<div role="presentation" onclick={(event) => event.stopPropagation()}>
															<AcerolaPopover
																state={{ open: openMenuId === chapter.id }}
																events={{
																	onOpenChange: (open) => (openMenuId = open ? chapter.id : null)
																}}
																ui={{
																	align: 'end',
																	contentClass:
																		'w-52 overflow-hidden rounded-2xl border-border/40 bg-card/95 p-1.5 shadow-2xl backdrop-blur-md'
																}}
															>
																{#snippet trigger()}
																	<span
																		class="text-overlay flex size-10 items-center justify-center rounded-xl transition-colors hover:bg-surface/60 hover:text-primary"
																	>
																		<MoreVertical size={20} />
																	</span>
																{/snippet}

																{#snippet content()}
																	<div class="flex flex-col gap-0.5">
																		{#if chapter.isRead}
																			<AcerolaButton
																				ui={{
																					variant: 'ghost',
																					class:
																						'h-9 w-full justify-start gap-2.5 rounded-xl px-2.5 text-sm font-medium text-amber-500 hover:bg-amber-500/10 hover:text-amber-400'
																				}}
																				events={{
																					onClick: () => {
																						openMenuId = null;
																						events.onMarkUnread?.(chapter);
																					}
																				}}
																			>
																				<BookOpen size={16} class="shrink-0" />
																				{m['pages.comic.selection.mark_unread']()}
																			</AcerolaButton>
																		{:else}
																			<AcerolaButton
																				ui={{
																					variant: 'ghost',
																					class:
																						'h-9 w-full justify-start gap-2.5 rounded-xl px-2.5 text-sm font-medium text-primary hover:bg-primary/10'
																				}}
																				events={{
																					onClick: () => {
																						openMenuId = null;
																						events.onMarkRead?.(chapter);
																					}
																				}}
																			>
																				<Check size={16} class="shrink-0" />
																				{m['pages.comic.selection.mark_read']()}
																			</AcerolaButton>
																		{/if}

																		<div class="mx-1 my-1 h-px bg-border/60"></div>

																		<AcerolaButton
																			ui={{
																				variant: 'ghost',
																				class:
																					'h-9 w-full justify-start gap-2.5 rounded-xl px-2.5 text-sm font-medium'
																			}}
																			events={{
																				onClick: () => {
																					openMenuId = null;
																					events.onEnterSelection?.(chapter);
																				}
																			}}
																		>
																			<CheckSquare
																				size={16}
																				class="shrink-0 text-muted-foreground"
																			/>
																			{m['pages.comic.selection.select']()}
																		</AcerolaButton>
																	</div>
																{/snippet}
															</AcerolaPopover>
														</div>
													</div>
												{/snippet}
											</AcerolaHeroButton>
										</div>
									{/if}
								{/each}
							{/each}
						{/if}

						{#if loading}
							<div class="flex items-center justify-center py-4">
								<RefreshCw size={24} class="animate-spin text-primary" />
							</div>
						{/if}

						{#if volume.totalChapters === 0 && !loading}
							<p class="py-4 text-center text-xs font-black tracking-widest uppercase opacity-50">
								{m['pages.comic.volume.no_chapters']()}
							</p>
						{/if}
					</div>
				</div>
			{/if}
		{/each}
	{:else}
		<div
			class="space-y-4 rounded-3xl border-2 border-dashed border-surface py-20 text-center opacity-50"
		>
			<RefreshCw size={48} class="animate-spin text-primary" />
			<p class="text-sm font-black tracking-widest uppercase">
				{m['pages.comic.volume.no_volumes']()}
			</p>
		</div>
	{/if}
</div>
