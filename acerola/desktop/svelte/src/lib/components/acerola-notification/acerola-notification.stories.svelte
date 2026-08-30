<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNotification, { notificationStore } from './acerola-notification.svelte';

	const { Story } = defineMeta({
		title: 'Compositores/AcerolaNotification',
		component: AcerolaNotification,
		tags: ['autodocs']
	});

	// Ações no template (ex.: {@const}) não podem mutar $state — mutar aqui,
	// numa action `use:`, roda após o mount, como um efeito.
	function seedNotifications(_node: HTMLElement) {
		notificationStore.notify.success('Scan concluído!');
		notificationStore.notify.error('Falha ao sincronizar', {
			description: 'Pasta não encontrada'
		});
		notificationStore.notify.warning('Arquivos ignorados', {
			description: '3 arquivos com extensão inválida'
		});
		notificationStore.notify.info('Sincronização disponível');
	}

	function seedActionNotification(_node: HTMLElement) {
		notificationStore.notify.success('Scan concluído!', {
			description: '42 quadrinhos encontrados',
			duration: 0,
			action: { label: 'Ver biblioteca', onClick: () => console.log('navegar') }
		});
	}
</script>

<Story name="Vazia" asChild>
	<AcerolaNotification />
</Story>

<Story name="Com Notificações" asChild>
	{#snippet children()}
		<div use:seedNotifications class="hidden"></div>
		<AcerolaNotification />
	{/snippet}
</Story>

<Story name="Com Ação" asChild>
	{#snippet children()}
		<div use:seedActionNotification class="hidden"></div>
		<AcerolaNotification />
	{/snippet}
</Story>
