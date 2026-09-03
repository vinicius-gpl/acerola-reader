<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSearchDialog from './acerola-search-dialog.svelte';

	// `open: true` como arg inicial (em vez de aberto via interação real) deixa o
	// Dialog.Content do bits-ui preso — a lógica de "presence" que revela o conteúdo só
	// roda numa transição fechado→aberto de verdade, então "nascer" já aberto quebra o
	// preview (e, na aba Docs, o overlay em tela cheia borra a página de documentação
	// inteira atrás dele). Falso por padrão; use o botão da story (ou Ctrl/Cmd+K) pra abrir.
	const { Story } = defineMeta({
		title: 'Compositores/AcerolaSearchDialog',
		component: AcerolaSearchDialog,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Command palette de busca, aberta via `bind:open` ou pelo atalho Ctrl/Cmd+K. O indice Pagefind so existe no build de producao, entao em desenvolvimento (e nesta story) o estado exibido e sempre "indisponivel".'
				}
			}
		}
	});
</script>

<script lang="ts">
	// O Command.Dialog não renderiza nada visível enquanto `open` é falso (é só um portal) —
	// sem um gatilho real na story, a aba Docs mostra um canvas vazio e não dá pra inspecionar
	// o componente sem já saber o atalho de teclado.
	let open = $state(false);
</script>

{#snippet template()}
	<div class="flex min-h-[240px] w-full items-center justify-center">
		<button
			type="button"
			onclick={() => (open = true)}
			class="rounded-lg border border-border bg-surface px-4 py-2 text-sm font-medium text-foreground hover:bg-surface-elevated"
		>
			Abrir busca (Ctrl/Cmd+K)
		</button>
	</div>
	<AcerolaSearchDialog bind:open />
{/snippet}

<Story
	name="Default"
	{template}
	parameters={{
		docs: {
			description: {
				story:
					'Estado fechado por padrão — clique no botão (ou Ctrl/Cmd+K) pra abrir e ver o estado de indisponibilidade da busca fora do build de producao.'
			}
		}
	}}
/>
