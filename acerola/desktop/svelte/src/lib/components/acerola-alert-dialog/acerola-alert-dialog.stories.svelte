<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaAlertDialog from './acerola-alert-dialog.svelte';
	import AcerolaButton from '../acerola-button/acerola-button.svelte';

	const { Story } = defineMeta({
		component: AcerolaAlertDialog,
		title: 'Primitivos/AcerolaAlertDialog',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Diálogo de confirmação com ações de cancelar/confirmar, usado antes de ações destrutivas ou irreversíveis.'
				}
			}
		}
	});
</script>

<Story
	name="Default"
	args={{
		state: { open: false },
		data: {
			title: 'Are you absolutely sure?',
			description:
				'This action cannot be undone. This will permanently delete your account and remove your data from our servers.',
			cancelText: 'Cancel',
			actionText: 'Continue'
		},
		ui: {
			variant: 'default'
		}
	}}
	asChild
>
	{#snippet children()}
		<AcerolaAlertDialog
			data={{
				title: 'Are you absolutely sure?',
				description:
					'This action cannot be undone. This will permanently delete your account and remove your data from our servers.',
				cancelText: 'Cancel',
				actionText: 'Continue'
			}}
			ui={{ variant: 'default' }}
		>
			<AcerolaButton>Open Dialog</AcerolaButton>
		</AcerolaAlertDialog>
	{/snippet}
</Story>

<Story
	name="Destructive"
	args={{
		state: { open: false },
		data: {
			title: 'Delete everything?',
			description: 'This will permanently delete all your files. Are you sure?',
			cancelText: 'Cancel',
			actionText: 'Delete'
		},
		ui: {
			variant: 'destructive'
		}
	}}
	asChild
>
	{#snippet children()}
		<AcerolaAlertDialog
			data={{
				title: 'Delete everything?',
				description: 'This will permanently delete all your files. Are you sure?',
				cancelText: 'Cancel',
				actionText: 'Delete'
			}}
			ui={{ variant: 'destructive' }}
		>
			<AcerolaButton ui={{ variant: 'destructive' }}>Delete All</AcerolaButton>
		</AcerolaAlertDialog>
	{/snippet}
</Story>
