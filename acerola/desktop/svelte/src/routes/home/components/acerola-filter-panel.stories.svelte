<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaFilterPanel from './acerola-filter-panel.svelte';
	import type { Category } from '$lib/contracts/bookmarks/bookmarks.payloads';

	const mockBookmarks: Category[] = [
		{ id: 1, name: 'Favoritos', color: 0xff0000 },
		{ id: 2, name: 'Lendo', color: 0x00ff00 },
		{ id: 3, name: 'Completados', color: 0x0000ff }
	];

	const { Story } = defineMeta({
		component: AcerolaFilterPanel,
		title: 'Páginas/Home/AcerolaFilterPanel'
	});
</script>

<Story name="Default">
	{#snippet children()}
		<div class="min-h-[600px] bg-surface">
			<AcerolaFilterPanel
				state={{ open: true }}
				data={{
					sortBy: 'title',
					sortOrder: 'asc',
					showHidden: false,
					metadataSource: 'all',
					bookmarkFilter: 'all',
					bookmarks: mockBookmarks
				}}
				events={{ onApply: () => {}, onClose: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Active Filters">
	{#snippet children()}
		<div class="min-h-[600px] bg-surface">
			<AcerolaFilterPanel
				state={{ open: true }}
				data={{
					sortBy: 'lastUpdated',
					sortOrder: 'desc',
					showHidden: true,
					metadataSource: 'mangadex',
					bookmarkFilter: 1,
					bookmarks: mockBookmarks
				}}
				events={{ onApply: () => {}, onClose: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="No Bookmarks">
	{#snippet children()}
		<div class="min-h-[600px] bg-surface">
			<AcerolaFilterPanel
				state={{ open: true }}
				data={{
					sortBy: 'title',
					sortOrder: 'asc',
					showHidden: false,
					metadataSource: 'all',
					bookmarkFilter: 'all',
					bookmarks: []
				}}
				events={{ onApply: () => {}, onClose: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Closed">
	{#snippet children()}
		<div class="min-h-[200px] bg-surface p-8 text-sm text-muted-foreground">
			Panel is closed - nothing renders when state.open is false.
			<AcerolaFilterPanel
				state={{ open: false }}
				data={{
					sortBy: 'title',
					sortOrder: 'asc',
					showHidden: false,
					metadataSource: 'all',
					bookmarkFilter: 'all',
					bookmarks: mockBookmarks
				}}
				events={{ onApply: () => {}, onClose: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
