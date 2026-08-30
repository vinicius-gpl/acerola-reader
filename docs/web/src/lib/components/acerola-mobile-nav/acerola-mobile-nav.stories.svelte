<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { SidebarGroup } from '$lib/content/docs';
	import AcerolaMobileNav from './acerola-mobile-nav.svelte';

	const groups: SidebarGroup[] = [
		{
			section: 'Primeiros passos',
			docs: [
				{
					locale: 'pt-br',
					slug: 'getting-started',
					component: {} as never,
					raw: '',
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
					raw: '',
					frontmatter: { title: 'Arquitetura', section: 'Conceitos', order: 1 }
				}
			]
		}
	];

	const { Story } = defineMeta({
		title: 'Layout/AcerolaMobileNav',
		component: AcerolaMobileNav,
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

<script lang="ts">
	// `open: true` como arg inicial faz o Sheet "nascer" já aberto — a lógica de "presence"
	// do bits-ui que revela o conteúdo só roda numa transição fechado→aberto de verdade, então
	// o painel nunca aparece (só o overlay em tela cheia, borrando a aba de Docs inteira atrás
	// dele). Abrir via efeito, um instante após montar, gera essa transição de verdade.
	let open = $state(false);

	$effect(() => {
		const raf = requestAnimationFrame(() => (open = true));
		return () => cancelAnimationFrame(raf);
	});
</script>

<Story
	name="Open"
	args={{ groups, activeSlug: 'architecture' }}
	parameters={{
		docs: { description: { story: 'Menu aberto, exibindo a navegacao completa e os controles.' } }
	}}
>
	{#snippet template(args: { groups: SidebarGroup[]; activeSlug: string })}
		<AcerolaMobileNav {...args} bind:open />
	{/snippet}
</Story>
