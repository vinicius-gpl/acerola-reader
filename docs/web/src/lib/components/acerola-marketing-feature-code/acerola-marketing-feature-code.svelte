<script module lang="ts">
	import type { Component } from 'svelte';

	export type FeatureItem = {
		icon?: Component;
		title: string;
		description: string;
	};

	export type CodeTab = {
		id: string;
		label: string;
		filename?: string;
		code: string;
	};

	export type MarketingFeatureCodeProps = {
		eyebrow?: string;
		title: string;
		description?: string;
		features: FeatureItem[];
		tabs: CodeTab[];
		class?: string;
	};
</script>

<script lang="ts">
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import TerminalIcon from '@lucide/svelte/icons/terminal';
	import { cn } from '$lib/cn.util';
	import { m } from '$lib/paraglide/messages';

	let {
		eyebrow,
		title,
		description,
		features = [],
		tabs = [],
		class: className
	}: MarketingFeatureCodeProps = $props();

	let selectedTabId = $state<string | null>(null);
	let activeTabId = $derived(
		selectedTabId && tabs.some((t) => t.id === selectedTabId) ? selectedTabId : (tabs[0]?.id ?? '')
	);

	let currentTab = $derived(tabs.find((t) => t.id === activeTabId) ?? tabs[0]);
	let copied = $state(false);
	let copyTimeout: ReturnType<typeof setTimeout> | undefined;

	async function handleCopy() {
		if (!currentTab?.code || copied) return;
		try {
			await navigator.clipboard.writeText(currentTab.code);
			copied = true;
			clearTimeout(copyTimeout);
			copyTimeout = setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy', err);
		}
	}
</script>

<section
	class={cn(
		'relative w-full overflow-hidden rounded-3xl border border-border/70 bg-gradient-to-b from-card/80 via-card/50 to-background/90 p-6 shadow-2xl backdrop-blur-xl sm:p-10',
		className
	)}
>
	<!-- Ambient top gradient line -->
	<div
		class="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent"
	></div>

	<!-- Header Area -->
	<div class="mb-10 text-center sm:text-left">
		{#if eyebrow}
			<div
				class="mb-3 inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-3.5 py-1 text-xs font-semibold text-primary"
			>
				<TerminalIcon size={13} />
				<span>{eyebrow}</span>
			</div>
		{/if}
		<h2 class="text-2xl font-black tracking-tight text-foreground sm:text-3xl lg:text-4xl">
			{title}
		</h2>
		{#if description}
			<p class="mt-2.5 max-w-2xl text-sm leading-relaxed text-muted-foreground sm:text-base">
				{description}
			</p>
		{/if}
	</div>

	<!-- Split Grid: Features on left, Code panel on right -->
	<div class="grid grid-cols-1 items-start gap-8 lg:grid-cols-12 lg:gap-10">
		<!-- Features List (Left) -->
		<div class="space-y-6 lg:col-span-5">
			{#each features as feat}
				<div class="group flex items-start gap-4">
					<div
						class="flex size-11 shrink-0 items-center justify-center rounded-2xl border border-border/80 bg-background/80 text-primary shadow-xs transition-transform duration-200 group-hover:scale-105 group-hover:border-primary/40 group-hover:shadow-[0_0_16px_rgba(var(--color-primary-rgb),0.2)]"
					>
						{#if feat.icon}
							<feat.icon size={20} />
						{:else}
							<TerminalIcon size={20} />
						{/if}
					</div>
					<div class="min-w-0 space-y-1 pt-0.5">
						<h3 class="text-sm font-bold tracking-tight text-foreground">
							{feat.title}
						</h3>
						<p class="text-xs leading-relaxed text-muted-foreground">
							{feat.description}
						</p>
					</div>
				</div>
			{/each}
		</div>

		<!-- Code Card Showcase (Right - Cult UI style) -->
		<div
			class="relative flex flex-col overflow-hidden rounded-2xl border border-border/80 bg-background/95 shadow-xl lg:col-span-7"
		>
			<!-- Top Tab Bar -->
			<div
				class="flex items-center justify-between border-b border-border/70 bg-muted/30 px-3 py-2 text-xs backdrop-blur-sm"
			>
				<!-- Window mock circles + Tabs -->
				<div class="flex items-center gap-3">
					<div class="flex items-center gap-1.5 pl-1.5">
						<span class="size-2.5 rounded-full bg-rose-500/80"></span>
						<span class="size-2.5 rounded-full bg-amber-500/80"></span>
						<span class="size-2.5 rounded-full bg-emerald-500/80"></span>
					</div>

					<div class="flex items-center gap-1 border-l border-border/60 pl-3">
						{#each tabs as tab}
							{@const isActive = tab.id === activeTabId}
							<button
								type="button"
								onclick={() => (selectedTabId = tab.id)}
								class={cn(
									'rounded-lg px-2.5 py-1 font-mono text-xs font-medium transition-all',
									isActive
										? 'border border-border/60 bg-card text-foreground shadow-xs'
										: 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
								)}
							>
								{tab.label}
							</button>
						{/each}
					</div>
				</div>

				<!-- Copy button with Cult UI check feedback -->
				<button
					type="button"
					onclick={handleCopy}
					aria-label={copied ? m['marketing_code.copied_aria']() : m['marketing_code.copy_aria']()}
					class={cn(
						'inline-flex items-center gap-1.5 rounded-lg border px-2.5 py-1 font-mono text-xs font-medium transition-all active:scale-95',
						copied
							? 'border-emerald-500/50 bg-emerald-500/10 text-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.25)]'
							: 'border-border/60 bg-card/60 text-muted-foreground hover:border-border hover:bg-card hover:text-foreground'
					)}
				>
					{#if copied}
						<CheckIcon size={12} strokeWidth={2.5} class="animate-in zoom-in-50" />
						<span>{m['marketing_code.copied']()}</span>
					{:else}
						<CopyIcon size={12} />
						<span>{m['marketing_code.copy']()}</span>
					{/if}
				</button>
			</div>

			<!-- Code Body -->
			<div
				class="max-h-[380px] overflow-x-auto overflow-y-auto p-4 font-mono text-xs leading-relaxed sm:p-5"
			>
				<pre class="whitespace-pre text-foreground/90"><code>{currentTab?.code ?? ''}</code></pre>
			</div>
		</div>
	</div>
</section>
