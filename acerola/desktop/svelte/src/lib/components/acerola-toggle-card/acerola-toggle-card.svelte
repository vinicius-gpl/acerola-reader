<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaToggleCardProps = {
		data: {
			title: string;
			subtitle?: string;
		};
		state: {
			/** Controla a borda/fundo destacados e o selo de check — "esta fonte/opção está
			 *  ligada agora". Pode ficar sempre `false` num consumidor que só usa o card como uma
			 *  simples "gaveta" (disclosure), sem noção de ligado/desligado (ver uso em
			 *  `acerola-bookmark-manager.svelte`). */
			active: boolean;
			/** Mostra/esconde `children` com uma transição de slide. Independente de `active` —
			 *  um consumidor pode ligar os dois juntos (o conteúdo aparece quando a fonte liga,
			 *  ver o toggle de sync externo em `acerola-comic-preferences.svelte`) ou mantê-los
			 *  separados (expandir só mostra/gerencia algo que já está ativo por outro motivo,
			 *  ver o card de relays próprios). */
			expanded?: boolean;
		};
		events: {
			onClick: () => void;
		};
		ui?: {
			disabled?: boolean;
			class?: string;
		};
	};

	export type AcerolaToggleCardSnippets = {
		icon?: Snippet;
		children?: Snippet;
	};
</script>

<script lang="ts">
	import { slide } from 'svelte/transition';
	import CheckIcon from '@lucide/svelte/icons/check';
	import { cn } from '$lib/utils/cn.utils';
	import { checkBadgePop } from '$lib/utils/check-badge-motion.utils';

	let {
		data,
		state,
		events,
		ui,
		icon,
		children
	}: AcerolaToggleCardProps & AcerolaToggleCardSnippets = $props();
</script>

<div
	class={cn(
		'overflow-hidden rounded-2xl border-2 transition-all duration-200',
		state.active
			? 'border-primary bg-primary/5 shadow-sm shadow-primary/10'
			: 'border-border/60 bg-card hover:border-muted-foreground/50',
		ui?.class
	)}
>
	<button
		type="button"
		onclick={events.onClick}
		disabled={ui?.disabled}
		class="flex w-full items-center gap-3 p-4 text-left enabled:cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
	>
		{#if icon}
			<div
				class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-muted text-foreground"
			>
				{@render icon()}
			</div>
		{/if}
		<div class="min-w-0 flex-1">
			<p class="text-sm font-semibold text-foreground">{data.title}</p>
			{#if data.subtitle}
				<p class="truncate text-xs text-muted-foreground">{data.subtitle}</p>
			{/if}
		</div>
		{#if state.active}
			<div
				use:checkBadgePop
				class="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground"
			>
				<CheckIcon size={12} strokeWidth={3} />
			</div>
		{/if}
	</button>

	{#if state.expanded && children}
		<div transition:slide={{ duration: 200 }} class="space-y-2 border-t border-border/60 p-3">
			{@render children()}
		</div>
	{/if}
</div>
