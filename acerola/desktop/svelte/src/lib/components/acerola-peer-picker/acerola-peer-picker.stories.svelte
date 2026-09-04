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

<script lang="ts">
	// AcerolaPeerPicker e totalmente controlado (state.open + events.onOpenChange) e nao tem
	// trigger proprio — cada story precisa do seu proprio estado local e de um botao real pra
	// abrir, senao a aba Docs mostra as variantes com o dialog aberto ao mesmo tempo.
	let openWithPeers = $state(false);
	let openEmpty = $state(false);
</script>

<Story name="With Peers" asChild>
	{#snippet children()}
		<div class="min-h-[300px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openWithPeers = true)}
				class="bg-surface-elevated rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaPeerPicker
				state={{ open: openWithPeers }}
				data={{ peers }}
				events={{ onOpenChange: (isOpen) => (openWithPeers = isOpen), onSelect: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty" asChild>
	{#snippet children()}
		<div class="min-h-[300px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openEmpty = true)}
				class="bg-surface-elevated rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaPeerPicker
				state={{ open: openEmpty }}
				data={{ peers: [] }}
				events={{ onOpenChange: (isOpen) => (openEmpty = isOpen), onSelect: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
