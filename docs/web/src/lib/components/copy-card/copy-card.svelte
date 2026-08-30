<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';

	let {
		label,
		value,
		href,
		copyLabel = 'Copy',
		copiedLabel = 'Copied'
	}: {
		label: string;
		value: string;
		href?: string;
		copyLabel?: string;
		copiedLabel?: string;
	} = $props();

	let copied = $state(false);
	// Módulo, não closure por instância — se duas CopyCard copiarem quase ao mesmo
	// tempo, o timeout da primeira não pode apagar o estado "copiado" da segunda.
	let resetTimeout: ReturnType<typeof setTimeout> | undefined;

	async function copy() {
		await navigator.clipboard.writeText(value);
		copied = true;
		clearTimeout(resetTimeout);
		resetTimeout = setTimeout(() => (copied = false), 2000);
	}
</script>

<Card.Root class="flex-row items-center justify-between gap-3 py-3">
	<Card.Header class="gap-0.5 px-4">
		<Card.Title class="text-sm font-semibold">{label}</Card.Title>
		<Card.Description>
			{#if href}
				<a {href} class="text-foreground underline underline-offset-4 hover:text-primary">{value}</a
				>
			{:else}
				<span class="text-foreground">{value}</span>
			{/if}
		</Card.Description>
	</Card.Header>
	<Button
		variant="outline"
		size="icon-sm"
		class="mr-4 shrink-0"
		onclick={copy}
		aria-label={copied ? copiedLabel : copyLabel}
	>
		{#if copied}
			<CheckIcon class="text-primary" />
		{:else}
			<CopyIcon />
		{/if}
	</Button>
</Card.Root>
