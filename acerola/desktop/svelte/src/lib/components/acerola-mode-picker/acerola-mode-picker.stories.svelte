<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaModePicker from './acerola-mode-picker.svelte';

	const { Story } = defineMeta({
		title: 'Compositores/AcerolaModePicker',
		component: AcerolaModePicker,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Botão que alterna entre modo claro e escuro. Exibe ícone de lua no modo escuro e sol no modo claro.'
				}
			}
		}
	});
</script>

<Story name="Default" />
