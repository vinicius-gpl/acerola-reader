<script lang="ts" module>
	export type ComicVolumeButtonProps = {
		data: {
			title: string;
			totalChapters: number;
			viewMode?: 'cover' | 'banner';
			coverUri?: string | null;
			bannerUri?: string | null;
			isExpanded?: boolean;
		};
		events: {
			onClick: () => void;
		};
	};
</script>

<script lang="ts">
	import Folder from '@lucide/svelte/icons/folder';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	import { cn } from '$lib/utils/cn.utils';
	import { AspectRatio } from '$lib/components/ui/aspect-ratio';
	import { m } from '$lib/paraglide/messages';

	let { data, events }: ComicVolumeButtonProps = $props();

	const activeBanner = $derived(
		(data.viewMode ?? 'cover') === 'banner' ? data.bannerUri || data.coverUri : null
	);
	const activeCover = $derived(data.coverUri || data.bannerUri);
</script>

<button
	onclick={events.onClick}
	class="group relative flex w-full items-center justify-between overflow-hidden rounded-3xl border border-surface/40 bg-mantle/50 p-5 text-left transition-all duration-300 hover:border-primary/50 hover:bg-mantle/70"
>
	{#if activeBanner}
		<div class="pointer-events-none absolute inset-0 z-0 overflow-hidden">
			<img
				src={activeBanner}
				alt={data.title}
				class="h-full w-full scale-105 object-cover opacity-20 blur-[2px] transition-all duration-300 group-hover:scale-110 group-hover:opacity-30"
			/>

			<div class="absolute inset-0 bg-linear-to-r from-mantle via-mantle/90 to-transparent"></div>
		</div>
	{/if}

	<div class="relative z-10 flex min-w-0 items-center gap-4">
		<div class="w-18 shrink-0 overflow-hidden rounded-2xl border border-white/10 bg-muted">
			<AspectRatio ratio={2 / 3}>
				{#if activeCover}
					<img
						src={activeCover}
						alt={data.title}
						class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
					/>
				{:else}
					<div class="flex h-full w-full items-center justify-center">
						<Folder size={28} />
					</div>
				{/if}
			</AspectRatio>
		</div>

		<div class="min-w-0">
			<h3 class="line-clamp-1 text-lg font-bold text-foreground">
				{data.title}
			</h3>

			<p class="text-sm text-muted-foreground">
				{m['pages.comic.volume.chapters_included']({ count: data.totalChapters })}
			</p>
		</div>
	</div>

	<div
		class={cn(
			'relative z-10 ml-4 shrink-0 transition-transform duration-300',
			data.isExpanded && 'rotate-180'
		)}
	>
		<ChevronDown size={20} />
	</div>
</button>
