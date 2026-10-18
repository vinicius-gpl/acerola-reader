<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="card-grid my-6 flex flex-wrap justify-center gap-4">
	{@render children()}
</div>

<style>
	/* Flexbox instead of CSS grid so a partially-filled last row (e.g. 5 cards in
	   2 or 3 columns) centers instead of hugging the left edge.

	   Targets `[data-slot=card]` rather than `> :global(*)`: a linked Card wraps
	   itself in `<a class="group contents">` (`display: contents`), which takes
	   the anchor out of the box tree entirely — the actual flex item the browser
	   lays out is then the Card.Root div one level down, not `.card-grid`'s direct
	   child. */
	.card-grid :global([data-slot='card']) {
		flex: 1 1 100%;
		min-width: 0;
		max-width: 100%;
		box-sizing: border-box;
	}

	@media (min-width: 42rem) {
		.card-grid :global([data-slot='card']) {
			flex: 1 1 calc(50% - 0.5rem);
			max-width: calc(50% - 0.5rem);
		}
	}
</style>
