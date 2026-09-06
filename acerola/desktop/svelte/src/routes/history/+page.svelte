<script lang="ts">
	import { goto } from '$app/navigation';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { toast } from 'svelte-sonner';

	import PlaceholderManga from '$lib/assets/placeholder/placeholder_manga.svg?component';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaBookmarkRibbon from '$lib/components/acerola-bookmark-ribbon/acerola-bookmark-ribbon.svelte';
	import AcerolaAlertDialog from '$lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaCardImage from '$lib/components/acerola-card/acerola-card-image.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import { buttonVariants } from '$lib/components/ui/button';
	import { cn } from '$lib/utils/cn.utils';
	import { useBookmarks } from '$lib/hooks/store/use-bookmarks.svelte';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import MonitorIcon from '@lucide/svelte/icons/monitor';

	import { resolveArtworkPath } from '$lib/utils/artwork.utils';
	import type { ReadingHistoryPayload } from '$lib/contracts/history/history.payloads';
	import { useHistory } from '$lib/hooks/store/use-history.svelte';
	import { usePeerConnection, shortId } from '$lib/hooks/store/use-peer-connection.svelte';
	import type { useNetworkSync } from '$lib/hooks/store/use-network-sync.svelte';
	import { CONTEXT_KEYS } from '$lib/constants/context-keys';

	const history = useHistory();
	const bookmarkStore = useBookmarks();
	const peers = usePeerConnection();
	// Compartilhada com `+layout.svelte` (nunca desmonta) via contexto — não cria uma instância
	// própria. Ver `CONTEXT_KEYS.networkSync` / `routes/network/+page.svelte` pro porquê: uma
	// instância só-desta-página, desmontada ao navegar pra outra tela, rejeitava promises de
	// sync em andamento (`sync.syncHistory`) com "sync cancelled: listener stopped" mesmo o
	// sync de verdade continuando no backend.
	const sync = getContext<ReturnType<typeof useNetworkSync>>(CONTEXT_KEYS.networkSync);

	let syncMenuOpen = $state(false);
	// Evita re-disparar `history.fetch()`/toast duas vezes pro mesmo evento — `sync.log[0]`
	// dispara o `$effect` de novo a cada re-render enquanto essa entrada continuar sendo a mais
	// recente do log.
	let lastHandledSyncLogId: number | undefined;

	type DisplayPeer = { peerId: string; deviceName: string | null; connected: boolean };

	// Mesmo merge de pareados persistidos + conectados ao vivo usado em `/network` — ver o
	// comentário lá para o motivo de precisar dos dois.
	const uniquePeers = $derived.by(() => {
		const byId = new Map<string, DisplayPeer>();
		for (const peer of peers.pairedPeers) {
			byId.set(peer.peerId, { peerId: peer.peerId, deviceName: peer.deviceName, connected: false });
		}

		for (const peer of peers.status?.peers ?? []) {
			byId.set(peer.peerId, {
				peerId: peer.peerId,
				deviceName: peer.device?.name ?? byId.get(peer.peerId)?.deviceName ?? null,
				connected: true
			});
		}
		return [...byId.values()];
	});

	const anyHistorySyncing = $derived(
		uniquePeers.some((peer) => sync.isSyncing(peer.peerId, 'history'))
	);

	function peerStatusLabel(peer: DisplayPeer): string {
		if (peer.connected) return m['pages.network.peers.online']();
		const timestamp = sync.lastSyncedAt(peer.peerId);

		if (!timestamp) return m['pages.network.peers.never_synced']();
		return m['pages.network.peers.last_synced']({ when: new Date(timestamp).toLocaleString() });
	}

	async function handleSyncHistory(peerId: string) {
		const addrs = peers.getKnownAddr(peerId);
		if (!addrs) return;
		try {
			await sync.syncHistory(peerId, addrs);
		} catch (err) {
			toast.error(String(err));
		}
	}

	// `sync.syncHistory` só resolve quando a conexão é aberta, não quando a sessão termina de
	// fato (ver `sync:history:*` no backend) — o resultado real chega aqui via `sync.log`,
	// alimentado pelos mesmos eventos que a tela de Rede escuta.
	$effect(() => {
		const entry = sync.log[0];
		// `id < 0` marca uma linha carregada do histórico persistido (ver `fromPersisted` em
		// `use-network-sync.svelte.ts`), não um evento ao vivo desta sessão — sem esse guard, a
		// linha "complete" mais recente do histórico disparava este toast assim que a tela de
		// Histórico montava, mesmo sem nenhum sync ter de fato acontecido agora.
		if (!entry || entry.id < 0 || entry.kind !== 'history' || entry.id === lastHandledSyncLogId)
			return;

		if (entry.status === 'complete') {
			lastHandledSyncLogId = entry.id;
			toast.success(m['pages.history.sync.success']({ peer: peers.peerLabel(entry.message) }));
			history.fetch();
			return;
		}

		if (entry.status === 'error') {
			lastHandledSyncLogId = entry.id;
			toast.error(m['pages.history.sync.error']({ msg: entry.message }));
			return;
		}
	});

	function resumeReading(item: ReadingHistoryPayload) {
		goto('/reader', {
			state: {
				comicDirectoryId: item.comicDirectoryId,
				startPage: item.lastPage,
				chapterScope: item.comicName,
				chapter: {
					id: item.chapterArchiveId,
					name: item.chapterName,
					path: item.chapterPath,
					chapterSort: item.chapterSort,
					isSpecial: item.isSpecial,
					lastModified: item.lastModified,
					volumeId: null,
					volumeName: null
				}
			}
		});
	}

	function openComic(item: ReadingHistoryPayload) {
		goto(`/comic/${item.folderName}`, {
			state: {
				comicDirectoryId: item.comicDirectoryId
			}
		});
	}

	onMount(() => {
		history.fetch();
		peers.startListening();
	});

	onDestroy(() => {
		peers.stopListening();
	});
</script>

<div class="flex h-full flex-col overflow-hidden">
	<div class="flex shrink-0 items-center justify-between px-8 py-6">
		<h2 class="text-2xl font-bold tracking-tight">{m['pages.history.title']()}</h2>
		<div class="flex items-center gap-2">
			<AcerolaPopover
				state={{ open: syncMenuOpen }}
				events={{ onOpenChange: (open) => (syncMenuOpen = open) }}
				ui={{
					align: 'end',
					contentClass:
						'w-72 overflow-hidden rounded-2xl border-border/40 bg-card/95 p-1.5 shadow-2xl backdrop-blur-md'
				}}
			>
				{#snippet trigger()}
					<span
						class={cn(
							buttonVariants({ variant: 'outline', size: 'sm' }),
							'gap-2 font-medium tracking-wide'
						)}
					>
						<RefreshCwIcon size={14} class={anyHistorySyncing ? 'animate-spin' : ''} />
						{m['pages.history.sync.button']()}
					</span>
				{/snippet}

				{#snippet content()}
					<div class="flex flex-col gap-0.5">
						<p
							class="px-2.5 pt-1.5 pb-2 text-xs font-bold tracking-widest text-muted-foreground uppercase"
						>
							{m['pages.history.sync.title']()}
						</p>

						{#if uniquePeers.length === 0}
							<p class="px-2.5 pb-2 text-xs text-muted-foreground">
								{m['pages.history.sync.empty']()}
							</p>
						{:else}
							{#each uniquePeers as peer (peer.peerId)}
								{@const addrs = peers.getKnownAddr(peer.peerId)}
								{@const syncing = sync.isSyncing(peer.peerId, 'history')}
								<button
									type="button"
									class="flex items-center gap-2.5 rounded-xl px-2.5 py-2 text-left text-sm font-medium transition-colors hover:bg-muted disabled:pointer-events-none disabled:opacity-50"
									disabled={!addrs || syncing}
									onclick={() => {
										syncMenuOpen = false;
										handleSyncHistory(peer.peerId);
									}}
								>
									<MonitorIcon size={16} class="shrink-0 text-muted-foreground" />
									<span class="min-w-0 flex-1">
										<span class="block truncate text-foreground">
											{peer.deviceName ?? shortId(peer.peerId)}
										</span>
										<span class="block truncate text-xs text-muted-foreground">
											{peerStatusLabel(peer)}
										</span>
									</span>
									<RefreshCwIcon
										size={14}
										class={cn('shrink-0 text-muted-foreground', syncing && 'animate-spin')}
									/>
								</button>
							{/each}
						{/if}
					</div>
				{/snippet}
			</AcerolaPopover>

			{#if history.items.length > 0}
				<AcerolaAlertDialog
					data={{
						title: m['pages.history.clear.title'](),
						description: m['pages.history.clear.desc'](),
						cancelText: m['pages.history.clear.cancel'](),
						actionText: m['pages.history.clear.confirm']()
					}}
					ui={{ variant: 'destructive' }}
					events={{
						onAction: () => history.clear()
					}}
				>
					<AcerolaButton
						ui={{ variant: 'destructive', size: 'sm', class: 'gap-2 font-medium tracking-wide' }}
					>
						<Trash2 size={16} />
						{m['pages.history.clear.button']()}
					</AcerolaButton>
				</AcerolaAlertDialog>
			{/if}
		</div>
	</div>

	{#if history.loading}
		<div class="flex flex-1 items-center justify-center p-8 text-muted-foreground">
			{m['pages.history.loading']()}
		</div>
	{:else if history.items.length > 0}
		{@const heroItem = history.items[0]}
		{@const heroCover = resolveArtworkPath(heroItem.comicCover)}

		<div class="flex-1 overflow-auto pb-8">
			<div class="relative mb-8 h-100 w-full shrink-0 overflow-hidden rounded-b-3xl">
				{#if heroCover}
					<img
						src={heroCover}
						class="absolute inset-0 h-full w-full object-cover"
						referrerpolicy="no-referrer"
						alt="background"
					/>
				{:else}
					<div class="absolute inset-0 bg-linear-to-b from-primary/20 via-base/50 to-base"></div>
				{/if}
				<div class="absolute inset-0 bg-linear-to-t from-base via-base/40 to-transparent"></div>
				<div
					class="absolute inset-0 hidden bg-linear-to-l from-transparent via-transparent to-base/80 lg:block"
				></div>

				<div class="absolute right-8 bottom-8 left-8 flex items-end gap-8">
					{#if heroCover}
						<button
							class="relative h-64 w-48 overflow-hidden rounded-xl shadow-2xl ring-1 ring-surface transition-transform hover:scale-105"
							onclick={() => openComic(heroItem)}
						>
							<img
								src={heroCover}
								class="h-full w-full object-cover"
								referrerpolicy="no-referrer"
								alt={heroItem.comicName}
							/>
						</button>
					{:else}
						<button
							class="flex h-64 w-48 items-center justify-center rounded-xl bg-surface text-muted-foreground transition-transform hover:scale-105"
							onclick={() => openComic(heroItem)}
						>
							<PlaceholderManga class="h-full w-full" preserveAspectRatio="none" />
						</button>
					{/if}

					<div class="flex-1 space-y-4 pb-2">
						<button
							class="text-left transition-opacity hover:opacity-80"
							onclick={() => openComic(heroItem)}
						>
							<h3 class="text-5xl font-black tracking-tight text-white drop-shadow-md">
								{heroItem.comicName}
							</h3>
						</button>

						<div class="flex items-center gap-2 text-lg font-medium text-white/80">
							<BookOpen size={20} />
							<span>{m['pages.history.chapter_label']({ chapter: heroItem.chapterName })}</span>
							<span class="mx-2 opacity-50">•</span>
							<span>{m['pages.history.page_label']({ page: heroItem.lastPage + 1 })}</span>
						</div>

						<AcerolaButton
							events={{ onClick: () => resumeReading(heroItem) }}
							ui={{
								class:
									'mt-4 px-8 py-6 rounded-full font-black tracking-widest text-sm hover:scale-105 transition-all shadow-xl shadow-primary/20'
							}}
						>
							{m['pages.history.resume']()}
						</AcerolaButton>
					</div>
				</div>
			</div>

			{#if history.items.length > 1}
				<div class="px-8">
					<h3 class="mb-6 text-xl font-bold tracking-tight opacity-80">
						{m['pages.history.older']()}
					</h3>
					<div class="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-6">
						{#each history.items.slice(1) as item (item.comicDirectoryId)}
							{@const cover = resolveArtworkPath(item.comicCover) || undefined}
							{@const comicBookmark = bookmarkStore.getBookmarkForComic(item.comicDirectoryId)}
							<AcerolaCardImage
								data={{
									title: item.comicName,
									cover
								}}
								ui={{ class: 'w-full' }}
								events={{ onClick: () => openComic(item) }}
							>
								{#snippet footer()}
									<div class="mt-1 flex flex-col gap-1">
										<span
											class="text-overlay flex items-center gap-1 text-[10px] font-black tracking-wider uppercase"
										>
											<BookOpen size={10} class="shrink-0" />
											<span class="truncate">
												{m['pages.history.chapter_label']({ chapter: item.chapterName })}
											</span>
										</span>

										<span class="text-overlay text-[10px] opacity-80">
											{m['pages.history.page_label']({ page: item.lastPage + 1 })}
										</span>
									</div>
								{/snippet}

								{#snippet floatingBadge()}
									{#if comicBookmark != null}
										<AcerolaBookmarkRibbon color={comicBookmark.color} name={comicBookmark.name} />
									{/if}
								{/snippet}

								{#snippet placeholder()}
									<div class="h-full w-full bg-surface">
										<PlaceholderManga class="h-full w-full" />
									</div>
								{/snippet}
							</AcerolaCardImage>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<div class="flex flex-1 items-center justify-center p-8 text-muted-foreground">
			{m['pages.history.no_events']()}
		</div>
	{/if}
</div>
