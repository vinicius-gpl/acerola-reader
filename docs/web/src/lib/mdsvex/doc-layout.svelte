<script lang="ts">
	import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
	import type { Snippet } from 'svelte';

	let {
		title,
		description,
		children
	}: { title?: string; description?: string; children: Snippet } = $props();

	const themeCtx = useTheme();
	let container = $state<HTMLElement>();

	// mermaid.run() replaces each node's content with a rendered <svg>, so the
	// original diagram source is captured here the first time through and reused
	// to reset the node before every subsequent re-render (palette/mode changes).
	const sources = new Map<HTMLElement, string>();

	async function renderMermaid() {
		const nodes = container?.querySelectorAll<HTMLElement>('.mermaid');
		if (!nodes || nodes.length === 0) return;

		const { default: mermaid } = await import('mermaid');
		const styles = getComputedStyle(document.documentElement);
		const primaryColor = styles.getPropertyValue('--primary').trim();
		const lineColor = styles.getPropertyValue('--border').trim();
		const textColor = styles.getPropertyValue('--foreground').trim();
		const background = styles.getPropertyValue('--card').trim();

		mermaid.initialize({
			startOnLoad: false,
			theme: themeCtx.resolved === 'dark' ? 'dark' : 'default',
			themeVariables: {
				primaryColor: background,
				primaryTextColor: textColor,
				primaryBorderColor: primaryColor,
				lineColor,
				textColor,
				background
			}
		});

		for (const node of nodes) {
			if (!sources.has(node)) sources.set(node, node.textContent ?? '');
			node.textContent = sources.get(node) ?? '';
			node.removeAttribute('data-processed');
		}

		await mermaid.run({ nodes: [...nodes] });
	}

	$effect(() => {
		// Re-render whenever the active palette or light/dark mode changes.
		themeCtx.resolved;
		themeCtx.theme;
		renderMermaid();
	});
</script>

<article bind:this={container} class="doc-content max-w-none">
	{#if title}
		<h1 class="mb-2 font-heading text-4xl font-semibold">{title}</h1>
	{/if}
	{#if description}
		<p class="mb-8 text-lg text-muted-foreground">{description}</p>
	{/if}
	{@render children()}
</article>
