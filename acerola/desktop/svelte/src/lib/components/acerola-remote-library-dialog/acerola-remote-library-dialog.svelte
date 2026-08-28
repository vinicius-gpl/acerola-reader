<script module lang="ts">
	import type { ComicSummary } from '$lib/contracts/network/network.payloads';

	export type AcerolaRemoteLibraryDialogProps = {
		state: {
			open: boolean;
		};
		data: {
			peerLabel: string;
			comics: ComicSummary[];
			isLoading: boolean;
			errorMessage?: string;
			coverPathFor: (comicName: string) => string | undefined;
			isSyncing: (comicName: string) => boolean;
		};
		events: {
			onOpenChange: (open: boolean) => void;
			onSelectComic: (comicName: string) => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AcerolaDialog from '$lib/components/acerola-dialog/acerola-dialog.svelte';
	import AcerolaCardImage from '$lib/components/acerola-card/acerola-card-image.svelte';
	import PlaceholderManga from '$lib/assets/placeholder/placeholder_manga.svg?component';
	import SearchIcon from '@lucide/svelte/icons/search';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import CloudDownloadIcon from '@lucide/svelte/icons/cloud-download';

	// Renomeado de `state` na desestruturação — um binding local chamado `state` no mesmo
	// escopo da rune `$state(...)` abaixo colide com o açúcar de auto-subscribe de store do
	// Svelte (`$nome`), e o compilador tenta tratar `state` como uma store.
	let { state: dialogState, data, events }: AcerolaRemoteLibraryDialogProps = $props();

	let searchQuery = $state('');

	// No teardown da story (Storybook + vitest browser mode), o efeito reativo deste
	// template roda mais uma vez com `data`/`events` já undefined antes do componente ser
	// destruído de fato — sem o fallback aqui isso vaza como unhandled error e derruba a
	// suíte mesmo com todos os asserts passando (mesmo padrão de
	// acerola-network-my-device-card.svelte).
	let safeData = $derived(
		data ?? {
			peerLabel: '',
			comics: [],
			isLoading: false,
			errorMessage: undefined,
			coverPathFor: () => undefined,
			isSyncing: () => false
		}
	);
	let safeEvents = $derived(events ?? { onOpenChange: () => {}, onSelectComic: () => {} });

	// O dialog é reaproveitado entre peers diferentes (não remontado a cada abertura) — sem
	// isso, o filtro de uma sessão anterior vazaria pra próxima vez que o usuário abrir pra
	// outro peer.
	$effect(() => {
		if (dialogState?.open) searchQuery = '';
	});

	const filteredComics = $derived.by(() => {
		const query = searchQuery.trim().toLowerCase();
		if (!query) return safeData.comics;
		return safeData.comics.filter((comic) => comic.comicName.toLowerCase().includes(query));
	});
</script>

<AcerolaDialog
	state={{ open: dialogState?.open ?? false }}
	data={{
		title: m['pages.network.remote_library.title'](),
		description: safeData.peerLabel
	}}
	events={{ onOpenChange: safeEvents.onOpenChange }}
	ui={{
		contentClass:
			'w-full max-w-[calc(100vw-2rem)] sm:max-w-2xl border-border/60 bg-background/95 backdrop-blur-xl shadow-2xl p-6 rounded-3xl overflow-hidden'
	}}
>
	<div class="flex w-full min-w-0 flex-col gap-4 py-1">
		<div class="relative w-full">
			<SearchIcon
				size={14}
				class="absolute top-1/2 left-3 shrink-0 -translate-y-1/2 text-muted-foreground"
			/>
			<input
				type="text"
				placeholder={m['pages.network.remote_library.search_placeholder']()}
				bind:value={searchQuery}
				class="h-9 w-full rounded-xl border border-border/60 bg-muted/60 pr-3 pl-8 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary/50 focus:outline-none"
			/>
		</div>

		{#if safeData.isLoading}
			<p class="p-4 text-center text-sm text-muted-foreground">
				{m['pages.network.remote_library.loading']()}
			</p>
		{:else if safeData.errorMessage}
			<p class="p-4 text-center text-sm text-destructive">
				{safeData.errorMessage}
			</p>
		{:else if filteredComics.length === 0}
			<p class="p-4 text-center text-sm text-muted-foreground">
				{m['pages.network.remote_library.empty']()}
			</p>
		{:else}
			<div
				class="grid max-h-[28rem] w-full grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-4 overflow-y-auto p-1"
			>
				{#each filteredComics as comic (comic.comicName)}
					{@const comicSyncing = safeData.isSyncing(comic.comicName)}
					{@const coverPath = safeData.coverPathFor(comic.comicName)}
					<AcerolaCardImage
						data={{
							title: comic.comicName,
							cover: coverPath
						}}
						ui={{ class: 'w-full' }}
						events={{
							onClick: () => {
								if (comicSyncing) return;
								safeEvents.onSelectComic(comic.comicName);
							}
						}}
					>
						{#snippet footer()}
							<span
								class="text-overlay flex items-center gap-1 text-[10px] font-black tracking-wider uppercase"
							>
								<BookOpenIcon size={10} />
								{m['pages.network.remote_library.chapter_count']({ count: comic.chapterCount })}
							</span>
						{/snippet}

						{#snippet placeholder()}
							<div class="h-full w-full bg-surface">
								<PlaceholderManga class="h-full w-full" />
							</div>
						{/snippet}

						{#snippet overlay()}
							{#if comicSyncing}
								<div class="absolute inset-0 flex items-center justify-center bg-crust/60">
									<CloudDownloadIcon size={24} class="animate-spin text-primary" />
								</div>
							{/if}
						{/snippet}
					</AcerolaCardImage>
				{/each}
			</div>
		{/if}
	</div>
</AcerolaDialog>
