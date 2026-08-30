<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import CopyMarkdownButton from './copy-markdown-button.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/CopyMarkdownButton',
		component: CopyMarkdownButton,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Botão com ícone e rótulo visível que copia o fonte Markdown de uma página de documentação (mais um comentário com a URL) para a área de transferência, para colar em uma IA e pedir um resumo. Mostra feedback de sucesso ou falha por alguns segundos.'
				}
			}
		},
		argTypes: {
			raw: { description: 'Fonte Markdown completo da página (com frontmatter).', control: 'text' },
			url: {
				description: 'URL canônica da página, incluída como comentário antes do Markdown.',
				control: 'text'
			},
			label: { description: 'Texto acessível do botão antes de copiar.', control: 'text' },
			copiedLabel: { description: 'Texto acessível do botão logo após copiar.', control: 'text' },
			failedLabel: {
				description: 'Texto acessível do botão quando a cópia falha.',
				control: 'text'
			}
		}
	});
</script>

<Story
	name="Default"
	args={{
		raw: '---\ntitle: Primeiros passos\nsection: Primeiros passos\n---\n\n# Primeiros passos\n\nConteúdo de exemplo.',
		url: 'https://docs.acerola-comic.com/docs/getting-started'
	}}
	parameters={{
		docs: {
			description: {
				story: 'Estado padrão, pronto para copiar o Markdown da página atual.'
			}
		}
	}}
/>

<Story
	name="CustomLabels"
	args={{
		raw: '---\ntitle: Contribuindo\n---\n\n# Contribuindo',
		url: 'https://docs.acerola-comic.com/docs/contributing-overview',
		label: 'Copiar como Markdown',
		copiedLabel: 'Copiado'
	}}
	parameters={{
		docs: {
			description: {
				story:
					'Rótulos acessíveis customizados (por exemplo, para uso fora do contexto i18n padrão).'
			}
		}
	}}
/>

<Story
	name="CustomFailedLabel"
	args={{
		raw: '# Doc',
		url: 'https://docs.acerola-comic.com/docs/doc',
		failedLabel: 'Falha ao copiar'
	}}
	parameters={{
		docs: {
			description: {
				story:
					'Rótulo de falha customizado, mostrado quando a área de transferência rejeita a cópia (ex.: contexto não seguro ou permissão negada).'
			}
		}
	}}
/>
