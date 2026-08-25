<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="my-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
	{@render children()}
</div>
