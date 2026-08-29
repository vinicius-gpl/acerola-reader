<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import TopNav from './top-nav.svelte';

	const { Story } = defineMeta({
		title: 'Layout/TopNav',
		component: TopNav,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Cabecalho fixo do topo com o menu mobile, logo, navegacao de docs, gatilho de busca e os controles de idioma/tema. `onOpenSearch`/`onOpenMobileNav` sao callbacks acionados pelos respectivos botoes.'
				}
			}
		},
		argTypes: {
			onOpenSearch: { description: 'Chamado ao clicar em qualquer gatilho de busca.' },
			onOpenMobileNav: { description: 'Chamado ao clicar no botao de menu (mobile).' }
		}
	});
</script>

<Story
	name="Default"
	args={{ onOpenSearch: () => {}, onOpenMobileNav: () => {} }}
	parameters={{
		docs: { description: { story: 'Cabecalho completo como aparece em qualquer pagina de docs.' } }
	}}
/>
