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

<script lang="ts">
	// AcerolaRemoteLibraryDialog e totalmente controlado (state.open + events.onOpenChange) e
	// nao tem trigger proprio — cada story precisa do seu proprio estado local e de um botao
	// real pra abrir, senao a aba Docs mostra as 5 variantes com o dialog aberto ao mesmo tempo.
	let openDefault = $state(false);
	let openLoading = $state(false);
	let openError = $state(false);
	let openEmpty = $state(false);
	let openSyncing = $state(false);
</script>

<Story name="Default" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openDefault = true)}
				class="bg-surface-elevated rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openDefault }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics,
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openDefault = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Loading" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openLoading = true)}
				class="bg-surface-elevated rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openLoading }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: true,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openLoading = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Error" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openError = true)}
				class="bg-surface-elevated rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openError }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: false,
					errorMessage: 'Não foi possível consultar a biblioteca remota.',
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openError = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openEmpty = true)}
				class="bg-surface-elevated rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openEmpty }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openEmpty = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Syncing" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openSyncing = true)}
				class="bg-surface-elevated rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openSyncing }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics,
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: (name) => name === 'One Piece'
				}}
				events={{ onOpenChange: (isOpen) => (openSyncing = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
