<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkPeerList, { type DisplayPeer } from './acerola-network-peer-list.svelte';

	const peers: DisplayPeer[] = [
		{ peerId: 'peer-1', deviceName: 'Meu Celular', connected: true },
		{ peerId: 'peer-2', deviceName: 'Notebook do Trabalho', connected: false }
	];

	const { Story } = defineMeta({
		component: AcerolaNetworkPeerList,
		title: 'Páginas/Network/AcerolaNetworkPeerList'
	});
</script>

<Story name="With Peers">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkPeerList
				data={{
					peers,
					addrFor: (id) => (id === 'peer-1' ? [1, 2, 3] : undefined),
					statusLabel: (peer) => (peer.connected ? 'Online' : 'Nunca sincronizado'),
					isSyncing: () => false
				}}
				events={{
					onSyncHistory: () => {},
					onSyncFiles: () => {},
					onSyncAll: () => {},
					onBrowseLibrary: () => {},
					onRemove: () => {}
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkPeerList
				data={{
					peers: [],
					addrFor: () => undefined,
					statusLabel: () => '',
					isSyncing: () => false
				}}
				events={{
					onSyncHistory: () => {},
					onSyncFiles: () => {},
					onSyncAll: () => {},
					onBrowseLibrary: () => {},
					onRemove: () => {}
				}}
			/>
		</div>
	{/snippet}
</Story>
