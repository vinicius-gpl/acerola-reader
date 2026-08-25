<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		description,
		children
	}: { title?: string; description?: string; children: Snippet } = $props();
</script>

<article class="doc-content max-w-none">
	{#if title}
		<h1 class="mb-2 font-heading text-4xl font-semibold">{title}</h1>
	{/if}
	{#if description}
		<p class="mb-8 text-lg text-muted-foreground">{description}</p>
	{/if}
	{@render children()}
</article>
