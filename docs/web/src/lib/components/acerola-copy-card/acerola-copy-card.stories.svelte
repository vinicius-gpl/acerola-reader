<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaCopyCard from './acerola-copy-card.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaCopyCard',
		component: AcerolaCopyCard,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Card compacto para um valor curto que o usuário provavelmente vai querer copiar (e-mail, ID, endpoint), com botão de copiar dedicado em vez de depender de selecionar o texto manualmente.'
				}
			}
		},
		argTypes: {
			label: { description: 'Rótulo acima do valor.', control: 'text' },
			value: {
				description: 'Valor exibido e copiado para a área de transferência.',
				control: 'text'
			},
			href: { description: 'Se definido, o valor vira um link (ex.: mailto:).', control: 'text' },
			copyLabel: { description: 'Texto acessível do botão antes de copiar.', control: 'text' },
			copiedLabel: { description: 'Texto acessível do botão logo após copiar.', control: 'text' }
		}
	});
</script>

<Story
	name="Default"
	args={{ label: 'Contato', value: 'contato@acerola-comic.com' }}
	parameters={{
		docs: { description: { story: 'Valor exibido como texto simples, com botão de copiar.' } }
	}}
/>

<Story
	name="WithLink"
	args={{
		label: 'Contato',
		value: 'contato@acerola-comic.com',
		href: 'mailto:contato@acerola-comic.com'
	}}
	parameters={{
		docs: {
			description: {
				story: 'O valor também é um link clicável (ex.: `mailto:`) além de copiável.'
			}
		}
	}}
/>
