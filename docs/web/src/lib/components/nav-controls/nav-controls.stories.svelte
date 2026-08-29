<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import NavControls from './nav-controls.svelte';

	const { Story } = defineMeta({
		title: 'Compositores/NavControls',
		component: NavControls,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Grupo de controles do cabecalho: alternancia de idioma, selecao de modo do tema, paleta de cores e link para o GitHub do projeto.'
				}
			}
		}
	});
</script>

<Story
	name="Default"
	parameters={{
		docs: { description: { story: 'Uso padrao dentro do cabecalho (top-nav) ou do menu mobile.' } }
	}}
/>
