<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import type { Component, Snippet } from 'svelte';

	let {
		title,
		href,
		icon,
		children
	}: { title: string; href?: string; icon?: Component; children?: Snippet } = $props();
</script>

{#snippet body()}
	<div class="flex items-center gap-2 font-heading text-lg font-semibold">
		{#if icon}
			{@const Icon = icon}
			<Icon size={20} class="text-primary" />
		{/if}
		{title}
		{#if href}
			<ArrowRightIcon size={16} class="ml-auto text-muted-foreground" />
		{/if}
	</div>
	{#if children}
		<div class="mt-2 text-sm text-muted-foreground [&>p]:my-0">
			{@render children()}
		</div>
	{/if}
{/snippet}

{#if href}
	<a
		{href}
		class="block rounded-lg border border-border bg-card p-5 transition-colors hover:border-primary/50 hover:bg-accent"
	>
		{@render body()}
	</a>
{:else}
	<div class="rounded-lg border border-border bg-card p-5">
		{@render body()}
	</div>
{/if}
