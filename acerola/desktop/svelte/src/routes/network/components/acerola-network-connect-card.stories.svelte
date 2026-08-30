<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkConnectCard from './acerola-network-connect-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkConnectCard,
		title: 'Páginas/Network/AcerolaNetworkConnectCard',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Cartão para iniciar a conexão com um peer a partir de um código de pareamento.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkConnectCard
				data={{ connecting: false }}
				events={{ onConnect: async () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Connecting" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkConnectCard
				data={{ connecting: true }}
				events={{ onConnect: async () => {} }}
			/>
		</div>
	{/snippet}
</Story>
