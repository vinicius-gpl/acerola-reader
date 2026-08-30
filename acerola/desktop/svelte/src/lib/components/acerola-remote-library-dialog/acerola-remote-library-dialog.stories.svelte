<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaRemoteLibraryDialog from './acerola-remote-library-dialog.svelte';
	import type { ComicSummary } from '$lib/contracts/network/network.payloads';

	const comics: ComicSummary[] = [
		{ comicName: 'One Piece', chapterCount: 1090, coverVersion: 1 },
		{ comicName: 'Berserk', chapterCount: 364, coverVersion: 2 },
		{ comicName: 'Vagabond', chapterCount: 327, coverVersion: 1 }
	];

	const { Story } = defineMeta({
		component: AcerolaRemoteLibraryDialog,
		title: 'Compositores/AcerolaRemoteLibraryDialog',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Diálogo que lista a biblioteca de quadrinhos de um peer remoto, com estados de carregamento, erro, vazio e sincronização.'
				}
			}
		}
	});
</script>

<Story name="Default">
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<AcerolaRemoteLibraryDialog
				state={{ open: true }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics,
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: () => {}, onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Loading">
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<AcerolaRemoteLibraryDialog
				state={{ open: true }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: true,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: () => {}, onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Error">
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<AcerolaRemoteLibraryDialog
				state={{ open: true }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: false,
					errorMessage: 'Não foi possível consultar a biblioteca remota.',
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: () => {}, onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty">
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<AcerolaRemoteLibraryDialog
				state={{ open: true }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: () => {}, onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Syncing">
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<AcerolaRemoteLibraryDialog
				state={{ open: true }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics,
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: (name) => name === 'One Piece'
				}}
				events={{ onOpenChange: () => {}, onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
