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

<script lang="ts">
	// AcerolaDialog nao tem trigger proprio (state.open e totalmente controlado pelo chamador) —
	// fechado por padrao aqui, com um botao real na story pra abrir, senao a aba Docs mostra
	// varias versoes do dialog abertas ao mesmo tempo.
	let open = $state(false);
</script>

<Story
	name="Default"
	args={{
		data: {
			title: 'Dialog Title',
			description: 'Dialog description goes here.'
		}
	}}
	asChild
>
	{#snippet children()}
		<button
			type="button"
			onclick={() => (open = true)}
			class="rounded-lg border border-border bg-surface px-4 py-2 text-sm font-medium text-foreground hover:bg-surface-elevated"
		>
			Open Dialog
		</button>
		<AcerolaDialog
			state={{ open }}
			data={{
				title: 'Dialog Title',
				description: 'Dialog description goes here.'
			}}
			events={{ onOpenChange: (isOpen) => (open = isOpen) }}
		/>
	{/snippet}
</Story>
