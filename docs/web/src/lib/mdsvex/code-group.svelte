<script lang="ts">
	import Tabs from './tabs.svelte';
	import type { Snippet } from 'svelte';

	let { items }: { items: { value: string; label: string; content: Snippet }[] } = $props();
</script>

<Tabs {items} />
