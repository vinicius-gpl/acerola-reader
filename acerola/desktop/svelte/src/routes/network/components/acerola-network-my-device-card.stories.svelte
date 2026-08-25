<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkMyDeviceCard from './acerola-network-my-device-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkMyDeviceCard,
		title: 'Components/AcerolaNetworkMyDeviceCard'
	});
</script>

<Story name="Local Mode">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnopqrstuvwxyz012345',
					mode: 'local',
					activeRelay: null,
					isRelayOverridden: false
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Relay Mode (Custom)">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnopqrstuvwxyz012345',
					mode: 'relay',
					activeRelay: 'relay.acerola.app',
					isRelayOverridden: true
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Loading">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: null,
					localId: null,
					mode: undefined,
					activeRelay: null,
					isRelayOverridden: false
				}}
			/>
		</div>
	{/snippet}
</Story>
