<script module lang="ts">
	import type { Component, Snippet } from 'svelte';

	export type SidebarItem = {
		href: string;
		label: string;
		icon: Component;
	};

	export type AcerolaSidebarProps = {
		data: {
			items: SidebarItem[];
		};
	};

	export type AcerolaSidebarSnippets = {
		brand?: Snippet;
	};
</script>

<script lang="ts">
	import { page } from '$app/state';
	import * as Tooltip from '$lib/components/ui/tooltip/index';
	import { slidingIndicator } from '$lib/utils/sliding-indicator.utils';

	let { data, brand }: AcerolaSidebarProps & AcerolaSidebarSnippets = $props();
</script>

<Tooltip.Provider delayDuration={200}>
	<nav
		aria-label="Navigation"
		use:slidingIndicator={{
			selector: '[aria-current="page"]',
			indicatorClass: 'rounded-xl bg-primary shadow-lg'
		}}
		class="flex w-[4.6rem] shrink-0 flex-col items-center gap-1 border-r border-surface/30 bg-mantle/40 py-4 backdrop-blur-xl"
	>
		{#if brand}
			<div
				class="mb-4 flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-full"
			>
				{@render brand()}
			</div>
		{/if}

		{#each data?.items ?? [] as item (item.href)}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<a
							href={item.href}
							{...props}
							aria-label={item.label}
							aria-current={page.url.pathname === item.href ? 'page' : undefined}
							class="relative flex size-11 shrink-0 items-center justify-center rounded-xl transition-colors [&_svg]:size-6 {page
								.url.pathname === item.href
								? 'text-primary-foreground'
								: 'text-foreground hover:bg-surface/80'}"
						>
							<item.icon />
						</a>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content side="right">{item.label}</Tooltip.Content>
			</Tooltip.Root>
		{/each}
	</nav>
</Tooltip.Provider>
