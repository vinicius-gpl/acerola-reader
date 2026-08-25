<script lang="ts">
	import PrevNextNav from '$lib/components/prev-next-nav/prev-next-nav.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const Doc = $derived(data.Doc);
</script>

<svelte:head>
	<title>{data.frontmatter.title} — {m['site.name']()}</title>
	{#if data.frontmatter.description}
		<meta name="description" content={data.frontmatter.description} />
	{/if}
</svelte:head>

<Doc />
<PrevNextNav prev={data.prev} next={data.next} />
