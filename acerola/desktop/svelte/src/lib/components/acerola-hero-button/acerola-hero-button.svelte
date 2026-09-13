<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaHeroButtonProps = {
		data?: {
			title?: string;
			description?: string;
		};
		events?: {
			onClick?: (event: MouseEvent) => void;
		};
		ui?: {
			class?: string;
			/** Sobrescreve o fundo/cor padrão (`bg-muted text-foreground`) do círculo do ícone —
			 *  pra casos como um status (sucesso/erro) onde o fundo tintado é o próprio contorno
			 *  colorido do ícone, não um `bg-muted` genérico. */
			iconClass?: string;
		};
	};

	export type AcerolaHeroButtonSnippets = {
		icon?: Snippet;
		action?: Snippet;
		children?: Snippet;
	};
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import * as Item from '$lib/components/ui/item/index';
	import { cn } from '$lib/utils/cn.utils';

	let { data, events, ui, icon, action }: AcerolaHeroButtonProps & AcerolaHeroButtonSnippets =
		$props();
</script>

<Item.Root
	class={cn(
		'group flex min-w-0 items-center justify-between rounded-3xl border border-border bg-card p-6 transition-colors',
		events?.onClick
			? 'cursor-pointer transition-transform hover:border-primary/50 active:scale-[0.98]'
			: '',
		ui?.class
	)}
	onclick={events?.onClick}
>
	<div class="flex min-w-0 flex-1 items-center gap-4">
		{#if icon}
			<Item.Media
				class={cn(
					'flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-muted text-foreground transition-[filter] group-hover:brightness-110',
					ui?.iconClass
				)}
			>
				{@render icon()}
			</Item.Media>
		{/if}

		<Item.Content class="min-w-0 flex-1 text-left">
			<Item.Title class="block w-full truncate text-lg font-bold text-foreground">
				{data?.title ?? m['components.hero_button.default.title']()}
			</Item.Title>

			{#if data?.description}
				<Item.Description class="block w-full truncate text-sm text-muted-foreground"
					>{data.description}</Item.Description
				>
			{/if}
		</Item.Content>
	</div>

	{#if action}
		<Item.Actions class="ml-4 shrink-0">
			{@render action()}
		</Item.Actions>
	{/if}
</Item.Root>
