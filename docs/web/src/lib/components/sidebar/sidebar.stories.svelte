<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { SidebarGroup } from '$lib/content/docs';
	import Sidebar from './sidebar.svelte';

	function doc(slug: string, title: string, section: string): SidebarGroup['docs'][number] {
		return {
			locale: 'pt-br',
			slug,
			component: {} as never,
			frontmatter: { title, section, order: 1 }
		};
	}

	const groups: SidebarGroup[] = [
		{
			section: 'Primeiros passos',
			docs: [doc('getting-started', 'Primeiros passos', 'Primeiros passos')]
		},
		{ section: 'Conceitos', docs: [doc('architecture', 'Arquitetura', 'Conceitos')] }
	];

	const { Story } = defineMeta({
		title: 'Layout/Sidebar',
		component: Sidebar,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Navegacao lateral da documentacao, agrupada por secao. O link cujo `slug` bate com `activeSlug` recebe destaque visual.'
				}
			}
		},
		argTypes: {
			groups: { description: 'Secoes e documentos a exibir.', control: false },
			activeSlug: { description: 'Slug do documento atualmente aberto.', control: 'text' }
		}
	});
</script>

<Story
	name="Default"
	args={{ groups, activeSlug: 'architecture' }}
	parameters={{
		docs: {
			description: {
				story: 'Duas secoes com um documento cada; o item "Arquitetura" esta ativo.'
			}
		}
	}}
/>
