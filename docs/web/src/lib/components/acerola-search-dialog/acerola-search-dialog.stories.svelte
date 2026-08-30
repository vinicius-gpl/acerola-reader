<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSearchDialog from './acerola-search-dialog.svelte';

	// `open: true` como arg inicial (em vez de aberto via interação real) deixa o
	// Dialog.Content do bits-ui preso — a lógica de "presence" que revela o conteúdo só
	// roda numa transição fechado→aberto de verdade, então "nascer" já aberto quebra o
	// preview (e, na aba Docs, o overlay em tela cheia borra a página de documentação
	// inteira atrás dele). Falso por padrão; use os Controls pra abrir interativamente.
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

<Story
	name="Default"
	args={{ open: false }}
	parameters={{
		docs: {
			description: {
				story:
					'Estado fechado por padrão — use o control `open` pra abrir e ver o estado de indisponibilidade da busca fora do build de producao.'
			}
		}
	}}
/>
