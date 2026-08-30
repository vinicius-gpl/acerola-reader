<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type ReaderShellProps = {
		toolbar: Snippet;
		viewport: Snippet;
		footer: Snippet;
		command?: Snippet;
	};
</script>

<script lang="ts">
	let { toolbar, viewport, footer, command }: ReaderShellProps = $props();
</script>

<div class="text-text fixed inset-x-0 top-8 bottom-0 z-50 flex flex-col overflow-hidden bg-base">
	{@render toolbar()}
	{@render viewport()}
	{@render footer()}

	{#if command}
		{@render command()}
	{/if}
</div>
