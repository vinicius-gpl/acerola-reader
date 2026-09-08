<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkMyDeviceCard from './acerola-network-my-device-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkMyDeviceCard,
		title: 'Páginas/Network/AcerolaNetworkMyDeviceCard',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Cartão com as informações do dispositivo local: nome, identificador e modo de conexão (local ou relay).'
				}
			}
		}
	});
</script>

<Story name="Local Mode" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnopqrstuvwxyz012345',
					mode: 'local'
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Relay Mode" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnopqrstuvwxyz012345',
					mode: 'relay'
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="With Rename Action" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: 'Meu Notebook',
					localId: 'abcdefghijklmnopqrstuvwxyz012345',
					mode: 'local'
				}}
				events={{ onRenameDevice: async () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Loading" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: null,
					localId: null,
					mode: undefined
				}}
			/>
		</div>
	{/snippet}
</Story>
