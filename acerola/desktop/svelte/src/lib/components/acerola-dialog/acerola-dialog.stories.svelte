<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaDialog from './acerola-dialog.svelte';

	const { Story } = defineMeta({
		component: AcerolaDialog,
		title: 'Primitivos/AcerolaDialog',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Diálogo modal genérico com título e descrição, base para composições mais específicas.'
				}
			}
		}
	});
</script>

<Story
	name="Default"
	args={{
		state: { open: true },
		data: {
			title: 'Dialog Title',
			description: 'Dialog description goes here.'
		}
	}}
>
	{#snippet children()}
		<AcerolaDialog
			state={{ open: true }}
			data={{
				title: 'Dialog Title',
				description: 'Dialog description goes here.'
			}}
		/>
	{/snippet}
</Story>
