<script lang="ts" module>
	export type AcerolaDialogProps = {
		state?: {
			open?: boolean;
		};
		data?: {
			title?: string;
			description?: string;
		};
		events?: {
			onOpenChange?: (open: boolean) => void;
		};
		ui?: {
			class?: string;
			contentClass?: string;
			showCloseButton?: boolean;
		};
	};
</script>

<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index';
	import type { Snippet } from 'svelte';
	import { cn } from '$lib/utils/cn.utils';

	let {
		state = $bindable({ open: false }),
		data,
		events,
		ui,
		children
	}: AcerolaDialogProps & {
		children?: Snippet;
	} = $props();
</script>

<Dialog.Root open={state?.open} onOpenChange={events?.onOpenChange}>
	<Dialog.Content
		class={cn('font-dm-sans rounded-2xl', ui?.contentClass)}
		showCloseButton={ui?.showCloseButton}
	>
		{#if data?.title || data?.description}
			<Dialog.Header>
				{#if data?.title}
					<Dialog.Title class="text-xl font-bold tracking-tight">{data.title}</Dialog.Title>
				{/if}
				{#if data?.description}
					<Dialog.Description class="opacity-80">
						{data.description}
					</Dialog.Description>
				{/if}
			</Dialog.Header>
		{:else}
			<Dialog.Header class="sr-only">
				<Dialog.Title>Dialog</Dialog.Title>
				<Dialog.Description>Dialog Content</Dialog.Description>
			</Dialog.Header>
		{/if}

		{#if children}
			{@render children()}
		{/if}
	</Dialog.Content>
</Dialog.Root>
