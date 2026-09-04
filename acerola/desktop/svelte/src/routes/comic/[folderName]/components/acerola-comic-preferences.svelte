<script lang="ts" module>
	import type { SyncDirection } from '$lib/hooks/store/use-network-sync.svelte';

	export type ComicPreferencesProps = {
		data?: {
			hasVolumeStructure?: boolean;
			bookmarks?: { id: number; name: string; color: number }[];
			/** Dispositivos já pareados (ver `usePeerConnection().pairedPeers`), resolvidos pelo
			 *  chamador com rótulo pronto pra exibir e o `addrs` já disponível pra disparar
			 *  `syncComic` sem essa tela precisar conhecer o hook de rede. */
			pairedPeers?: { peerId: string; label: string; addrs: number[] }[];
		};
		state: {
			volumeViewMode: 'cover' | 'banner';
			bookmarkId: number | null;
			externalSyncEnabled: boolean;
			/** `peerId`s com um sync individual em andamento agora (pode ser mais de um em
			 *  paralelo, um por dispositivo) — só pra desabilitar o item da lista certo e
			 *  mostrar o spinner. */
			syncingPeerIds?: string[];
			/** `useMetadataSync().isSyncing` do chamador — true durante mangadex/anilist/comicInfo,
			 *  desabilita os três botões pra evitar disparo duplo enquanto um já roda. */
			metadataSyncing?: boolean;
		};
		events: {
			onVolumeViewModeChange: (value: 'cover' | 'banner') => void;
			onBookmarkChange: (value: number | null) => void;
			onExternalSyncChange: (value: boolean) => void;
			onSyncMangadex?: () => void;
			onSyncAnilist?: () => void;
			onSyncComicInfo?: () => void;
			onRescanComic?: () => void;
			onDeepRescanComic?: () => void;
			onRegenerateCover?: () => void;
			onRegenerateVolumeCovers?: () => void;
			onClearMetadata?: () => Promise<void> | void;
			/** Dispara `syncComic` pro `peerId` escolhido, na direção explícita clicada — ver
			 *  `use-network-sync.svelte.ts`. */
			onSyncToDevice?: (
				peerId: string,
				addrs: number[],
				direction: SyncDirection
			) => Promise<void> | void;
		};
	};
</script>

<script lang="ts">
	import { SvelteSet } from 'svelte/reactivity';
	import AcerolaAccordionCard from '$lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaSelect from '$lib/components/acerola-select/acerola-select.svelte';
	import AcerolaToggleGroup from '$lib/components/acerola-toggle-group/acerola-toggle-group.svelte';
	import AcerolaAlertDialog from '$lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import AcerolaSwitch from '$lib/components/acerola-switch/acerola-switch.svelte';
	import { buttonVariants } from '$lib/components/ui/button';
	import { ToggleGroupItem } from '$lib/components/ui/toggle-group/index';
	import { m } from '$lib/paraglide/messages';

	import Layers from '@lucide/svelte/icons/layers';
	import Layers2 from '@lucide/svelte/icons/layers-2';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import CloudSync from '@lucide/svelte/icons/cloud-sync';
	import Link from '@lucide/svelte/icons/link';
	import FileText from '@lucide/svelte/icons/file-text';
	import MangaDexIcon from '$lib/assets/icons/mangadex.svg?component';
	import AniListIcon from '$lib/assets/icons/anilist.svg?component';
	import FolderSync from '@lucide/svelte/icons/folder-sync';
	import DatabaseZap from '@lucide/svelte/icons/database-zap';
	import Image from '@lucide/svelte/icons/image';
	import Eraser from '@lucide/svelte/icons/eraser';
	import Share2 from '@lucide/svelte/icons/share-2';
	import Upload from '@lucide/svelte/icons/upload';
	import Download from '@lucide/svelte/icons/download';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import MonitorSmartphone from '@lucide/svelte/icons/monitor-smartphone';

	let { data, events, state: preferences }: ComicPreferencesProps = $props();

	let showClearMetadataDialog = $state(false);
	let showDeepRescanDialog = $state(false);
	let showPeerMenu = $state(false);

	// 3 categorias (Leitura / Sincronização / Avançado) em vez das 6 abas de antes, todas
	// colapsadas por padrão e expandindo inline na própria lista — sem navegar pra outra
	// tela, mesmo padrão usado em config/. Mais de uma pode ficar aberta ao mesmo tempo.
	const expandedCategories = new SvelteSet<string>();

	function toggleCategory(id: string) {
		if (expandedCategories.has(id)) {
			expandedCategories.delete(id);
		} else {
			expandedCategories.add(id);
		}
	}

	// Cor de borda no hover de cada categoria — combina com a cor do ícone (mesma paleta
	// chart-N), reforçando a identidade visual de cada card no hover.
	const CATEGORY_HOVER_BORDER: Record<string, string> = {
		reading: 'hover:border-chart-2/60',
		sync: 'hover:border-chart-1/60',
		advanced: 'hover:border-chart-3/60'
	};
</script>

<div class="grid gap-4">
	<!-- Leitura -->
	<AcerolaAccordionCard
		data={{
			title: m['pages.comic.preferences.reading'](),
			description: m['pages.comic.preferences.categories.reading.desc']()
		}}
		state={{ expanded: expandedCategories.has('reading') }}
		events={{ onToggle: () => toggleCategory('reading') }}
		ui={{ class: CATEGORY_HOVER_BORDER.reading }}
	>
		{#snippet icon()}
			<Settings2 class="text-chart-2" size={24} />
		{/snippet}

		{#snippet children()}
			<!-- Volume Highlight (Conditional) -->
			{#if data?.hasVolumeStructure}
				<AcerolaHeroButton
					data={{
						title: m['pages.comic.preferences.volume_highlight.title'](),
						description: m['pages.comic.preferences.volume_highlight.desc']()
					}}
				>
					{#snippet icon()}
						<Layers class="text-chart-2" size={24} />
					{/snippet}

					{#snippet action()}
						<AcerolaToggleGroup
							config={{ type: 'single' }}
							state={{ value: preferences.volumeViewMode }}
							events={{
								onValueChange: (value) => {
									if (value === 'cover' || value === 'banner') {
										events.onVolumeViewModeChange(value);
									}
								}
							}}
						>
							{#snippet children()}
								<ToggleGroupItem value="cover" class="px-4 py-1.5 text-[10px] font-black uppercase">
									{m['pages.comic.preferences.volume_highlight.cover']()}
								</ToggleGroupItem>

								<ToggleGroupItem
									value="banner"
									class="px-4 py-1.5 text-[10px] font-black uppercase"
								>
									{m['pages.comic.preferences.volume_highlight.banner']()}
								</ToggleGroupItem>
							{/snippet}
						</AcerolaToggleGroup>
					{/snippet}
				</AcerolaHeroButton>
			{/if}

			<!-- Bookmark Assignment -->
			<AcerolaHeroButton
				data={{
					title: m['pages.comic.preferences.bookmark.title'](),
					description: m['pages.comic.preferences.bookmark.desc']()
				}}
			>
				{#snippet icon()}
					<BookmarkIcon class="text-chart-4" size={24} />
				{/snippet}

				{#snippet action()}
					<AcerolaSelect
						data={{
							options: [
								{ value: 'none', label: m['pages.comic.preferences.bookmark.none']() },
								...(data?.bookmarks ?? []).map((b) => ({
									value: b.id.toString(),
									label: b.name,
									color: b.color
								}))
							]
						}}
						state={{ value: preferences.bookmarkId ? preferences.bookmarkId.toString() : 'none' }}
						events={{
							onValueChange: (v) => events.onBookmarkChange(v === 'none' ? null : parseInt(v))
						}}
					/>
				{/snippet}
			</AcerolaHeroButton>
		{/snippet}
	</AcerolaAccordionCard>

	<!-- Sincronização -->
	<AcerolaAccordionCard
		data={{
			title: m['pages.comic.preferences.categories.sync.title'](),
			description: m['pages.comic.preferences.categories.sync.desc']()
		}}
		state={{ expanded: expandedCategories.has('sync') }}
		events={{ onToggle: () => toggleCategory('sync') }}
		ui={{ class: CATEGORY_HOVER_BORDER.sync }}
	>
		{#snippet icon()}
			<CloudSync class="text-chart-1" size={24} />
		{/snippet}

		{#snippet children()}
			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<CloudSync size={16} />
					{m['pages.config.metadata.title']()}
				</div>

				<div class="grid gap-4">
					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.external_sync.title'](),
							description: m['pages.comic.preferences.external_sync.desc']()
						}}
					>
						{#snippet icon()}
							<Link class="text-chart-1" size={24} />
						{/snippet}

						{#snippet action()}
							<AcerolaSwitch
								state={{ checked: preferences.externalSyncEnabled }}
								events={{ onCheckedChange: events.onExternalSyncChange }}
							/>
						{/snippet}
					</AcerolaHeroButton>

					{#if preferences.externalSyncEnabled}
						<AcerolaHeroButton
							data={{
								title: m['pages.config.metadata.mangadex.title'](),
								description: m['pages.config.metadata.mangadex.desc']()
							}}
							events={{
								onClick: preferences.metadataSyncing ? undefined : events.onSyncMangadex
							}}
						>
							{#snippet icon()}
								<span style="all: unset; display: inline-flex;">
									<MangaDexIcon class="h-6 w-6 rounded-lg" />
								</span>
							{/snippet}

							{#snippet action()}
								<AcerolaButtonIcon
									ui={{
										class:
											'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground',
										disabled: preferences.metadataSyncing
									}}
								>
									<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
								</AcerolaButtonIcon>
							{/snippet}
						</AcerolaHeroButton>

						<AcerolaHeroButton
							data={{
								title: m['pages.config.metadata.anilist.title'](),
								description: m['pages.config.metadata.anilist.desc']()
							}}
							events={{
								onClick: preferences.metadataSyncing ? undefined : events.onSyncAnilist
							}}
						>
							{#snippet icon()}
								<span style="all: unset; display: inline-flex;">
									<AniListIcon class="h-6 w-6 rounded-lg" />
								</span>
							{/snippet}

							{#snippet action()}
								<AcerolaButtonIcon
									ui={{
										class:
											'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground',
										disabled: preferences.metadataSyncing
									}}
								>
									<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
								</AcerolaButtonIcon>
							{/snippet}
						</AcerolaHeroButton>
					{/if}

					<AcerolaHeroButton
						data={{
							title: m['pages.comic.toast.comic_info.title'](),
							description: m['pages.comic.toast.comic_info.desc']()
						}}
						events={{
							onClick: preferences.metadataSyncing ? undefined : events.onSyncComicInfo
						}}
					>
						{#snippet icon()}
							<span style="all: unset; display: inline-flex;">
								<FileText class="h-6 w-6 text-foreground" />
							</span>
						{/snippet}

						{#snippet action()}
							<AcerolaButtonIcon
								ui={{
									class:
										'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground',
									disabled: preferences.metadataSyncing
								}}
							>
								<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
							</AcerolaButtonIcon>
						{/snippet}
					</AcerolaHeroButton>
				</div>
			</section>

			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<FolderSync size={16} />
					{m['pages.comic.preferences.file_sync.title']()}
				</div>

				<div class="grid gap-4">
					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.file_sync.rescan.title'](),
							description: m['pages.comic.preferences.file_sync.rescan.desc']()
						}}
						events={{ onClick: events.onRescanComic }}
					>
						{#snippet icon()}
							<FolderSync class="text-chart-1" size={24} />
						{/snippet}

						{#snippet action()}
							<AcerolaButtonIcon
								ui={{
									class:
										'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
								}}
							>
								<RefreshCw />
							</AcerolaButtonIcon>
						{/snippet}
					</AcerolaHeroButton>

					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.file_sync.deep_rescan.title'](),
							description: m['pages.comic.preferences.file_sync.deep_rescan.desc']()
						}}
						events={{ onClick: () => (showDeepRescanDialog = true) }}
					>
						{#snippet icon()}
							<DatabaseZap class="text-destructive" size={24} />
						{/snippet}

						{#snippet action()}
							<AcerolaButtonIcon
								ui={{
									class:
										'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
								}}
							>
								<RefreshCw />
							</AcerolaButtonIcon>
						{/snippet}
					</AcerolaHeroButton>
				</div>
			</section>

			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<Share2 size={16} />
					{m['pages.comic.preferences.p2p_sync.title']()}
				</div>

				<div class="grid gap-4">
					{#if !data?.pairedPeers || data.pairedPeers.length === 0}
						<p class="px-2 text-xs text-muted-foreground">
							{m['pages.comic.preferences.p2p_sync.empty']()}
						</p>
					{:else}
						<AcerolaHeroButton
							data={{
								title: m['pages.comic.preferences.p2p_sync.send.title'](),
								description: m['pages.comic.preferences.p2p_sync.send.desc']()
							}}
						>
							{#snippet icon()}
								<Share2 class="text-chart-1" size={24} />
							{/snippet}

							{#snippet action()}
								<AcerolaPopover
									state={{ open: showPeerMenu }}
									events={{ onOpenChange: (open) => (showPeerMenu = open) }}
									ui={{
										align: 'end',
										contentClass:
											'w-80 overflow-hidden rounded-2xl border-border/40 bg-card/95 p-2 shadow-2xl backdrop-blur-md'
									}}
								>
									{#snippet trigger()}
										<span
											class={buttonVariants({ variant: 'outline', size: 'sm', class: 'gap-2' })}
										>
											<Share2 size={14} />
											{m['pages.comic.preferences.p2p_sync.send.button']()}
										</span>
									{/snippet}

									{#snippet content()}
										<div class="flex flex-col gap-2">
											{#each data.pairedPeers as peer (peer.peerId)}
												{@const syncing = (preferences.syncingPeerIds ?? []).includes(peer.peerId)}
												<div
													class="flex items-center gap-3 rounded-2xl border border-border/50 bg-muted/30 p-3"
												>
													<div
														class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-muted text-foreground"
													>
														<MonitorSmartphone size={20} />
													</div>

													<span class="min-w-0 flex-1 truncate text-sm font-semibold"
														>{peer.label}</span
													>

													{#if syncing}
														<Loader2
															size={20}
															class="shrink-0 animate-spin text-muted-foreground"
														/>
													{:else}
														<div class="flex shrink-0 gap-1.5">
															<AcerolaButtonIcon
																ui={{
																	variant: 'ghost',
																	class:
																		'size-9 rounded-xl bg-chart-1/10 text-chart-1 transition-colors hover:bg-chart-1 hover:text-primary-foreground',
																	title: m['pages.comic.preferences.p2p_sync.send.push']()
																}}
																events={{
																	onClick: () => {
																		showPeerMenu = false;
																		events.onSyncToDevice?.(peer.peerId, peer.addrs, 'push');
																	}
																}}
															>
																<Upload size={18} />
															</AcerolaButtonIcon>
															<AcerolaButtonIcon
																ui={{
																	variant: 'ghost',
																	class:
																		'size-9 rounded-xl bg-chart-3/10 text-chart-3 transition-colors hover:bg-chart-3 hover:text-primary-foreground',
																	title: m['pages.comic.preferences.p2p_sync.send.pull']()
																}}
																events={{
																	onClick: () => {
																		showPeerMenu = false;
																		events.onSyncToDevice?.(peer.peerId, peer.addrs, 'pull');
																	}
																}}
															>
																<Download size={18} />
															</AcerolaButtonIcon>
														</div>
													{/if}
												</div>
											{/each}
										</div>
									{/snippet}
								</AcerolaPopover>
							{/snippet}
						</AcerolaHeroButton>
					{/if}
				</div>
			</section>
		{/snippet}
	</AcerolaAccordionCard>

	<!-- Avançado -->
	<AcerolaAccordionCard
		data={{
			title: m['pages.comic.preferences.categories.advanced.title'](),
			description: m['pages.comic.preferences.categories.advanced.desc']()
		}}
		state={{ expanded: expandedCategories.has('advanced') }}
		events={{ onToggle: () => toggleCategory('advanced') }}
		ui={{ class: CATEGORY_HOVER_BORDER.advanced }}
	>
		{#snippet icon()}
			<Image class="text-chart-3" size={24} />
		{/snippet}

		{#snippet children()}
			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<Image size={16} />
					{m['pages.comic.preferences.cover.title']()}
				</div>

				<div class="grid gap-4">
					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.cover.regenerate.title'](),
							description: m['pages.comic.preferences.cover.regenerate.desc']()
						}}
						events={{ onClick: events.onRegenerateCover }}
					>
						{#snippet icon()}
							<Image class="text-chart-2" size={24} />
						{/snippet}

						{#snippet action()}
							<AcerolaButtonIcon
								ui={{
									class:
										'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
								}}
							>
								<RefreshCw />
							</AcerolaButtonIcon>
						{/snippet}
					</AcerolaHeroButton>

					{#if data?.hasVolumeStructure}
						<AcerolaHeroButton
							data={{
								title: m['pages.comic.preferences.cover.regenerate_volumes.title'](),
								description: m['pages.comic.preferences.cover.regenerate_volumes.desc']()
							}}
							events={{ onClick: events.onRegenerateVolumeCovers }}
						>
							{#snippet icon()}
								<Layers2 class="text-chart-3" size={24} />
							{/snippet}

							{#snippet action()}
								<AcerolaButtonIcon
									ui={{
										class:
											'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
									}}
								>
									<RefreshCw />
								</AcerolaButtonIcon>
							{/snippet}
						</AcerolaHeroButton>
					{/if}
				</div>
			</section>

			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<Eraser size={16} />
					{m['pages.comic.preferences.danger_zone.title']()}
				</div>

				<div class="grid gap-4">
					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.danger_zone.clear_metadata.title'](),
							description: m['pages.comic.preferences.danger_zone.clear_metadata.desc']()
						}}
						events={{ onClick: () => (showClearMetadataDialog = true) }}
					>
						{#snippet icon()}
							<Eraser class="text-destructive" size={24} />
						{/snippet}
					</AcerolaHeroButton>
				</div>
			</section>
		{/snippet}
	</AcerolaAccordionCard>
</div>

<AcerolaAlertDialog
	state={{ open: showClearMetadataDialog }}
	data={{
		title: m['pages.comic.preferences.danger_zone.clear_metadata.confirm.title'](),
		description: m['pages.comic.preferences.danger_zone.clear_metadata.confirm.desc'](),
		cancelText: m['pages.comic.preferences.danger_zone.clear_metadata.confirm.cancel'](),
		actionText: m['pages.comic.preferences.danger_zone.clear_metadata.confirm.action']()
	}}
	events={{
		onAction: async () => {
			await events.onClearMetadata?.();
			showClearMetadataDialog = false;
		},
		onCancel: () => (showClearMetadataDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>

<AcerolaAlertDialog
	state={{ open: showDeepRescanDialog }}
	data={{
		title: m['pages.comic.preferences.file_sync.deep_rescan.confirm.title'](),
		description: m['pages.comic.preferences.file_sync.deep_rescan.confirm.desc'](),
		cancelText: m['pages.comic.preferences.file_sync.deep_rescan.confirm.cancel'](),
		actionText: m['pages.comic.preferences.file_sync.deep_rescan.confirm.action']()
	}}
	events={{
		onAction: () => {
			events.onDeepRescanComic?.();
			showDeepRescanDialog = false;
		},
		onCancel: () => (showDeepRescanDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>
