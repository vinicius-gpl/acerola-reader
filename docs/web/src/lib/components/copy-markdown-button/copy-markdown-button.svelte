<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import ClipboardCopyIcon from '@lucide/svelte/icons/clipboard-copy';
	import { Button } from '$lib/components/ui/button/index.js';

	let {
		raw,
		url,
		label = 'Copy page as Markdown',
		copiedLabel = 'Copied'
	}: { raw: string; url: string; label?: string; copiedLabel?: string } = $props();

	let copied = $state(false);
	let resetTimeout: ReturnType<typeof setTimeout> | undefined;

	async function copy() {
		await navigator.clipboard.writeText(`<!-- URL: ${url} -->\n\n${raw}`);
		copied = true;
		clearTimeout(resetTimeout);
		resetTimeout = setTimeout(() => (copied = false), 2000);
	}
</script>

<Button
	variant="ghost"
	size="icon-sm"
	class="text-muted-foreground"
	onclick={copy}
	aria-label={copied ? copiedLabel : label}
	title={copied ? copiedLabel : label}
>
	{#if copied}
		<CheckIcon class="text-primary" />
	{:else}
		<ClipboardCopyIcon />
	{/if}
</Button>
