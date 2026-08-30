<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaPeerPicker from './acerola-peer-picker.svelte';
	import type { PairedPeerPayload } from '$lib/contracts/network/network.payloads';

	const peers: PairedPeerPayload[] = [
		{ peerId: 'peer-1', addrs: [1, 2, 3], deviceName: 'Meu Celular' },
		{ peerId: 'peer-2-longid-abcdefghij', addrs: [4, 5, 6], deviceName: null }
	];

	const { Story } = defineMeta({
		component: AcerolaPeerPicker,
		title: 'Compositores/AcerolaPeerPicker',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Diálogo para selecionar um peer pareado da rede, usado antes de ações que dependem de um dispositivo de destino.'
				}
			}
		}
	});
</script>

<Story name="With Peers" asChild>
	{#snippet children()}
		<div class="min-h-[300px] bg-surface p-8">
			<AcerolaPeerPicker
				state={{ open: true }}
				data={{ peers }}
				events={{ onOpenChange: () => {}, onSelect: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty" asChild>
	{#snippet children()}
		<div class="min-h-[300px] bg-surface p-8">
			<AcerolaPeerPicker
				state={{ open: true }}
				data={{ peers: [] }}
				events={{ onOpenChange: () => {}, onSelect: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
