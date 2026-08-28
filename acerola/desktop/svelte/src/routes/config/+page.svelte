<script lang="ts">
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaCommand from '$lib/components/acerola-command/acerola-command.svelte';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import AcerolaSwitch from '$lib/components/acerola-switch/acerola-switch.svelte';
	import { Button } from '$lib/components/ui/button';
	import ThemePicker from './components/theme-picker.svelte';

	import * as Command from '$lib/components/ui/command';
	import { LANGUAGES, type LanguageCode } from '$lib/constants/languages';
	import { m } from '$lib/paraglide/messages';

	import { useComicInfoPreference } from '$lib/hooks/preferences/use-comic-info.svelte';
	import { useMetadataLanguage } from '$lib/hooks/preferences/use-metadata-language.svelte';
	import { useLibraryScanner } from '$lib/hooks/store/use-comic-scanner.svelte';
	import { DIRECTORY_SCAN_COMMANDS } from '$lib/contracts/library/library.commands';
	import { METADATA_COMMANDS } from '$lib/contracts/metadata/metadata.commands';
	import { useSelectFolder } from '$lib/hooks/store/use-select-folder.svelte';
	import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
	import { useBookmarks } from '$lib/hooks/store/use-bookmarks.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Input } from '$lib/components/ui/input';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { notificationStore } from '$lib/components/acerola-notification/acerola-notification.svelte';
	import { extractErrorMessage } from '$lib/utils/error.utils';

	const { notify } = notificationStore;

	import AniListIcon from '$lib/assets/icons/anilist.svg?component';
	import MangaDexIcon from '$lib/assets/icons/mangadex.svg?component';
	import CloudSync from '@lucide/svelte/icons/cloud-sync';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import FolderSync from '@lucide/svelte/icons/folder-sync';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import PlayIcon from '@lucide/svelte/icons/play';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import FileCode2 from '@lucide/svelte/icons/file-code-2';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';

	const ctx = useTheme();
	const folder = useSelectFolder();
	const comicInfoPreference = useComicInfoPreference();
	const bookmarkStore = useBookmarks();

	const CATEGORY_COLORS = [
		0xfff44336, 0xffe91e63, 0xff9c27b0, 0xff673ab7, 0xff3f51b5, 0xff2196f3, 0xff03a9f4, 0xff00bcd4,
		0xff009688, 0xff4caf50, 0xff8bc34a, 0xffcddc39, 0xffffeb3b, 0xffffc107, 0xffff9800, 0xffff5722,
		0xff795548, 0xff9e9e9e, 0xff607d8b
	];

	let newBookmarkName = $state('');
	let newBookmarkColor = $state<number>(CATEGORY_COLORS[0]);

	let metadataLanguagePopoverOpen = $state(false);
	const metadataLanguageStore = useMetadataLanguage();

	const selectedMetadataLanguageLabel = $derived(
		LANGUAGES.find((lang) => lang.code === metadataLanguageStore.metadataLanguage)?.label ??
			metadataLanguageStore.metadataLanguage
	);

	const refreshScanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.refreshLibrary,
		() => folder.folderPath
	);

	const rebuildScanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.rebuildLibrary,
		() => folder.folderPath
	);

	function selectMetadataLanguage(code: LanguageCode) {
		metadataLanguageStore.selectMetadataLanguage(code);
		metadataLanguagePopoverOpen = false;
	}

	async function handleSyncAllMangadex() {
		try {
			notify.info(m['pages.config.toast.sync.mangadex.start'](), { duration: 0 });
			await invoke(METADATA_COMMANDS.syncAllMangadex, {
				language: metadataLanguageStore.metadataLanguage,
				generateComicInfo: comicInfoPreference.comicInfoPreference ?? false
			});
		} catch (error: unknown) {
			const msg = extractErrorMessage(error);
			notify.error(m['pages.config.toast.sync.mangadex.error']({ msg }), { duration: 0 });
		}
	}

	async function handleSyncAllAnilist() {
		try {
			notify.info(m['pages.config.toast.sync.anilist.start'](), { duration: 0 });
			await invoke(METADATA_COMMANDS.syncAllAnilist, {
				language: metadataLanguageStore.metadataLanguage,
				generateComicInfo: comicInfoPreference.comicInfoPreference ?? false
			});
		} catch (error: unknown) {
			const msg = extractErrorMessage(error);
			notify.error(m['pages.config.toast.sync.anilist.error']({ msg }), { duration: 0 });
		}
	}

	onMount(() => {
		let unlistenProgress: UnlistenFn | undefined;
		let unlistenComplete: UnlistenFn | undefined;
		let unlistenError: UnlistenFn | undefined;

		(async () => {
			await folder.loadSavedPath();
			await bookmarkStore.loadBookmarks();

			unlistenProgress = await listen<string>('metadata:sync_all:progress', (event) => {
				notify.info(m['pages.config.toast.sync.progress']({ name: event.payload }), {
					duration: 0
				});
			});

			unlistenComplete = await listen('metadata:sync_all:complete', () => {
				notify.success(m['pages.config.toast.sync.complete'](), { duration: 0 });
			});

			unlistenError = await listen<any>('metadata:sync_all:error', (event) => {
				const msg = event.payload?.message || event.payload;
				notify.error(m['pages.config.toast.sync.error']({ msg }), { duration: 0 });
			});
		})();

		return () => {
			if (unlistenProgress) unlistenProgress();
			if (unlistenComplete) unlistenComplete();
			if (unlistenError) unlistenError();
		};
	});

	$effect(() => {
		folder.loadSavedPath();
	});

	$effect(() => {
		comicInfoPreference.loadSavedComicInfoPreference();
	});

	$effect(() => {
		metadataLanguageStore.loadSavedMetadataLanguage();
	});
</script>

<div class="mx-auto w-full max-w-5xl space-y-12 p-8">
	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold tracking-tight text-foreground">
			{m['pages.config.title']()}
		</h1>

		<p class="mt-2 text-muted-foreground">
			{m['pages.config.desc']()}
		</p>
	</div>

	<!-- Configuração dos Arquivos -->
	<section class="space-y-4">
		<div
			class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
		>
			<FileTextIcon size={16} />
			{m['pages.config.file_system.title']()}
		</div>

		<div class="grid gap-4">
			<!-- Item: Pasta dos mangás -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.file_system.comic_path.title'](),
					description: m['pages.config.file_system.comic_path.desc']({
						path: folder.folderPath ?? ''
					})
				}}
				events={{ onClick: folder.selectFolder }}
			>
				{#snippet icon()}
					<FolderIcon class="text-chart-5" size={24} />
				{/snippet}

				{#snippet action()}
					<AcerolaButtonIcon
						ui={{
							class:
								'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
						}}
					>
						<PlayIcon />
					</AcerolaButtonIcon>
				{/snippet}
			</AcerolaHeroButton>

			<!-- Item: Gerar ComicInfo.xml para os mangás -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.file_system.comic_info.title'](),
					description: m['pages.config.file_system.comic_info.desc']()
				}}
			>
				{#snippet icon()}
					<FileTextIcon class="text-chart-2" size={24} />
				{/snippet}

				{#snippet action()}
					<AcerolaSwitch
						state={{ checked: comicInfoPreference.comicInfoPreference ?? false }}
						events={{
							onCheckedChange: async (checked) => {
								await comicInfoPreference.selectComicInfoPreference(checked);
							}
						}}
					/>
				{/snippet}
			</AcerolaHeroButton>
		</div>
	</section>

	<!-- Configuração dos Arquivos -->
	<section class="space-y-4">
		<div
			class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
		>
			<FolderIcon size={16} />
			{m['pages.config.library.title']()}
		</div>

		<div class="grid gap-4">
			<!-- Item: Iniciar sincronização rápida, aqui sera usado o refresh_library rapido -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.file_system.sync.fast.title'](),
					description: m['pages.config.file_system.sync.fast.desc']()
				}}
				events={{ onClick: () => refreshScanner.start() }}
			>
				{#snippet icon()}
					<FolderSync class="text-chart-3" size={24} />
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

			<!-- Item: Sincronização profunda, reescreve tudo do banco de dados, aqui sera usado o rebuild_library -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.file_system.sync.deep.title'](),
					description: m['pages.config.file_system.sync.deep.desc']()
				}}
				events={{ onClick: () => rebuildScanner.start() }}
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

			<!-- Item: Navega para a tela de templates de nomenclatura -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.templates.nav.title'](),
					description: m['pages.config.templates.nav.desc']()
				}}
				events={{ onClick: () => goto('/config/templates') }}
			>
				{#snippet icon()}
					<FileCode2 class="text-chart-2" size={24} />
				{/snippet}

				{#snippet action()}
					<AcerolaButtonIcon
						ui={{
							class:
								'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
						}}
					>
						<ChevronRightIcon />
					</AcerolaButtonIcon>
				{/snippet}
			</AcerolaHeroButton>
		</div>
	</section>

	<!-- Aparência (Componente Existente) -->
	<ThemePicker
		data={{ theme: ctx.theme, mode: ctx.resolved }}
		events={{ onSelect: ctx.setTheme }}
	/>

	<!-- Metadados -->

	<section class="space-y-4">
		<div
			class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
		>
			<CloudSync size={16} />
			{m['pages.config.metadata.title']()}
		</div>

		<div class="grid gap-4">
			<!-- Item: Seleção do idioma dos metadados -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.metadata.lang.title'](),
					description: m['pages.config.metadata.lang.desc']()
				}}
			>
				{#snippet icon()}
					<LanguagesIcon class="text-chart-4" size={24} />
				{/snippet}

				{#snippet action()}
					<AcerolaPopover
						state={{ open: metadataLanguagePopoverOpen }}
						events={{ onOpenChange: (open) => (metadataLanguagePopoverOpen = open) }}
						ui={{
							contentClass:
								'w-80 overflow-hidden rounded-2xl border-border/40 bg-card/95 p-0 shadow-2xl backdrop-blur-md'
						}}
					>
						{#snippet trigger()}
							<Button
								variant="outline"
								class="h-10 min-w-24 gap-2 rounded-full border-border/60 bg-background/70 px-3 font-medium transition-all group-hover:border-primary/50 group-hover:bg-primary group-hover:text-primary-foreground"
							>
								<span class="max-w-36 truncate text-sm">{selectedMetadataLanguageLabel}</span>
								<ChevronDownIcon class="size-4 opacity-70" />
							</Button>
						{/snippet}

						{#snippet content()}
							<div class="flex flex-col">
								<div class="border-b border-border/40 bg-muted/20 px-4 py-3">
									<div class="flex items-start gap-3">
										<div class="rounded-xl bg-chart-4/10 p-2 text-chart-4">
											<LanguagesIcon size={18} />
										</div>
										<div class="min-w-0">
											<h3 class="text-sm font-bold text-foreground">
												{m['pages.config.metadata.lang.popover_title']()}
											</h3>
											<p class="mt-0.5 text-xs leading-relaxed text-muted-foreground">
												{m['pages.config.metadata.lang.popover_desc']({
													language: selectedMetadataLanguageLabel
												})}
											</p>
										</div>
									</div>
								</div>

								<AcerolaCommand state={{ value: metadataLanguageStore.metadataLanguage }}>
									<Command.Input
										placeholder={m['pages.config.metadata.lang.search_placeholder']()}
									/>

									<Command.List class="max-h-72 p-2">
										<Command.Empty class="px-4 py-8 text-center text-sm text-muted-foreground">
											{m['pages.config.metadata.lang.empty']()}
										</Command.Empty>

										<Command.Group heading={m['pages.config.metadata.lang.group_title']()}>
											{#each LANGUAGES as lang}
												<Command.Item
													value={lang.code}
													class="group/language cursor-pointer rounded-xl px-3 py-2.5 data-selected:bg-primary/10 data-selected:text-foreground data-[checked=true]:bg-primary/10"
													onSelect={() => selectMetadataLanguage(lang.code)}
												>
													<div class="min-w-0 flex-1">
														<p class="truncate font-semibold">{lang.label}</p>
														<p class="text-xs text-muted-foreground">
															{lang.code}
														</p>
													</div>
												</Command.Item>
											{/each}
										</Command.Group>
									</Command.List>
								</AcerolaCommand>
							</div>
						{/snippet}
					</AcerolaPopover>
				{/snippet}
			</AcerolaHeroButton>

			<!-- Item: Sync com o mangadex -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.metadata.mangadex.title'](),
					description: m['pages.config.metadata.mangadex.desc']()
				}}
				events={{ onClick: handleSyncAllMangadex }}
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
								'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
						}}
					>
						<RefreshCw />
					</AcerolaButtonIcon>
				{/snippet}
			</AcerolaHeroButton>

			<!-- Item: Sync com o anilist -->
			<AcerolaHeroButton
				data={{
					title: m['pages.config.metadata.anilist.title'](),
					description: m['pages.config.metadata.anilist.desc']()
				}}
				events={{ onClick: handleSyncAllAnilist }}
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
								'rounded-full transition-all group-hover:bg-primary group-hover:text-primary-foreground'
						}}
					>
						<RefreshCw />
					</AcerolaButtonIcon>
				{/snippet}
			</AcerolaHeroButton>
		</div>
	</section>

	<!-- Marcadores (Bookmarks) -->
	<section class="space-y-4">
		<div
			class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
		>
			<BookmarkIcon size={16} />
			{m['pages.config.bookmarks.title']()}
		</div>

		<div class="grid gap-4">
			<div class="rounded-2xl border border-border/40 bg-card/50 p-6 backdrop-blur-sm">
				<p class="mb-4 text-sm text-muted-foreground">{m['pages.config.bookmarks.desc']()}</p>

				<div class="mb-6 space-y-6">
					<!-- Row 1: Name and Button -->
					<div class="flex items-end gap-4">
						<div class="flex-1 space-y-1">
							<label for="bookmarkName" class="text-xs font-semibold"
								>{m['pages.config.bookmarks.name']()}</label
							>
							<Input
								id="bookmarkName"
								placeholder={m['pages.config.bookmarks.name']()}
								bind:value={newBookmarkName}
								class="h-10 bg-background text-foreground"
							/>
						</div>
						<Button
							disabled={!newBookmarkName.trim() || bookmarkStore.isLoading}
							onclick={async () => {
								await bookmarkStore.createBookmark(newBookmarkName, newBookmarkColor);
								newBookmarkName = '';
							}}
							class="h-10 gap-2 px-6"
						>
							<PlusIcon size={16} />
							{m['pages.config.bookmarks.create']()}
						</Button>
					</div>

					<!-- Row 2: Colors -->
					<div class="space-y-2">
						<span class="block text-xs font-semibold">{m['pages.config.bookmarks.color']()}</span>
						<div class="flex flex-wrap gap-2">
							{#each CATEGORY_COLORS as hexColor}
								{@const hexLabel = '#' + (hexColor & 0xffffff).toString(16).padStart(6, '0')}
								<button
									type="button"
									class="relative h-8 w-8 cursor-pointer rounded-full transition-transform hover:scale-110"
									style="background-color: {hexLabel}"
									onclick={() => (newBookmarkColor = hexColor)}
									aria-label={m['pages.config.bookmarks.color_option']({ hex: hexLabel })}
									aria-pressed={newBookmarkColor === hexColor}
								>
									{#if newBookmarkColor === hexColor}
										<div
											class="absolute inset-0 rounded-full border-2 border-primary ring-2 ring-background"
										></div>
									{/if}
								</button>
							{/each}
						</div>
					</div>
				</div>

				<div class="space-y-2">
					{#if bookmarkStore.bookmarks.length === 0}
						<div
							class="rounded-xl border border-dashed border-border/40 p-8 text-center text-sm text-muted-foreground"
						>
							{m['pages.config.bookmarks.empty']()}
						</div>
					{:else}
						{#each bookmarkStore.bookmarks as bookmark (bookmark.id)}
							<div
								class="flex items-center justify-between rounded-xl border border-border/40 bg-background/50 p-3 transition-colors hover:bg-muted/50"
							>
								<div class="flex items-center gap-3">
									<div
										class="h-6 w-6 rounded-full shadow-inner"
										style="background-color: #{(bookmark.color & 0xffffff)
											.toString(16)
											.padStart(6, '0')}"
									></div>
									<span class="font-medium">{bookmark.name}</span>
								</div>
								<Button
									variant="ghost"
									size="icon"
									class="h-8 w-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
									onclick={() => bookmarkStore.deleteBookmark(bookmark.id)}
								>
									<Trash2Icon size={16} />
								</Button>
							</div>
						{/each}
					{/if}
				</div>
			</div>
		</div>
	</section>
</div>
