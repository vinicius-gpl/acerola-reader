<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkPairingCard from './acerola-network-pairing-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkPairingCard,
		title: 'Components/AcerolaNetworkPairingCard'
	});
</script>

<Story name="With Code">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkPairingCard data={{ code: 'acerola://connect/abc123' }} />
		</div>
	{/snippet}
</Story>

<Story name="Loading">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkPairingCard data={{ code: undefined }} />
		</div>
	{/snippet}
</Story>
