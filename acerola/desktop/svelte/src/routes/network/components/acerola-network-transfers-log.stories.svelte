<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkTransfersLog from './acerola-network-transfers-log.svelte';
	import type { TransferLogEntry } from '$lib/hooks/store/use-network-sync.svelte';

	const entries: TransferLogEntry[] = [
		{
			id: 1,
			peerId: 'peer-1',
			kind: 'history',
			status: 'complete',
			message: 'peer-1',
			timestamp: Date.now()
		},
		{
			id: 2,
			peerId: 'peer-2',
			kind: 'files',
			status: 'progress',
			message: 'chapter-012.cbz',
			timestamp: Date.now()
		},
		{
			id: 3,
			peerId: 'peer-3',
			kind: 'comic',
			status: 'error',
			message: 'connection reset',
			timestamp: Date.now()
		}
	];

	const { Story } = defineMeta({
		component: AcerolaNetworkTransfersLog,
		title: 'Components/AcerolaNetworkTransfersLog'
	});
</script>

<Story name="With Entries">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkTransfersLog data={{ entries, peerLabel: () => 'Meu Celular' }} />
		</div>
	{/snippet}
</Story>

<Story name="Empty">
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkTransfersLog data={{ entries: [], peerLabel: () => '' }} />
		</div>
	{/snippet}
</Story>
