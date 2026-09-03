<script lang="ts">
	import { Toaster as Sonner, type ToasterProps as SonnerProps } from 'svelte-sonner';
	import { mode } from 'mode-watcher';
	import { fade } from 'svelte/transition';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import OctagonXIcon from '@lucide/svelte/icons/octagon-x';
	import InfoIcon from '@lucide/svelte/icons/info';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import { toastIconEnter } from '$lib/utils/toast-icon-motion.utils';

	let { ...restProps }: SonnerProps = $props();
</script>

<Sonner
	theme={mode.current}
	richColors
	class="toaster group"
	style="--normal-bg: var(--color-popover); --normal-text: var(--color-popover-foreground); --normal-border: var(--color-border);"
	{...restProps}
>
	{#snippet loadingIcon()}
		<span use:toastIconEnter={'loading'} out:fade={{ duration: 120 }}>
			<Loader2Icon class="size-4 animate-spin" />
		</span>
	{/snippet}
	{#snippet successIcon()}
		<span use:toastIconEnter={'success'} out:fade={{ duration: 120 }}>
			<CircleCheckIcon class="size-4" />
		</span>
	{/snippet}
	{#snippet errorIcon()}
		<span use:toastIconEnter={'error'} out:fade={{ duration: 120 }}>
			<OctagonXIcon class="size-4" />
		</span>
	{/snippet}
	{#snippet infoIcon()}
		<span use:toastIconEnter={'info'} out:fade={{ duration: 120 }}>
			<InfoIcon class="size-4" />
		</span>
	{/snippet}
	{#snippet warningIcon()}
		<span use:toastIconEnter={'warning'} out:fade={{ duration: 120 }}>
			<TriangleAlertIcon class="size-4" />
		</span>
	{/snippet}
</Sonner>
