<script lang="ts">
	import { untrack, type Snippet } from 'svelte';

	// Svelte 5 doesn't allow a dynamic `slot` name, so each tab carries its own
	// snippet instead of relying on named-slot dispatch.
	let { items }: { items: { value: string; label: string; content: Snippet }[] } = $props();

	// Only the initial tab matters here — `items` isn't expected to change after mount.
	let active = $state(untrack(() => items[0]?.value));
</script>

<div class="my-6 overflow-hidden rounded-lg border border-border">
	<div class="flex gap-1 overflow-x-auto border-b border-border bg-muted/40 p-1">
		{#each items as item (item.value)}
			<button
				type="button"
				class={[
					'rounded-md px-3 py-1.5 text-sm font-medium whitespace-nowrap transition-colors',
					active === item.value
						? 'bg-background text-foreground shadow-sm'
						: 'text-muted-foreground hover:text-foreground'
				]}
				onclick={() => (active = item.value)}
			>
				{item.label}
			</button>
		{/each}
	</div>
	<div class="p-4 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0">
		{#each items as item (item.value)}
			<div class={active === item.value ? 'block' : 'hidden'}>
				{@render item.content()}
			</div>
		{/each}
	</div>
</div>
