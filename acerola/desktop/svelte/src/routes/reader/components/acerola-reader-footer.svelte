<script module lang="ts">
	import type { ReaderMode } from '../hooks/use-reader-zoom.svelte';

	export type ReaderFooterProps = {
		data: {
			pageProgressPercent: number;
			pageProgressWidth: string;
			chapterProgressLabel: string;
			modeLabel: string;
			zoomStatusLabel: string;
			chaptersRemainingLabel: string;
			hasNextChapter?: boolean;
			hasPreviousChapter?: boolean;
		};
		state: {
			readingMode: ReaderMode;
		};
		events: {
			onReadingModeChange: (mode: ReaderMode) => void;
			onNextChapter?: () => void;
			onPreviousChapter?: () => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import ReaderModeToggle from './acerola-reader-mode-toggle.svelte';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';

	let { data, events, state }: ReaderFooterProps = $props();
</script>

<footer
	class="relative z-20 shrink-0 border-t border-surface/40 bg-base/95 px-4 py-3 backdrop-blur-md"
>
	<div class="flex flex-col gap-3">
		<div
			class="text-overlay flex items-center justify-between gap-3 text-[10px] font-black tracking-widest uppercase"
		>
			<span class="shrink-0"
				>{m['pages.reader.progress.read_percent']({ percent: data.pageProgressPercent })}</span
			>
			<span class="hidden min-w-0 truncate md:inline"
				>{data.modeLabel} - {data.zoomStatusLabel}</span
			>
			<span class="min-w-0 truncate text-right">{data.chaptersRemainingLabel}</span>
		</div>

		<div
			role="progressbar"
			aria-valuemin="0"
			aria-valuemax="100"
			aria-valuenow={data.pageProgressPercent}
			class="h-2 overflow-hidden rounded-full bg-surface/60"
			title={data.chapterProgressLabel}
		>
			<div
				class="h-full rounded-full bg-primary transition-[width] duration-300 ease-out"
				style:width={data.pageProgressWidth}
			></div>
		</div>

		<div class="flex items-center justify-between gap-4">
			<div class="flex items-center gap-2">
				{#if data.hasPreviousChapter && events.onPreviousChapter}
					<button
						class="flex items-center gap-2 rounded-lg bg-surface/60 px-4 py-2 text-xs font-bold text-primary transition-all hover:bg-surface active:scale-95"
						onclick={events.onPreviousChapter}
					>
						<ChevronLeft size={16} />
						<span class="hidden md:inline">{m['pages.reader.actions.chapter.previous']()}</span>
					</button>
				{/if}

				<ReaderModeToggle
					state={{ value: state.readingMode }}
					events={{ onValueChange: events.onReadingModeChange }}
					ui={{ variant: 'mobile' }}
				/>
			</div>

			{#if data.hasNextChapter && events.onNextChapter}
				<button
					class="flex items-center gap-2 rounded-lg bg-surface/60 px-4 py-2 text-xs font-bold text-primary transition-all hover:bg-surface active:scale-95"
					onclick={events.onNextChapter}
				>
					<span class="hidden md:inline">{m['pages.reader.actions.chapter.next']()}</span>
					<ChevronRight size={16} />
				</button>
			{/if}
		</div>
	</div>
</footer>
