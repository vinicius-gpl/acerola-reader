<script module lang="ts">
	import type { ReaderMode } from '../hooks/acerola-use-reader-zoom.svelte';

	export type ReaderToolbarProps = {
		data: {
			title: string;
			subtitle?: string;
			zoomLevel: number;
			zoomMode: boolean;
			isPaginatedMode: boolean;
			pageControlsDisabled: boolean;
			canPreviousPage: boolean;
			canNextPage: boolean;
		};
		state: {
			readingMode: ReaderMode;
		};
		events: {
			onBack: () => void;
			onReadingModeChange: (mode: ReaderMode) => void;
			onToggleQuickZoom: () => void;
			onToggleZoomMode: () => void;
			onOpenCommandPalette: () => void;
			onPreviousPage: () => void | Promise<void>;
			onNextPage: () => void | Promise<void>;
		};
	};
</script>

<script lang="ts">
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import { m } from '$lib/paraglide/messages';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import CommandIcon from '@lucide/svelte/icons/command';
	import ZoomIn from '@lucide/svelte/icons/zoom-in';
	import ZoomOut from '@lucide/svelte/icons/zoom-out';
	import ReaderModeToggle from './acerola-reader-mode-toggle.svelte';

	let { data, events, state }: ReaderToolbarProps = $props();
</script>

<header
	class="relative z-20 flex min-h-16 shrink-0 items-center justify-between gap-3 border-b border-surface/40 bg-base/95 px-4 py-2 backdrop-blur-md"
>
	<div class="flex min-w-0 flex-1 items-center gap-3">
		<AcerolaButtonIcon
			events={{ onClick: events.onBack }}
			ui={{
				variant: 'ghost',
				title: m['pages.reader.actions.back']()
			}}
		>
			<ArrowLeft size={20} />
		</AcerolaButtonIcon>

		<div class="min-w-0">
			<p class="truncate text-sm font-black">{data.title}</p>
			{#if data.subtitle}
				<p class="text-overlay truncate text-xs">{data.subtitle}</p>
			{/if}
		</div>
	</div>

	<ReaderModeToggle
		state={{ value: state.readingMode }}
		events={{ onValueChange: events.onReadingModeChange }}
	/>

	<div class="flex shrink-0 items-center gap-1">
		<AcerolaButtonIcon
			events={{ onClick: events.onToggleQuickZoom }}
			ui={{
				variant: data.zoomLevel > 1 ? 'secondary' : 'ghost',
				title:
					data.zoomLevel > 1
						? `${m['pages.reader.actions.reset_zoom']()} (Ctrl+0)`
						: `${m['pages.reader.actions.apply_zoom']()} (Ctrl++)`
			}}
		>
			{#if data.zoomLevel > 1}
				<ZoomOut size={20} />
			{:else}
				<ZoomIn size={20} />
			{/if}
		</AcerolaButtonIcon>

		<AcerolaButtonIcon
			events={{ onClick: events.onToggleZoomMode }}
			ui={{
				variant: data.zoomMode ? 'default' : 'ghost',
				title: `${m['pages.reader.actions.zoom_mode']()} (Z)`
			}}
		>
			<ZoomIn size={20} />
		</AcerolaButtonIcon>

		<AcerolaButtonIcon
			events={{ onClick: events.onOpenCommandPalette }}
			ui={{
				variant: 'ghost',
				title: `${m['pages.reader.actions.commands']()} (Ctrl+K)`
			}}
		>
			<CommandIcon size={20} />
		</AcerolaButtonIcon>

		{#if data.isPaginatedMode}
			<AcerolaButtonIcon
				events={{ onClick: () => events.onPreviousPage() }}
				ui={{
					variant: 'ghost',
					disabled: !data.canPreviousPage || data.pageControlsDisabled,
					title: data.pageControlsDisabled
						? m['pages.reader.actions.page.navigation_locked']()
						: `${m['pages.reader.actions.page.previous']()} (←)`
				}}
			>
				<ChevronLeft size={20} />
			</AcerolaButtonIcon>

			<AcerolaButtonIcon
				events={{ onClick: () => events.onNextPage() }}
				ui={{
					variant: 'ghost',
					disabled: !data.canNextPage || data.pageControlsDisabled,
					title: data.pageControlsDisabled
						? m['pages.reader.actions.page.navigation_locked']()
						: `${m['pages.reader.actions.page.next']()} (→)`
				}}
			>
				<ChevronRight size={20} />
			</AcerolaButtonIcon>
		{/if}
	</div>
</header>
