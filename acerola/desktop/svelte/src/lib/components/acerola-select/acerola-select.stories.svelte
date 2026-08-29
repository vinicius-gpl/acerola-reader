<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSelect from './acerola-select.svelte';
	import type { AcerolaSelectOption } from './acerola-select.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaSelect',
		component: AcerolaSelect,
		tags: ['autodocs'],
		argTypes: {
			data: {
				description: 'Array de opcoes ({ value, label })',
				control: 'object'
			},
			state: { description: 'Valor selecionado atualmente', control: 'object' },
			ui: { description: 'Configuração visual e placeholder', control: 'object' }
		}
	});

	const defaultOptions: AcerolaSelectOption[] = [
		{ value: 'manga', label: 'Manga' },
		{ value: 'webtoon', label: 'Webtoon' },
		{ value: 'comic', label: 'Comic Book' }
	];
</script>

<Story
	name="Default"
	args={{
		data: { options: defaultOptions },
		ui: { placeholder: 'Selecione o tipo...' }
	}}
/>

<Story
	name="Com valor predefinido"
	args={{
		data: { options: defaultOptions },
		state: { value: 'manga' }
	}}
/>

<Story
	name="Placeholder customizado"
	args={{
		data: { options: defaultOptions },
		ui: { placeholder: 'Escolha um formato' }
	}}
/>
