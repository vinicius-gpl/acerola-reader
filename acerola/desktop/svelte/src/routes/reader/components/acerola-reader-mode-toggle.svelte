<script module lang="ts">
	import type { ReaderMode } from '../hooks/use-reader-zoom.svelte';

	export type ReaderModeToggleProps = {
		state: {
			value: ReaderMode;
		};
		events: {
			onValueChange: (value: ReaderMode) => void;
		};
		ui?: {
			variant?: 'desktop' | 'mobile';
			class?: string;
		};
	};
</script>

<script lang="ts">
	import AcerolaToggleGroup from '$lib/components/acerola-toggle-group/acerola-toggle-group.svelte';
	import { ToggleGroupItem } from '$lib/components/ui/toggle-group/index.js';
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/utils/cn.utils';
	import Columns2 from '@lucide/svelte/icons/columns-2';
	import Rows2 from '@lucide/svelte/icons/rows-2';
	import ScrollText from '@lucide/svelte/icons/scroll-text';

	let { events, state, ui }: ReaderModeToggleProps = $props();

	const variant = $derived(ui?.variant ?? 'desktop');
</script>

<AcerolaToggleGroup
	config={{ type: 'single' }}
	state={{ value: state.value }}
	events={{
		onValueChange: (value) => {
			if (value === 'vertical' || value === 'horizontal' || value === 'webtoon') {
				events.onValueChange(value);
			}
		}
	}}
	ui={{
		class: cn(
			variant === 'desktop' &&
				'hidden shrink-0 gap-1 rounded-xl border border-surface/40 bg-mantle/60 p-1 md:flex',
			variant === 'mobile' &&
				'grid grid-cols-3 gap-1 rounded-xl border border-surface/40 bg-mantle/60 p-1 md:hidden',
			ui?.class
		)
	}}
>
	{#snippet children()}
		<ToggleGroupItem
			value="vertical"
			title={m['pages.reader.modes.vertical.full']()}
			class={cn(
				'h-9 gap-2 rounded-lg text-[10px] font-black tracking-widest uppercase data-[state=on]:bg-primary data-[state=on]:text-primary-foreground',
				variant === 'desktop' && 'px-3'
			)}
		>
			<Rows2 size={15} />
			{#if variant === 'desktop'}
				<span class="hidden lg:inline">{m['pages.reader.modes.vertical.short']()}</span>
			{:else}
				<span>{m['pages.reader.modes.vertical.short']()}</span>
			{/if}
		</ToggleGroupItem>

		<ToggleGroupItem
			value="horizontal"
			title={m['pages.reader.modes.horizontal.full']()}
			class={cn(
				'h-9 gap-2 rounded-lg text-[10px] font-black tracking-widest uppercase data-[state=on]:bg-primary data-[state=on]:text-primary-foreground',
				variant === 'desktop' && 'px-3'
			)}
		>
			<Columns2 size={15} />
			{#if variant === 'desktop'}
				<span class="hidden lg:inline">{m['pages.reader.modes.horizontal.short']()}</span>
			{:else}
				<span>{m['pages.reader.modes.horizontal.short']()}</span>
			{/if}
		</ToggleGroupItem>

		<ToggleGroupItem
			value="webtoon"
			title={m['pages.reader.modes.webtoon']()}
			class={cn(
				'h-9 gap-2 rounded-lg text-[10px] font-black tracking-widest uppercase data-[state=on]:bg-primary data-[state=on]:text-primary-foreground',
				variant === 'desktop' && 'px-3'
			)}
		>
			<ScrollText size={15} />
			{#if variant === 'desktop'}
				<span class="hidden lg:inline">{m['pages.reader.modes.webtoon']()}</span>
			{:else}
				<span>{m['pages.reader.modes.webtoon']()}</span>
			{/if}
		</ToggleGroupItem>
	{/snippet}
</AcerolaToggleGroup>
