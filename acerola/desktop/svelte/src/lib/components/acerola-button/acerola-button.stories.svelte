<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaButton from './acerola-button.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaButton',
		component: AcerolaButton,
		tags: ['autodocs'],
		argTypes: {
			ui: {
				description: 'Variação visual do botão',
				control: 'object'
			}
		}
	});
</script>

<Story name="Default" asChild>
	<AcerolaButton>Clique aqui</AcerolaButton>
</Story>

<Story name="Outline" asChild>
	<AcerolaButton ui={{ variant: 'outline' }}>Outline</AcerolaButton>
</Story>

<Story name="Ghost" asChild>
	<AcerolaButton ui={{ variant: 'ghost' }}>Ghost</AcerolaButton>
</Story>

<Story name="Destructive" asChild>
	<AcerolaButton ui={{ variant: 'destructive' }}>Deletar</AcerolaButton>
</Story>

<Story name="Disabled" asChild>
	<AcerolaButton ui={{ disabled: true }}>Desabilitado</AcerolaButton>
</Story>
