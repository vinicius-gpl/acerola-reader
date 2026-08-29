<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ThemePicker from './theme-picker.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/ThemePicker',
		component: ThemePicker,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Botao que alterna o modo de exibicao do tema entre claro, escuro e sistema, ciclando nessa ordem a cada clique. O estado vem do hook `useTheme`, um singleton reativo compartilhado por toda a aplicacao.'
				}
			}
		}
	});
</script>

<Story
	name="Default"
	parameters={{
		docs: {
			description: {
				story: 'Icone reflete o modo atual (sol, lua ou monitor) e alterna ao ser clicado.'
			}
		}
	}}
/>
