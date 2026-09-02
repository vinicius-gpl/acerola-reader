<script module lang="ts">
	export type AcerolaSectionNavItem = {
		id: string;
		label: string;
	};

	export type AcerolaSectionNavProps = {
		data: {
			sections: AcerolaSectionNavItem[];
		};
		state: {
			activeId: string;
		};
		events: {
			onSelect: (id: string) => void;
		};
		ui?: {
			/// Classe de offset do `sticky top-*` — default `top-0`. Usado quando este nav fica
			/// dentro de um scroll container que já tem outra barra sticky acima dele (ex.: a
			/// aba Conteúdo/Preferências da página do quadrinho), pra não empilhar as duas no
			/// mesmo `top`.
			stickyTop?: string;
		};
	};
</script>

<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import { slidingIndicator } from '$lib/utils/sliding-indicator.utils';
	import { cn } from '$lib/utils/cn.utils';

	let { data, state, events, ui }: AcerolaSectionNavProps = $props();
</script>

<div
	class={cn(
		'sticky z-30 rounded-2xl border border-surface/30 bg-base/80 px-4 py-2 backdrop-blur-xl',
		ui?.stickyTop ?? 'top-0'
	)}
>
	<!--
	Tabs (não ToggleGroup): um ToggleGroup type="single" permite desmarcar o item ativo
	clicando nele de novo, deixando `activeId` vazio e nenhuma seção visível — o primitivo de
	abas do bits-ui nunca fica sem seleção, então clicar na aba já ativa é um no-op.
	-->
	<Tabs.Root value={state.activeId} onValueChange={(value) => events.onSelect(value)}>
		<div
			class="relative inline-flex"
			use:slidingIndicator={{
				selector: '[data-state="active"]',
				indicatorClass: 'rounded-full bg-primary'
			}}
		>
			<Tabs.List class="gap-1">
				{#each data.sections as section (section.id)}
					<Tabs.Trigger
						value={section.id}
						class="relative z-10 cursor-pointer rounded-full px-4 py-1.5 text-xs font-bold tracking-wide text-muted-foreground uppercase transition-colors data-[state=active]:text-primary-foreground"
					>
						{section.label}
					</Tabs.Trigger>
				{/each}
			</Tabs.List>
		</div>
	</Tabs.Root>
</div>
