<script lang="ts" module>
	export type Chapter = {
		id: string;
		name?: string;
		title: string;
		fileName: string;
		isRead: boolean;
		hasConflict?: boolean;
		chapterSort: string;
		path?: string;
		volumeId?: string | null;
		volumeName?: string | null;
		isSpecial?: boolean;
		lastModified?: number;
		chapterIndex?: number;
	};

	export type ChapterPage = {
		page: number;
		items: Chapter[];
	};

	export type ComicChapterListProps = {
		data: {
			pagesData: ChapterPage[];
			totalChapters: number;
			pageSize: number;
			isSelectionMode?: boolean;
			isSelected?: (id: string) => boolean;
		};
		events?: {
			onOpenChapter?: (chapter: Chapter) => void;
			onToggleSelect?: (chapter: Chapter) => void;
			onEnterSelection?: (chapter: Chapter) => void;
			onMarkRead?: (chapter: Chapter) => void;
			onMarkUnread?: (chapter: Chapter) => void;
		};
	};
</script>

<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Check from '@lucide/svelte/icons/check';
	import CheckSquare from '@lucide/svelte/icons/check-square';
	import MoreVertical from '@lucide/svelte/icons/more-vertical';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import { m } from '$lib/paraglide/messages';

	let { data, events }: ComicChapterListProps = $props();

	let openMenuId = $state<string | null>(null);

	/**
	 * ITEM_HEIGHT (112px): Altura total do slot (item + gap).
	 * BUTTON_HEIGHT (100px): Altura real do botão para caber Title + Description + Padding.
	 */
	const ITEM_HEIGHT = 112;
	const BUTTON_HEIGHT = 100;

	const totalPages = $derived(Math.ceil(data.totalChapters / data.pageSize));

	// Mesma cor de destaque que o círculo/badge do ícone usa pra cada estado — conflito
	// tem prioridade sobre lido (precisa de resolução do usuário, é mais urgente que um
	// capítulo já lido).
	function rowClass(chapter: Chapter): string {
		const base = 'h-full flex-nowrap overflow-hidden';
		if (chapter.hasConflict)
			return `${base} border-destructive/30 bg-destructive/10 hover:bg-destructive/20`;
		if (chapter.isRead) return `${base} border-primary/30 bg-primary/10 hover:bg-primary/20`;
		return `${base} border-surface/40 bg-mantle/40 hover:bg-surface/30`;
	}

	// INFO: Reativo de propósito — decide o que fica montado no DOM. Dado de
	// capítulo fica em cache pra sempre (não evictamos mais), mas manter todo
	// AcerolaHeroButton já carregado montado pra sempre gera dezenas de
	// milhares de nós de DOM numa série grande — cada AcerolaHeroButton monta
	// perto de 40 nós. Renderizar só o que está perto da viewport atual
	// resolve isso sem causar flicker: como o dado já está em cache, sair e
	// voltar da janela é instantâneo, sem refetch.
	let visiblePages = new SvelteSet<number>();

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
	 * Reciclagem de nós de DOM, igual RecyclerView/LazyColumn: um pool FIXO de
	 * "cadeiras" reaproveitadas em vez de montar/desmontar um AcerolaHeroButton
	 * por página toda vez que ela entra/sai da janela de renderização. Cada
	 * cadeira é keyed pela própria posição no pool (não pelo id do capítulo),
	 * então trocar qual página ela representa só rebind os props — o nó de
	 * DOM nunca é destruído, o que elimina o pop-in/fade toda vez que se rola.
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

<div class="relative flex w-full flex-col">
	{#if totalPages > 0}
		{#each Array(totalPages) as _, pageIndex}
			<!-- prettier-ignore -->
			{@const itemsInThisPage = pageIndex === totalPages - 1 ? data.totalChapters % data.pageSize || data.pageSize : data.pageSize}

			<!-- Espaçador: só reserva a altura no fluxo do documento e dispara o
			     IntersectionObserver — o conteúdo real vem do pool reciclado abaixo. -->
			<div use:trackPage={pageIndex} style:height="{itemsInThisPage * ITEM_HEIGHT}px"></div>
		{/each}

		{#each slots as slot, slotIndex (slotIndex)}
			{@const blockData =
				slot.pageIndex !== null
					? data.pagesData.find((page) => page.page === slot.pageIndex)
					: undefined}

			{#each Array(data.pageSize) as _, itemIndex (itemIndex)}
				{@const chapter = blockData?.items[itemIndex]}

				{#if chapter}
					<div
						class="absolute right-0 left-0 animate-in duration-300 fade-in"
						style:top="{(slot.pageIndex ?? 0) * data.pageSize * ITEM_HEIGHT +
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
										? events?.onToggleSelect?.(chapter)
										: events?.onOpenChapter?.(chapter)
							}}
							ui={{
								class: rowClass(chapter)
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
											<div class="h-5 w-5 rounded-full border-2 border-muted-foreground"></div>
										</div>
									{/if}
								{:else if chapter.hasConflict}
									<!-- Mesma ideia do círculo de "lido" (ícone principal + cor diferente), mas
									     com bg-destructive/15 (tintado) em vez de sólido: não existe
									     --destructive-foreground no tema (nenhum lugar do app usa `destructive`
									     como fundo sólido com texto/ícone em cima), então não há garantia de
									     contraste pra essa combinação — tintado, o ícone fica só na cor
									     `text-destructive` sobre o fundo por trás, que já é usada em outros
									     lugares do app. -->
									<div
										class="flex h-6 w-6 items-center justify-center rounded-full bg-destructive/15 text-destructive"
									>
										<TriangleAlert size={16} />
									</div>
								{:else if chapter.isRead}
									<div
										class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-crust"
									>
										<Check size={16} strokeWidth={3} />
									</div>
								{:else}
									<div class="text-primary">
										<BookOpen size={24} />
									</div>
								{/if}
							{/snippet}

							{#snippet action()}
								<div class="flex items-center gap-2">
									{#if chapter.hasConflict}
										<span
											class="rounded-full bg-muted px-3 py-1 text-[10px] font-black tracking-widest text-muted-foreground uppercase"
										>
											{m['pages.comic.metadata.conflict']()}
										</span>
									{/if}
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
																	events?.onMarkUnread?.(chapter);
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
																	events?.onMarkRead?.(chapter);
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
																events?.onEnterSelection?.(chapter);
															}
														}}
													>
														<CheckSquare size={16} class="shrink-0 text-muted-foreground" />
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
	{:else}
		<div class="space-y-4 py-20 text-center opacity-50">
			<RefreshCw size={48} class="mx-auto animate-spin text-primary" />
			<p class="text-sm font-black tracking-widest uppercase">{m['pages.comic.loading']()}</p>
		</div>
	{/if}
</div>
