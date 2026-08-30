<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaComicActionDialog from './acerola-comic-action-dialog.svelte';
	import type { Category } from '$lib/contracts/bookmarks/bookmarks.payloads';

	const mockBookmarks: Category[] = [
		{ id: 1, name: 'Favoritos', color: 0xff0000 },
		{ id: 2, name: 'Lendo', color: 0x00ff00 },
		{ id: 3, name: 'Completados', color: 0x0000ff }
	];

	const { Story } = defineMeta({
		component: AcerolaComicActionDialog,
		title: 'Páginas/Home/AcerolaComicActionDialog',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Diálogo de ações em lote para quadrinhos selecionados: ocultar, deletar, limpar metadados e marcar em uma categoria.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<AcerolaComicActionDialog
				state={{ open: true }}
				data={{ selectedIds: [1, 2, 3], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => {}
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Single Selection" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<AcerolaComicActionDialog
				state={{ open: true }}
				data={{ selectedIds: [1], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => {}
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Many Selected" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<AcerolaComicActionDialog
				state={{ open: true }}
				data={{ selectedIds: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => {}
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<AcerolaComicActionDialog
				state={{ open: true }}
				data={{ selectedIds: [], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => {}
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="No Bookmarks" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<AcerolaComicActionDialog
				state={{ open: true }}
				data={{ selectedIds: [1, 2], bookmarks: [] }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => {}
				}}
			/>
		</div>
	{/snippet}
</Story>
