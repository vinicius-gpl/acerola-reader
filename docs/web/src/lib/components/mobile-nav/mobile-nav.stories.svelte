<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { SidebarGroup } from '$lib/content/docs';
	import MobileNav from './mobile-nav.svelte';

	const groups: SidebarGroup[] = [
		{
			section: 'Primeiros passos',
			docs: [
				{
					locale: 'pt-br',
					slug: 'getting-started',
					component: {} as never,
					frontmatter: { title: 'Primeiros passos', section: 'Primeiros passos', order: 1 }
				}
			]
		},
		{
			section: 'Conceitos',
			docs: [
				{
					locale: 'pt-br',
					slug: 'architecture',
					component: {} as never,
					frontmatter: { title: 'Arquitetura', section: 'Conceitos', order: 1 }
				}
			]
		}
	];

	const { Story } = defineMeta({
		title: 'Layout/MobileNav',
		component: MobileNav,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Menu de navegacao em tela cheia (Sheet) usado em telas pequenas, com a mesma `Sidebar` da navegacao lateral desktop e os `NavControls` no rodape.'
				}
			}
		},
		argTypes: {
			open: { description: 'Controla a visibilidade do menu.', control: 'boolean' },
			groups: { description: 'Secoes e documentos a exibir.', control: false },
			activeSlug: { description: 'Slug do documento atualmente aberto.', control: 'text' }
		}
	});
</script>

<Story
	name="Open"
	args={{ open: true, groups, activeSlug: 'architecture' }}
	parameters={{
		docs: { description: { story: 'Menu aberto, exibindo a navegacao completa e os controles.' } }
	}}
/>
