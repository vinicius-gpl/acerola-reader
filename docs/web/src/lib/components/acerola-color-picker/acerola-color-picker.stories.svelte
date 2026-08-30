<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaColorPicker from './acerola-color-picker.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaColorPicker',
		component: AcerolaColorPicker,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Menu suspenso para troca da familia de cores do tema (Catppuccin, Nord, Dracula, Tokyo Night). O estado atual vem do hook `useTheme`, um singleton reativo compartilhado por toda a aplicacao.'
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
				story:
					'Botao de gatilho que abre o menu com as familias de cores disponiveis e marca a atualmente selecionada.'
			}
		}
	}}
/>
