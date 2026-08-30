<script lang="ts">
	import { setContext } from 'svelte';
	import AcerolaPrevNextNav from '$lib/components/acerola-prev-next-nav/acerola-prev-next-nav.svelte';
	import { DOC_RAW_CONTEXT_KEY } from '$lib/content/doc-raw-context';
	import { m } from '$lib/paraglide/messages';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const Doc = $derived(data.Doc);

	// O Markdown cru não faz parte do frontmatter que o mdsvex repassa automaticamente
	// para o layout (doc-layout.svelte) — só existe aqui, no retorno do load(). Um
	// getter em contexto é o jeito de fazer esse dado atravessar o componente `<Doc />`
	// (gerado pelo mdsvex, fora do nosso controle) até o layout, e continuar lendo o
	// valor atual quando o usuário navega de uma doc para outra sem remontar a página.
	setContext(DOC_RAW_CONTEXT_KEY, {
		get value() {
			return data.raw;
		}
	});
</script>

<svelte:head>
	<title>{data.frontmatter.title} — {m['site.name']()}</title>
	{#if data.frontmatter.description}
		<meta name="description" content={data.frontmatter.description} />
	{/if}
</svelte:head>

<AcerolaPrevNextNav prev={data.prev} next={data.next} position="top" />
<Doc />
<AcerolaPrevNextNav prev={data.prev} next={data.next} />
