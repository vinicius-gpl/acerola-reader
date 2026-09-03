<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import * as Card from '$lib/components/ui/card/index';
	import type { Component, Snippet } from 'svelte';

	let {
		title,
		href,
		icon,
		class: className,
		children
	}: {
		title: string;
		href?: string;
		icon?: Component;
		class?: string;
		children?: Snippet;
	} = $props();

	const isExternal = $derived(href?.startsWith('http') ?? false);
</script>

{#snippet body()}
	<Card.Header>
		<Card.Title class="flex items-center gap-2">
			{#if icon}
				{@const Icon = icon}
				<Icon size={20} class="text-primary" />
			{/if}
			{title}
			{#if href}
				<ArrowRightIcon size={16} class="ml-auto text-muted-foreground" />
			{/if}
		</Card.Title>
	</Card.Header>
	{#if children}
		<Card.Content class="text-sm text-muted-foreground [&>p]:my-0">
			{@render children()}
		</Card.Content>
	{/if}
{/snippet}

{#if href}
	<a
		{href}
		target={isExternal ? '_blank' : undefined}
		rel={isExternal ? 'noopener noreferrer' : undefined}
		class="group contents"
	>
		<Card.Root
			class={['transition-colors group-hover:bg-accent group-hover:ring-primary/50', className]}
		>
			{@render body()}
		</Card.Root>
	</a>
{:else}
	<Card.Root class={className}>
		{@render body()}
	</Card.Root>
{/if}
