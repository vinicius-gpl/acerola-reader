<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="doc-steps my-6">
	{@render children()}
</div>

<style>
	.doc-steps :global(ol) {
		list-style: none;
		counter-reset: step;
		margin: 0;
		padding: 0;
	}

	.doc-steps :global(ol > li) {
		counter-increment: step;
		position: relative;
		padding-left: 2.5rem;
		padding-bottom: 1.5rem;
		border-left: 2px solid var(--border);
		margin-left: 0.9rem;
	}

	.doc-steps :global(ol > li:last-child) {
		border-left-color: transparent;
		padding-bottom: 0;
	}

	.doc-steps :global(ol > li::before) {
		content: counter(step);
		position: absolute;
		left: -0.9rem;
		top: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.8rem;
		height: 1.8rem;
		border-radius: 9999px;
		background: var(--color-primary);
		color: var(--color-primary-foreground);
		font-size: 0.75rem;
		font-weight: 600;
	}
</style>
