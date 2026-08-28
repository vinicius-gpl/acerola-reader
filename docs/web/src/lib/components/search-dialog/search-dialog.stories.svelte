<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import SearchDialog from './search-dialog.svelte';

	const { Story } = defineMeta({
		title: 'Components/SearchDialog',
		component: SearchDialog,
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
	args={{ open: true }}
	parameters={{
		docs: {
			description: {
				story:
					'Dialogo aberto exibindo o estado de indisponibilidade da busca fora do build de producao.'
			}
		}
	}}
/>
