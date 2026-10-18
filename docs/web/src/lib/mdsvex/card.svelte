<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import * as Card from '$lib/components/ui/card/index';
	import { card3D } from '$lib/actions/card-3d';
	import { cn } from '$lib/cn.util';
	import type { Component, Snippet } from 'svelte';

	let {
		title,
		href,
		icon,
		badge,
		class: className,
		children
	}: {
		title: string;
		href?: string;
		icon?: Component;
		badge?: string;
		class?: string;
		children?: Snippet;
	} = $props();

	let cardEl = $state<HTMLDivElement | null>(null);

	$effect(() => {
		if (!cardEl) return;
		const action = card3D(cardEl, { maxRotation: 4, hoverScale: 1.012 });
		return () => action?.destroy?.();
	});

	const isExternal = $derived(href?.startsWith('http') ?? false);
</script>

{#snippet body()}
	<!-- Glowing ambient background highlight -->
	<div
		class="pointer-events-none absolute -top-16 -right-16 size-44 rounded-full bg-primary/10 opacity-30 blur-3xl transition-opacity duration-500 group-hover:bg-primary/20 group-hover:opacity-100"
	></div>

	<Card.Header class="relative z-10 p-0">
		<Card.Title class="flex items-center gap-3">
			{#if icon}
				{@const Icon = icon}
				<div
					class="flex size-10 shrink-0 items-center justify-center rounded-xl border border-primary/30 bg-gradient-to-br from-primary/20 via-primary/10 to-transparent text-primary shadow-xs transition-all duration-300 group-hover:scale-105 group-hover:border-primary/60 group-hover:shadow-[0_0_16px_rgba(203,166,247,0.3)]"
				>
					<Icon size={18} />
				</div>
			{/if}
			<div class="flex min-w-0 flex-1 items-center justify-between gap-2">
				<span
					class="font-heading leading-tight font-bold text-base text-foreground transition-colors group-hover:text-primary"
				>
					{title}
				</span>
				{#if badge}
					<span
						class="shrink-0 rounded-full border border-primary/35 bg-primary/15 px-2 py-0.5 font-mono text-[10px] font-bold tracking-wider whitespace-nowrap text-primary uppercase shadow-xs backdrop-blur-xs transition-colors group-hover:border-primary/60 group-hover:bg-primary/25"
					>
						{badge}
					</span>
				{/if}
			</div>
			{#if href}
				<ArrowRightIcon
					size={16}
					class="shrink-0 text-muted-foreground transition-all duration-300 group-hover:translate-x-1 group-hover:text-primary"
				/>
			{/if}
		</Card.Title>
	</Card.Header>
	{#if children}
		<Card.Content
			class="relative z-10 p-0 text-sm leading-relaxed text-muted-foreground transition-colors group-hover:text-foreground/90 [&>p]:my-0"
		>
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
			bind:ref={cardEl}
			class={cn(
				'group relative box-border w-full max-w-full min-w-0 gap-3 overflow-hidden rounded-2xl border border-border/80 bg-gradient-to-b from-card/95 via-card/80 to-card/50 p-4.5 shadow-xs ring-0 backdrop-blur-md transition-all duration-300 hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-lg hover:ring-1 hover:shadow-primary/5 hover:ring-primary/40',
				className
			)}
		>
			{@render body()}
		</Card.Root>
	</a>
{:else}
	<Card.Root
		bind:ref={cardEl}
		class={cn(
			'group relative box-border w-full max-w-full min-w-0 gap-3 overflow-hidden rounded-2xl border border-border/80 bg-gradient-to-b from-card/95 via-card/80 to-card/50 p-4.5 shadow-xs ring-0 backdrop-blur-md transition-all duration-300 hover:-translate-y-0.5 hover:border-primary/40 hover:shadow-lg hover:shadow-primary/5',
			className
		)}
	>
		{@render body()}
	</Card.Root>
{/if}
