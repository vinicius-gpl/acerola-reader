<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaToggleGroup from './acerola-toggle-group.svelte';
	import type { AcerolaToggleGroupProps } from './acerola-toggle-group.svelte';
	import { ToggleGroupItem } from '$lib/components/ui/toggle-group/index.js';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaToggleGroup',
		component: AcerolaToggleGroup,
		tags: ['autodocs'],
		parameters: {
			controls: {
				include: ['config', 'ui', 'state']
			}
		},
		argTypes: {
			config: { description: 'Configuração de seleção do grupo', control: 'object' },
			ui: { description: 'Configuração visual do grupo', control: 'object' },
			state: { description: 'Valor selecionado', control: 'object' }
		}
	});
</script>

{#snippet template(args: AcerolaToggleGroupProps)}
	<AcerolaToggleGroup {...args}>
		{#snippet children()}
			<ToggleGroupItem value="a">Opcao A</ToggleGroupItem>
			<ToggleGroupItem value="b">Opcao B</ToggleGroupItem>
			<ToggleGroupItem value="c">Opcao C</ToggleGroupItem>
		{/snippet}
	</AcerolaToggleGroup>
{/snippet}

<Story
	name="Single Default"
	args={{
		config: { type: 'single' },
		ui: { variant: 'default', size: 'default' },
		state: { value: 'a' }
	}}
	{template}
/>

<Story
	name="Single Outline"
	args={{
		config: { type: 'single' },
		ui: { variant: 'outline', size: 'default' },
		state: { value: 'b' }
	}}
	{template}
/>

<Story
	name="Multiple Default"
	args={{
		config: { type: 'multiple' },
		ui: { variant: 'default', size: 'default' },
		state: { value: ['a', 'c'] }
	}}
	{template}
/>

<Story
	name="Multiple Outline"
	args={{
		config: { type: 'multiple' },
		ui: { variant: 'outline', size: 'default' },
		state: { value: ['b'] }
	}}
	{template}
/>

<Story
	name="Size Small"
	args={{
		config: { type: 'single' },
		ui: { variant: 'default', size: 'sm' },
		state: { value: 'a' }
	}}
	{template}
/>

<Story
	name="Size Large"
	args={{
		config: { type: 'single' },
		ui: { variant: 'default', size: 'lg' },
		state: { value: 'c' }
	}}
	{template}
/>
