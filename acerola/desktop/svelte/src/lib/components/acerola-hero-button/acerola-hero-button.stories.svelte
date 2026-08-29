<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaHeroButton from './acerola-hero-button.svelte';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import PlayIcon from '@lucide/svelte/icons/play';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaHeroButton',
		component: AcerolaHeroButton,
		tags: ['autodocs'],
		args: {
			data: {
				title: 'Título Padrão',
				description: 'Descrição padrão do componente'
			}
		},
		argTypes: {
			data: { description: 'Conteúdo principal do botão', control: 'object' },
			events: { description: 'Callbacks do botão', control: 'object' },
			ui: { description: 'Configuração visual do botão', control: 'object' },
			icon: { description: 'Snippet para renderizar um ícone à esquerda', control: 'object' },
			action: {
				description: 'Snippet para renderizar um componente de ação à direita',
				control: 'object'
			}
		}
	});
</script>

{#snippet folderIcon()}
	<FolderIcon size={24} />
{/snippet}

{#snippet actionButton()}
	<div
		class="rounded-full bg-primary/10 p-2 text-primary transition-all group-hover:bg-primary group-hover:text-primary-foreground"
	>
		<PlayIcon size={20} />
	</div>
{/snippet}

<Story
	name="Completo"
	args={{
		data: {
			title: 'Pasta dos Mangás',
			description: 'C:/Users/acerola/comics'
		},
		icon: folderIcon,
		action: actionButton,
		events: { onClick: () => console.log('Clicou!') }
	}}
/>

<Story
	name="Apenas Texto"
	args={{
		data: {
			title: 'Título de Exemplo',
			description: 'Esta descrição deve aparecer agora.'
		}
	}}
/>

<Story name="Uso Manual">
	<AcerolaHeroButton
		data={{
			title: 'Uso Manual',
			description: 'Passando props diretamente no componente'
		}}
	>
		{#snippet icon()}
			{@render folderIcon()}
		{/snippet}

		{#snippet action()}
			{@render actionButton()}
		{/snippet}
	</AcerolaHeroButton>
</Story>
