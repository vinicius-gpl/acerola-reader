<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaPopoverProps = {
		state?: {
			open?: boolean;
		};
		events?: {
			onOpenChange?: (open: boolean) => void;
		};
		ui?: {
			contentClass?: string;
			align?: 'start' | 'center' | 'end';
			side?: 'top' | 'right' | 'bottom' | 'left';
			sideOffset?: number;
			collisionPadding?: number | { top?: number; right?: number; bottom?: number; left?: number };
		};
	};

	export type AcerolaPopoverSnippets = {
		trigger: Snippet;
		content: Snippet;
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import * as Popover from '$lib/components/ui/popover/index.js';

	let {
		trigger,
		content,
		state: control,
		events,
		ui
	}: AcerolaPopoverProps & AcerolaPopoverSnippets = $props();

	let open = $state(false);
	let lastOpen = $state(false);

	$effect(() => {
		if (control?.open === undefined) return;

		const nextOpen = control.open;

		if (nextOpen !== untrack(() => lastOpen)) {
			open = nextOpen;
			lastOpen = nextOpen;
		}
	});

	$effect(() => {
		if (open === lastOpen) return;

		lastOpen = open;
		events?.onOpenChange?.(open);
	});
</script>

<Popover.Root bind:open>
	<Popover.Trigger>
		{@render trigger()}
	</Popover.Trigger>

	<Popover.Content
		align={ui?.align}
		side={ui?.side}
		sideOffset={ui?.sideOffset}
		collisionPadding={ui?.collisionPadding}
		class={ui?.contentClass}
	>
		{@render content()}
	</Popover.Content>
</Popover.Root>
