<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ReaderShell from './reader-shell.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/ReaderShell',
		component: ReaderShell,
		tags: ['autodocs']
	});
</script>

<Story name="Default" asChild>
	<ReaderShell>
		{#snippet toolbar()}
			<div class="border-b border-surface/40 bg-base p-4 font-bold">Toolbar</div>
		{/snippet}

		{#snippet viewport()}
			<div class="flex flex-1 items-center justify-center bg-mantle/30">Viewport</div>
		{/snippet}

		{#snippet footer()}
			<div class="border-t border-surface/40 bg-base p-4">Footer</div>
		{/snippet}
	</ReaderShell>
</Story>

<Story name="With Command" asChild>
	<ReaderShell>
		{#snippet toolbar()}
			<div class="border-b border-surface/40 bg-base p-4 font-bold">Toolbar</div>
		{/snippet}

		{#snippet viewport()}
			<div class="flex flex-1 items-center justify-center bg-mantle/30">Viewport</div>
		{/snippet}

		{#snippet footer()}
			<div class="border-t border-surface/40 bg-base p-4">Footer</div>
		{/snippet}

		{#snippet command()}
			<div class="absolute inset-0 grid place-items-center bg-base/30">
				<div class="rounded-lg border border-surface/40 bg-base p-6">Command palette</div>
			</div>
		{/snippet}
	</ReaderShell>
</Story>
