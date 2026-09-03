<script lang="ts">
	import { onMount } from 'svelte';
	import gsap from 'gsap';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let container: HTMLDivElement | undefined = $state();

	onMount(() => {
		const ctx = gsap.context(() => {
			const cards = container?.querySelectorAll('[data-slot="card"]');
			if (!cards?.length) return;

			// Sem deslocamento vertical (`y`): com stagger, um card só termina de subir
			// depois do outro, então por uma fração de segundo eles ficam em alturas
			// diferentes — só o fade evita essa impressão de desalinhamento.
			gsap.from(cards, {
				opacity: 0,
				duration: 0.4,
				stagger: 0.08,
				ease: 'power2.out'
			});
		}, container);

		return () => ctx.revert();
	});
</script>

<div bind:this={container} class="card-grid my-6 flex flex-wrap justify-center gap-4">
	{@render children()}
</div>

<style>
	/* Flexbox instead of CSS grid so a partially-filled last row (e.g. 5 cards in
	   3 columns) centers instead of hugging the left edge.

	   Targets `[data-slot=card]` rather than `> :global(*)`: a linked Card wraps
	   itself in `<a class="group contents">` (`display: contents`), which takes
	   the anchor out of the box tree entirely — the actual flex item the browser
	   lays out is then the Card.Root div one level down, not `.card-grid`'s direct
	   child. A `>` child selector misses it and every card falls back to its
	   default size, which is what was stacking them into a single column instead
	   of wrapping into a grid. */
	.card-grid :global([data-slot='card']) {
		flex: 1 1 100%;
	}

	@media (min-width: 40rem) {
		.card-grid :global([data-slot='card']) {
			flex-basis: calc(50% - 0.5rem);
		}
	}

	@media (min-width: 64rem) {
		.card-grid :global([data-slot='card']) {
			flex-basis: calc(33.333% - 0.667rem);
		}
	}
</style>
