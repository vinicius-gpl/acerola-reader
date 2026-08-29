<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import * as Command from '$lib/components/ui/command/index.js';
	import AcerolaCommand from './acerola-command.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaCommand',
		component: AcerolaCommand,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Container para interfaces no estilo command palette. O wrapper recebe `children` como snippet e organiza estado/eventos em `state` e `events`, sendo adequado para busca local, navegacao rapida, menus acionaveis por teclado e selecao assistida por filtro.'
				}
			},
			controls: {
				include: ['state']
			}
		},
		argTypes: {
			state: {
				description: 'Texto atual usado pelo input de busca do command.',
				control: 'object'
			},
			events: {
				description: 'Callback disparado quando o texto de busca e alterado.',
				control: 'object'
			},
			children: {
				description: 'Snippet com a composicao interna do command, como input, lista e itens.',
				control: false
			}
		}
	});
</script>

{#snippet template(args: Record<string, unknown>)}
	<div class="w-full max-w-md">
		<AcerolaCommand {...args}>
			{#snippet children()}
				<Command.Input placeholder="Pesquisar..." />
				<Command.List>
					<Command.Item value="item1">Item 1</Command.Item>
					<Command.Item value="item2">Item 2</Command.Item>
					<Command.Item value="item3">Item 3</Command.Item>
				</Command.List>
			{/snippet}
		</AcerolaCommand>
	</div>
{/snippet}

<Story
	name="Default"
	args={{ state: { value: '' } }}
	parameters={{
		docs: {
			description: {
				story:
					'Exemplo base de uma command palette com campo de busca vazio e uma lista curta de itens. Serve como ponto de partida para cenarios de busca local e navegacao interna.'
			}
		}
	}}
	{template}
/>

<Story
	name="With Search Value"
	args={{ state: { value: 'item' } }}
	parameters={{
		docs: {
			description: {
				story:
					'Exemplo com valor inicial preenchido. E util para demonstrar estado controlado, restauracao de filtro e telas em que a busca ja chega preconfigurada a partir de contexto anterior.'
			}
		}
	}}
	{template}
/>
