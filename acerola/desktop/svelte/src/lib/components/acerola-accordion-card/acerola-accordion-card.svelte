<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaAccordionCardProps = {
		data: {
			title: string;
			description?: string;
		};
		state: {
			expanded: boolean;
		};
		events: {
			onToggle: () => void;
		};
		ui?: {
			class?: string;
			/** `false` renderiza a seção sempre aberta, sem botão/chevron/clique — pra misturar,
			 *  numa mesma tela, seções que valem a pena ver de cara com outras densas o bastante
			 *  pra ficarem escondidas por padrão (mesmo espírito da tela de Rede, onde só o card
			 *  de relay colapsa e o resto fica direto na tela). Default `true` (comportamento
			 *  de sempre, sem quebrar nenhum uso existente). */
			collapsible?: boolean;
		};
	};

	export type AcerolaAccordionCardSnippets = {
		icon?: Snippet;
		children?: Snippet;
	};
</script>

<script lang="ts">
	import { slide } from 'svelte/transition';
	import { cn } from '$lib/utils/cn.utils';

	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';

	let {
		data,
		state,
		events,
		ui,
		icon,
		children
	}: AcerolaAccordionCardProps & AcerolaAccordionCardSnippets = $props();

	const collapsible = $derived(ui?.collapsible ?? true);
</script>

{#snippet header()}
	{#if icon}
		<div
			class="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-muted text-foreground transition-colors group-hover:text-primary"
		>
			{@render icon()}
		</div>
	{/if}

	<div class="min-w-0 flex-1">
		<p class="truncate text-lg font-bold text-foreground">{data.title}</p>

		{#if data.description}
			<p class="truncate text-sm text-muted-foreground">{data.description}</p>
		{/if}
	</div>
{/snippet}

<div
	class={cn(
		'overflow-hidden rounded-3xl border border-border bg-card transition-colors',
		ui?.class
	)}
>
	{#if collapsible}
		<button
			type="button"
			aria-expanded={state.expanded}
			onclick={events.onToggle}
			class="group flex w-full min-w-0 cursor-pointer items-center gap-4 p-6 text-left transition-colors"
		>
			{@render header()}

			<ChevronRightIcon
				class={cn(
					'shrink-0 text-muted-foreground transition-transform duration-200',
					state.expanded ? 'rotate-90' : ''
				)}
			/>
		</button>
	{:else}
		<div class="flex w-full min-w-0 items-center gap-4 p-6 text-left">
			{@render header()}
		</div>
	{/if}

	{#if !collapsible}
		<div class="border-t border-border/60 p-4">
			<div class="grid gap-4">
				{@render children?.()}
			</div>
		</div>
	{:else if state.expanded}
		<div transition:slide={{ duration: 200 }} class="border-t border-border/60 p-4">
			<div class="grid gap-4">
				{@render children?.()}
			</div>
		</div>
	{/if}
</div>
