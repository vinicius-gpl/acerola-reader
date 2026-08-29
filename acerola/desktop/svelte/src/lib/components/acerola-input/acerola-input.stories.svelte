<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaInput from './acerola-input.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaInput',
		component: AcerolaInput,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Campo de texto padrão da aplicação.'
				}
			}
		},
		argTypes: {
			state: { description: 'Estado controlado do campo', control: 'object' },
			ui: { description: 'Configuração visual do input', control: 'object' }
		}
	});
</script>

{#snippet template(args: Record<string, unknown>)}
	<AcerolaInput {...args} />
{/snippet}

<Story name="Default" args={{ ui: { placeholder: 'Digite algo...' } }} {template} />

<Story
	name="With Value"
	args={{ state: { value: 'Texto inicial' }, ui: { placeholder: 'Digite algo...' } }}
	{template}
/>

<Story
	name="Search"
	args={{ ui: { type: 'search', placeholder: 'Pesquisar na biblioteca...' } }}
	{template}
/>

<Story
	name="Disabled"
	args={{ ui: { disabled: true, placeholder: 'Campo desabilitado' } }}
	{template}
/>
