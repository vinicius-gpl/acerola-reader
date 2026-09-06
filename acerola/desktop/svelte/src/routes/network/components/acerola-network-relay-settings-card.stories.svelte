<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkRelaySettingsCard from './acerola-network-relay-settings-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkRelaySettingsCard,
		title: 'Páginas/Network/AcerolaNetworkRelaySettingsCard',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Configuração combinável de relay: relay do Acerola, Iroh Services (conta própria, autenticada por ticket) e lista de relays próprios cadastrados pelo usuário.'
				}
			}
		}
	});

	const noop = () => {};
	const noopAsync = async () => {};
	const events = {
		onToggleAcerolaRelay: noop,
		onToggleIrohPublicNetwork: noop,
		onAddCustomRelayUrl: noop,
		onRemoveCustomRelayUrl: noop,
		onSetIrohServicesTicket: noopAsync,
		onClearIrohServicesTicket: noopAsync,
		onRestart: noopAsync
	};
</script>

<Story name="Default (Acerola Relay Only)" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola-comic.com',
					useAcerolaRelay: true,
					useIrohPublicNetwork: false,
					customRelayUrls: [],
					hasIrohServicesTicket: false
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Multiple Sources Combined" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola-comic.com',
					useAcerolaRelay: true,
					useIrohPublicNetwork: false,
					customRelayUrls: ['https://relay.example.com'],
					hasIrohServicesTicket: false
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Iroh Services (Exclusive, Ticket Configured)" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola-comic.com',
					useAcerolaRelay: true,
					useIrohPublicNetwork: true,
					customRelayUrls: [],
					hasIrohServicesTicket: true
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>

<Story name="mDNS Only (Nothing Active)" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola-comic.com',
					useAcerolaRelay: false,
					useIrohPublicNetwork: false,
					customRelayUrls: [],
					hasIrohServicesTicket: false
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>
