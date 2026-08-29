<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNotification, { notificationStore } from './acerola-notification.svelte';

	const { Story } = defineMeta({
		title: 'Compositores/AcerolaNotification',
		component: AcerolaNotification,
		tags: ['autodocs']
	});
</script>

<Story name="Vazia">
	<AcerolaNotification />
</Story>

<Story name="Com Notificações">
	{#snippet children()}
		{@const _ = (() => {
			notificationStore.notify.success('Scan concluído!');
			notificationStore.notify.error('Falha ao sincronizar', {
				description: 'Pasta não encontrada'
			});
			notificationStore.notify.warning('Arquivos ignorados', {
				description: '3 arquivos com extensão inválida'
			});
			notificationStore.notify.info('Sincronização disponível');
		})()}
		<AcerolaNotification />
	{/snippet}
</Story>

<Story name="Com Ação">
	{#snippet children()}
		{@const _ = (() => {
			notificationStore.notify.success('Scan concluído!', {
				description: '42 quadrinhos encontrados',
				duration: 0,
				action: { label: 'Ver biblioteca', onClick: () => console.log('navegar') }
			});
		})()}
		<AcerolaNotification />
	{/snippet}
</Story>
