<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index';
	import { untrack, type Snippet } from 'svelte';

	// Svelte 5 doesn't allow a dynamic `slot` name, so each tab carries its own
	// snippet instead of relying on named-slot dispatch.
	let { items }: { items: { value: string; label: string; content: Snippet }[] } = $props();

	// Only the initial tab matters here — `items` isn't expected to change after mount.
	let active = $state(untrack(() => items[0]?.value ?? ''));
</script>

<Tabs.Root bind:value={active} class="my-6 overflow-hidden rounded-lg border border-border">
	<Tabs.List
		class="w-full justify-start overflow-x-auto rounded-none border-b border-border bg-muted/40 p-1"
	>
		{#each items as item (item.value)}
			<Tabs.Trigger value={item.value} class="flex-none">{item.label}</Tabs.Trigger>
		{/each}
	</Tabs.List>
	{#each items as item (item.value)}
		<Tabs.Content value={item.value} class="p-4 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0">
			{@render item.content()}
		</Tabs.Content>
	{/each}
</Tabs.Root>
