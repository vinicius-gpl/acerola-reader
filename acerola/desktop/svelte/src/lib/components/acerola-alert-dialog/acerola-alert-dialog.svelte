<script lang="ts" module>
	export type AcerolaAlertDialogProps = {
		state?: {
			open: boolean;
		};
		data?: {
			title?: string;
			description?: string;
			cancelText?: string;
			actionText?: string;
		};
		events?: {
			onOpenChange?: (open: boolean) => void;
			onAction?: () => void;
			onCancel?: () => void;
		};
		ui?: {
			variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
		};
	};
</script>

<script lang="ts">
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index';
	import type { Snippet } from 'svelte';
	import { buttonVariants } from '$lib/components/ui/button/index';
	import { cn } from '$lib/utils/cn.utils';

	let {
		state = $bindable({ open: false }),
		data,
		events,
		ui,
		children
	}: AcerolaAlertDialogProps & {
		children?: Snippet;
	} = $props();
</script>

<AlertDialog.Root bind:open={state.open} onOpenChange={events?.onOpenChange}>
	<AlertDialog.Trigger>
		{#if children}
			{@render children()}
		{/if}
	</AlertDialog.Trigger>
	<AlertDialog.Content class="font-dm-sans rounded-2xl">
		<AlertDialog.Header>
			<AlertDialog.Title class="text-xl font-bold tracking-tight">{data?.title}</AlertDialog.Title>
			<AlertDialog.Description class="opacity-80">
				{data?.description}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer class="mt-6">
			<AlertDialog.Cancel class="rounded-xl" onclick={events?.onCancel}>
				{data?.cancelText ?? 'Cancel'}
			</AlertDialog.Cancel>
			<AlertDialog.Action
				class={cn('rounded-xl', buttonVariants({ variant: ui?.variant ?? 'default' }))}
				onclick={events?.onAction}
			>
				{data?.actionText ?? 'Continue'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
