<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { DocEntry } from '$lib/content/docs';
	import PrevNextNav from './prev-next-nav.svelte';

	function doc(slug: string, title: string): DocEntry {
		return {
			locale: 'pt-br',
			slug,
			component: {} as never,
			frontmatter: { title, section: 'Docs', order: 1 }
		};
	}

	const { Story } = defineMeta({
		title: 'Components/PrevNextNav',
		component: PrevNextNav,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Par de links de navegacao sequencial ao fim de uma pagina de documentacao. Cada lado e omitido quando nao ha vizinho (inicio ou fim da lista de docs).'
				}
			}
		},
		argTypes: {
			prev: { description: 'Documento anterior, ou null se for o primeiro.', control: false },
			next: { description: 'Proximo documento, ou null se for o ultimo.', control: false }
		}
	});
</script>

<Story
	name="Default"
	args={{
		prev: doc('getting-started', 'Primeiros passos'),
		next: doc('architecture', 'Arquitetura')
	}}
	parameters={{
		docs: { description: { story: 'Ambos os vizinhos presentes, caso comum no meio da lista.' } }
	}}
/>

<Story
	name="First Page"
	args={{ prev: null, next: doc('architecture', 'Arquitetura') }}
	parameters={{
		docs: { description: { story: 'Primeira pagina da lista: sem link "Anterior".' } }
	}}
/>

<Story
	name="Last Page"
	args={{ prev: doc('getting-started', 'Primeiros passos'), next: null }}
	parameters={{
		docs: { description: { story: 'Ultima pagina da lista: sem link "Proximo".' } }
	}}
/>
