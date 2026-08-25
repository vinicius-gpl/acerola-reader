<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="card-grid my-6 flex flex-wrap justify-center gap-4">
	{@render children()}
</div>

<style>
	/* Flexbox instead of CSS grid so a partially-filled last row (e.g. 5 cards in
	   3 columns) centers instead of hugging the left edge. */
	.card-grid > :global(*) {
		flex: 1 1 100%;
	}

	@media (min-width: 40rem) {
		.card-grid > :global(*) {
			flex-basis: calc(50% - 0.5rem);
		}
	}

	@media (min-width: 64rem) {
		.card-grid > :global(*) {
			flex-basis: calc(33.333% - 0.667rem);
		}
	}
</style>
