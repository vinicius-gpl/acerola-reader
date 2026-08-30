<script module lang="ts">
	import type { ReaderCachedPagePayload } from '$lib/contracts/reader/reader.payloads';
	import type { Action } from 'svelte/action';
	import type { ReaderMode } from '../hooks/use-reader-zoom.svelte';

	export type ReaderPageTracker = Action<HTMLElement, number>;

	export type ReaderPagesProps = {
		data: {
			mode: ReaderMode;
			pageCount: number;
			currentPage: number;
			openFailed: boolean;
			chapterAvailable: boolean;
		};
		services: {
			pageAt: (index: number) => ReaderCachedPagePayload | undefined;
			loadPage?: (index: number, setCurrent?: boolean) => Promise<unknown>;
			trackPage: ReaderPageTracker;
		};
		events?: {
			onRetry?: () => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/utils/cn.utils';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import { fade } from 'svelte/transition';
	import { PREFETCH_RADIUS } from '$lib/hooks/store/use-reader.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';

	let { data, services, events }: ReaderPagesProps = $props();

	const trackPage = $derived(services.trackPage);

	/**
	 * Todas as `pageCount` seções montam de uma vez (sem virtualização), então
	 * sem esse raio essa action dispararia loadPage para o capítulo inteiro
	 * assim que o leitor abre. O IntersectionObserver em +page.svelte já cobre
	 * o carregamento conforme o usuário rola — isso aqui é só o fallback pro
	 * primeiro paint ao redor da página atual.
	 */
	function loadIfMissing(node: HTMLElement, pageIndex: number) {
		if (!services.pageAt(pageIndex) && Math.abs(pageIndex - data.currentPage) <= PREFETCH_RADIUS) {
			void services.loadPage?.(pageIndex, false);
		}
		return {
			destroy() {}
		};
	}
</script>

{#if !data.chapterAvailable}
	<div class="text-overlay flex h-full items-center justify-center text-sm font-black uppercase">
		{m['pages.reader.fallback.chapter_unavailable']()}
	</div>
{:else if data.openFailed}
	<div class="flex h-full flex-col items-center justify-center gap-4 px-8 text-center">
		<AlertTriangle size={40} class="text-destructive" />
		<div class="space-y-1.5">
			<p class="text-sm font-black tracking-widest text-foreground uppercase">
				{m['pages.reader.fallback.open_failed.title']()}
			</p>
			<p class="text-overlay max-w-sm text-xs">
				{m['pages.reader.fallback.open_failed.desc']()}
			</p>
		</div>
		{#if events?.onRetry}
			<AcerolaButton
				ui={{ variant: 'secondary', class: 'rounded-xl gap-2' }}
				events={{ onClick: events.onRetry }}
			>
				<RefreshCw size={16} />
				{m['pages.reader.fallback.retry']()}
			</AcerolaButton>
		{/if}
	</div>
{:else if data.pageCount === 0}
	<div class="flex h-full items-center justify-center">
		<RefreshCw size={40} class="animate-spin text-primary" />
	</div>
{:else}
	<div
		class={cn(
			data.mode === 'horizontal' && 'flex h-full w-max',
			data.mode === 'webtoon' && 'mx-auto flex w-full max-w-3xl flex-col items-center gap-0 py-2',
			data.mode === 'vertical' && 'mx-auto flex h-full w-full max-w-6xl flex-col items-center'
		)}
	>
		{#each Array.from({ length: data.pageCount }) as _, pageIndex}
			{@const pageItem = services.pageAt(pageIndex)}

			<section
				use:trackPage={pageIndex}
				class={cn(
					data.mode === 'horizontal' &&
						'flex h-full w-screen shrink-0 snap-center items-center justify-center px-5 py-6',
					data.mode === 'webtoon' && 'flex w-full items-start justify-center',
					data.mode === 'vertical' &&
						'flex h-full w-full shrink-0 snap-center items-center justify-center px-3 py-6'
				)}
				aria-label={m['pages.reader.pages.label']({ page: pageIndex + 1 })}
			>
				{#if pageItem}
					<div
						class={cn(
							data.mode === 'webtoon' && 'mx-auto flex w-full justify-center',
							data.mode !== 'webtoon' && 'flex h-full w-full items-center justify-center'
						)}
					>
						<img
							in:fade={{ duration: 120 }}
							src={pageItem.url}
							alt={m['pages.reader.pages.image_alt']({ page: pageIndex + 1 })}
							class={cn(
								'bg-base object-contain',
								data.mode === 'webtoon' && 'w-full',
								data.mode !== 'webtoon' && 'max-h-full max-w-full shadow-2xl shadow-base/40'
							)}
							loading={Math.abs(pageIndex - data.currentPage) <= 2 ? 'eager' : 'lazy'}
							draggable="false"
						/>
					</div>
				{:else}
					<div
						use:loadIfMissing={pageIndex}
						class={cn(
							data.mode !== 'webtoon' &&
								'flex h-full w-full max-w-4xl items-center justify-center border border-surface/40 bg-base/50',
							data.mode === 'webtoon' &&
								'flex min-h-[64vh] w-full items-center justify-center border-y border-surface/40 bg-base/50'
						)}
					>
						<RefreshCw size={32} class="animate-spin text-primary" />
					</div>
				{/if}
			</section>
		{/each}
	</div>
{/if}
