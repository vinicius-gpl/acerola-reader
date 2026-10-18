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

	let { toastOptions, class: className, ...restProps }: SonnerProps = $props();
</script>

<Sonner
	theme={mode.current}
	richColors
	class="toaster group {className ?? ''}"
	toastOptions={{
		classes: {
			toast:
				'group toast rounded-2xl border px-4 py-3.5 text-sm font-medium backdrop-blur-2xl backdrop-saturate-150 transition-all duration-300 shadow-[0_1px_0_rgba(255,255,255,0.12)_inset,0_2px_4px_rgba(0,0,0,0.04),0_12px_28px_rgba(0,0,0,0.12)] dark:shadow-[0_1px_0_rgba(255,255,255,0.08)_inset,0_2px_4px_rgba(0,0,0,0.4),0_16px_36px_rgba(0,0,0,0.6)] data-[type=default]:border-border/80 data-[type=default]:bg-card/90 data-[type=default]:text-card-foreground data-[type=success]:border-emerald-500/40 data-[type=success]:bg-linear-to-b data-[type=success]:from-emerald-950/25 data-[type=success]:to-card/95 data-[type=success]:shadow-[0_0_24px_rgba(16,185,129,0.18)] data-[type=error]:border-rose-500/45 data-[type=error]:bg-linear-to-b data-[type=error]:from-rose-950/25 data-[type=error]:to-card/95 data-[type=error]:shadow-[0_0_24px_rgba(244,63,94,0.22)] data-[type=warning]:border-amber-500/45 data-[type=warning]:bg-linear-to-b data-[type=warning]:from-amber-950/25 data-[type=warning]:to-card/95 data-[type=warning]:shadow-[0_0_24px_rgba(245,158,11,0.18)] data-[type=info]:border-primary/40 data-[type=info]:bg-linear-to-b data-[type=info]:from-primary/15 data-[type=info]:to-card/95 data-[type=info]:shadow-[0_0_24px_rgba(121,40,202,0.18)]',
			title: 'font-semibold text-foreground text-sm tracking-tight',
			description: 'text-xs text-muted-foreground/90 leading-relaxed mt-0.5',
			actionButton:
				'rounded-xl bg-primary text-primary-foreground font-semibold text-xs px-3 py-1.5 shadow-sm hover:bg-primary/90 transition-transform active:scale-95',
			cancelButton:
				'rounded-xl bg-muted text-muted-foreground font-medium text-xs px-3 py-1.5 hover:bg-muted/80 transition-colors',
			closeButton:
				'rounded-full border border-border/60 bg-background/80 hover:bg-muted text-muted-foreground hover:text-foreground transition-colors p-1',
			...toastOptions?.classes
		},
		...toastOptions
	}}
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
