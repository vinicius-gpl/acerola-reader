<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import Toc from './toc.svelte';

	const { Story } = defineMeta({
		title: 'Compositores/Toc',
		component: Toc,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Tabela de conteudos (table of contents) gerada a partir dos titulos `h2`/`h3` com `id` encontrados dentro de `containerSelector` (por padrao `.doc-content`). Destaca o titulo atualmente visivel usando um `IntersectionObserver`. Nao renderiza nada quando nenhum titulo e encontrado.'
				}
			}
		},
		argTypes: {
			containerSelector: {
				description: 'Seletor CSS do container onde os titulos sao procurados.',
				control: 'text'
			}
		}
	});
</script>

{#snippet template(args: Record<string, unknown>)}
	<div class="flex gap-8">
		<div class="doc-content max-w-md">
			<h2 id="introducao">Introdução</h2>
			<p>Conteudo de exemplo para demonstrar a tabela de conteudos.</p>
			<h3 id="instalacao">Instalação</h3>
			<p>Mais conteudo de exemplo.</p>
			<h2 id="proximos-passos">Próximos passos</h2>
		</div>
		<Toc {...args} />
	</div>
{/snippet}

<Story
	name="Default"
	args={{}}
	parameters={{
		docs: {
			description: {
				story: 'Titulos de exemplo dentro do container padrao `.doc-content`, renderizado ao lado.'
			}
		}
	}}
	{template}
/>
