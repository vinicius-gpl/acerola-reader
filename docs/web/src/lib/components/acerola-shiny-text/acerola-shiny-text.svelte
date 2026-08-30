<script lang="ts">
	let {
		text,
		speed = 6,
		class: className = ''
	}: { text: string; speed?: number; class?: string } = $props();
</script>

<span class="shiny-text {className}" style="--shiny-text-duration: {speed}s">
	{text}
</span>

<style>
	/* A band of the theme's destructive/red color sweeps across the text every
	   `--shiny-text-duration`, sitting on top of the normal foreground color —
	   a lightweight stand-in for the reactbits/svelte-bits "ShinyText" effect. */
	.shiny-text {
		background-image: linear-gradient(
			110deg,
			var(--foreground) 35%,
			var(--destructive) 50%,
			var(--foreground) 65%
		);
		background-size: 250% 100%;
		background-clip: text;
		-webkit-background-clip: text;
		color: transparent;
		animation: shiny-text-sweep var(--shiny-text-duration) linear infinite;
	}

	@media (prefers-reduced-motion: reduce) {
		.shiny-text {
			animation: none;
			background-position: 0% 0%;
		}
	}

	@keyframes shiny-text-sweep {
		from {
			background-position: 200% 0;
		}
		to {
			background-position: -50% 0;
		}
	}
</style>
