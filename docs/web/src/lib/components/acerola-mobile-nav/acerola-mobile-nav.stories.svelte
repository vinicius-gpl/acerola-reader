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
	// dele). Fechado por padrão; um botão real na story dispara a transição de abertura.
	let open = $state(false);
</script>

<Story
	name="Default"
	args={{ groups, activeSlug: 'architecture' }}
	globals={{ viewport: 'mobile1' }}
	parameters={{
		docs: {
			description: {
				story:
					'Fechado por padrão, numa viewport mobile fixa (o componente e `md:hidden` — sem isso a story so aparece dando zoom out no navegador). Clique no botao pra abrir e ver a navegacao completa e os controles.'
			}
		}
	}}
>
	{#snippet template(args: { groups: SidebarGroup[]; activeSlug: string })}
		<div class="flex min-h-[200px] w-full items-center justify-center">
			<button
				type="button"
				onclick={() => (open = true)}
				class="rounded-lg border border-border bg-surface px-4 py-2 text-sm font-medium text-foreground hover:bg-surface-elevated"
			>
				Abrir menu
			</button>
		</div>
		<AcerolaMobileNav {...args} bind:open />
	{/snippet}
</Story>
