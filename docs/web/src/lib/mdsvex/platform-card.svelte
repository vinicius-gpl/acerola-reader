<script lang="ts">
	import * as Card from '$lib/components/ui/card/index.js';
	import type { Component, Snippet } from 'svelte';

	let {
		title,
		description,
		icon,
		children
	}: {
		title: string;
		description?: string;
		icon?: Component;
		children?: Snippet;
	} = $props();
</script>

<Card.Root
	class="items-center gap-4 p-6 text-center ring-1 ring-border transition-all hover:-translate-y-0.5 hover:shadow-lg hover:ring-primary/40"
>
	{#if icon}
		{@const Icon = icon}
		<div class="flex h-16 w-16 items-center justify-center rounded-2xl bg-muted">
			<Icon class="h-8 w-auto max-w-10 text-foreground" />
		</div>
	{/if}

	<Card.Header class="w-full items-center gap-1 p-0 text-center">
		<Card.Title class="text-base font-bold">{title}</Card.Title>
		{#if description}
			<Card.Description class="text-sm">{description}</Card.Description>
		{/if}
	</Card.Header>

	{#if children}
		<Card.Content class="w-full p-0">
			{@render children()}
		</Card.Content>
	{/if}
</Card.Root>
