<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkPairingCard from './acerola-network-pairing-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkPairingCard,
		title: 'Páginas/Network/AcerolaNetworkPairingCard',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Cartão que exibe o código de pareamento do dispositivo para que outro peer se conecte a ele.'
				}
			}
		}
	});
</script>

<Story name="With Code" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkPairingCard data={{ code: 'acerola://connect/abc123' }} />
		</div>
	{/snippet}
</Story>

<Story name="Loading" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkPairingCard data={{ code: undefined }} />
		</div>
	{/snippet}
</Story>
