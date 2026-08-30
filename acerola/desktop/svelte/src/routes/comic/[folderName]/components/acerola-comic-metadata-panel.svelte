<script lang="ts" module>
	export type ComicMetadataPanelProps = {
		data: {
			title: string;
			author: string;
			status: string;
			source: string;
			chaptersCount: number;
			description: string;
			cover: string | null;
			bookmarkColor?: number | null;
			bookmarkName?: string | null;
		};
		state?: {
			isResuming?: boolean;
			isLoading?: boolean;
		};
		events: {
			onBack: () => void;
			onReadNow?: () => void;
		};
	};
</script>

<script lang="ts">
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaCardImage from '$lib/components/acerola-card/acerola-card-image.svelte';
	import AcerolaBookmarkRibbon from '$lib/components/acerola-bookmark-ribbon/acerola-bookmark-ribbon.svelte';
	import PlaceholderManga from '$lib/assets/placeholder/placeholder_manga.svg?component';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import Play from '@lucide/svelte/icons/play';
	import { m } from '$lib/paraglide/messages';

	let { data, events, state }: ComicMetadataPanelProps = $props();
</script>

<div
	class="relative z-10 hidden h-full w-100 shrink-0 flex-col border-r border-surface/30 bg-mantle/60 backdrop-blur-3xl select-none lg:flex"
>
	<div class="flex h-full flex-col p-10">
		<AcerolaButtonIcon events={{ onClick: events.onBack }} ui={{ class: 'group mb-10 shadow-lg' }}>
			<ArrowLeft size={24} />
		</AcerolaButtonIcon>

		<div
			class="scrollbar-hide flex flex-1 flex-col items-center space-y-6 overflow-y-auto pt-3 text-center"
		>
			<AcerolaCardImage
				data={{
					title: data.title,
					cover: data.cover
				}}
				ui={{
					class: 'w-64 shrink-0 [&_.mt-3]:hidden'
				}}
			>
				{#snippet floatingBadge()}
					{#if data.bookmarkColor != null}
						<AcerolaBookmarkRibbon
							color={data.bookmarkColor}
							name={data.bookmarkName ?? undefined}
							class="-top-1.5 left-5 h-10 w-6"
						/>
					{/if}
				{/snippet}

				{#snippet placeholder()}
					<div class="h-full w-full bg-surface">
						<PlaceholderManga class="h-full w-full" />
					</div>
				{/snippet}
			</AcerolaCardImage>

			<div class="w-full space-y-3 px-4">
				<div class="space-y-1">
					<h1 class="line-clamp-2 text-4xl leading-tight font-black tracking-tighter">
						{data.title}
					</h1>
					<p class="text-lg font-bold text-primary">{data.author}</p>
				</div>

				<div class="flex flex-wrap justify-center gap-2">
					<span
						class="text-text rounded-lg border border-surface/30 bg-surface/80 px-4 py-1.5 text-[10px] font-black tracking-widest uppercase shadow-sm"
						>{data.status}</span
					>
					<span
						class="text-text rounded-lg border border-surface/30 bg-surface/80 px-4 py-1.5 text-[10px] font-black tracking-widest uppercase shadow-sm"
						>{data.source}</span
					>
				</div>
			</div>

			<div class="w-full space-y-3 rounded-3xl border border-surface/30 bg-base/30 p-6 text-left">
				<div class="flex items-center justify-between">
					<h3 class="text-overlay text-[10px] font-black tracking-widest uppercase">
						{m['pages.comic.metadata.synopsis']()}
					</h3>
					<span class="text-[9px] font-black tracking-widest text-primary/60 uppercase"
						>{m['pages.comic.metadata.chapters.count']({ count: data.chaptersCount })}</span
					>
				</div>
				<p class="text-subtext line-clamp-6 text-xs leading-relaxed font-medium">
					{data.description}
				</p>
			</div>

			<div class="mt-auto w-full space-y-3 pt-4">
				<AcerolaButton
					events={{ onClick: events.onReadNow }}
					ui={{
						class: 'flex w-full items-center justify-center gap-3 rounded-3xl py-8 font-black',
						disabled: state?.isLoading
					}}
				>
					<Play size={24} fill="currentColor" />
					{state?.isResuming
						? m['pages.history.resume']
							? m['pages.history.resume']()
							: 'Continue Reading'
						: m['pages.comic.metadata.read_now']()}
				</AcerolaButton>
			</div>
		</div>
	</div>
</div>
