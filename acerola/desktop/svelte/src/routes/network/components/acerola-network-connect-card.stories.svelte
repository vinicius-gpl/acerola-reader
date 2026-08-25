<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkConnectCard from './acerola-network-connect-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkConnectCard,
		title: 'Components/AcerolaNetworkConnectCard'
	});
</script>

<Story name="Default">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkConnectCard
				data={{ connecting: false }}
				events={{ onConnect: async () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Connecting">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkConnectCard
				data={{ connecting: true }}
				events={{ onConnect: async () => {} }}
			/>
		</div>
	{/snippet}
</Story>
