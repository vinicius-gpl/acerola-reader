<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { fade } from 'svelte/transition';
	import { SvelteSet } from 'svelte/reactivity';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import AcerolaAccordionCard from '$lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaCommand from '$lib/components/acerola-command/acerola-command.svelte';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import AcerolaSwitch from '$lib/components/acerola-switch/acerola-switch.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Command from '$lib/components/ui/command';
	import ThemePicker from './components/acerola-theme-picker.svelte';
	import AcerolaBookmarkManager from './components/acerola-bookmark-manager.svelte';
	import { useComicInfoPreference } from '$lib/hooks/preferences/use-comic-info.svelte';
	import { useMetadataLanguage } from '$lib/hooks/preferences/use-metadata-language.svelte';
	import { useLibraryScanner } from '$lib/hooks/store/use-comic-scanner.svelte';
	import { useSelectFolder } from '$lib/hooks/store/use-select-folder.svelte';
	import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
	import { DIRECTORY_SCAN_COMMANDS } from '$lib/contracts/library/library.commands';
	import { METADATA_COMMANDS } from '$lib/contracts/metadata/metadata.commands';
	import { notificationStore } from '$lib/components/acerola-notification/acerola-notification.svelte';
	import { extractErrorMessage } from '$lib/utils/error.utils';
	import { LANGUAGES, type LanguageCode } from '$lib/constants/languages';
	import { m } from '$lib/paraglide/messages';

	import AniListIcon from '$lib/assets/icons/anilist.svg?component';
	import MangaDexIcon from '$lib/assets/icons/mangadex.svg?component';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import CloudSync from '@lucide/svelte/icons/cloud-sync';
	import FileCode2 from '@lucide/svelte/icons/file-code-2';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import FolderSync from '@lucide/svelte/icons/folder-sync';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import PlayIcon from '@lucide/svelte/icons/play';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';

	const { notify } = notificationStore;

	const ctx = useTheme();
	const folder = useSelectFolder();
	const comicInfoPreference = useComicInfoPreference();
	const metadataLanguageStore = useMetadataLanguage();

	let metadataLanguagePopoverOpen = $state(false);
	// `metadata:sync_all:complete`/`:error` não dizem qual fonte terminou (ver
	// sync_all_metadata_mangadex/anilist no backend, que emitem o mesmo evento pras duas) —
	// por isso só uma sincronização "all" roda por vez, nunca mangadex e anilist juntas.
	let syncingSource = $state<'mangadex' | 'anilist' | null>(null);

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
		if (syncingSource) return;
		syncingSource = 'mangadex';
		try {
			notify.info(m['pages.config.toast.sync.mangadex.start'](), { duration: 0 });
			await invoke(METADATA_COMMANDS.syncAllMangadex, {
				language: metadataLanguageStore.metadataLanguage,
				generateComicInfo: comicInfoPreference.comicInfoPreference ?? false
			});
		} catch (error: unknown) {
			syncingSource = null;
			const msg = extractErrorMessage(error);
			notify.error(m['pages.config.toast.sync.mangadex.error']({ msg }), { duration: 0 });
		}
	}

	async function handleSyncAllAnilist() {
		if (syncingSource) return;
		syncingSource = 'anilist';
		try {
			notify.info(m['pages.config.toast.sync.anilist.start'](), { duration: 0 });
			await invoke(METADATA_COMMANDS.syncAllAnilist, {
				language: metadataLanguageStore.metadataLanguage,
				generateComicInfo: comicInfoPreference.comicInfoPreference ?? false
			});
		} catch (error: unknown) {
			syncingSource = null;
			const msg = extractErrorMessage(error);
			notify.error(m['pages.config.toast.sync.anilist.error']({ msg }), { duration: 0 });
		}
	}

	onMount(() => {
		let unlistenProgress: UnlistenFn | undefined;
		let unlistenComplete: UnlistenFn | undefined;
		let unlistenError: UnlistenFn | undefined;

		(async () => {
			unlistenProgress = await listen<string>('metadata:sync_all:progress', (event) => {
				notify.info(m['pages.config.toast.sync.progress']({ name: event.payload }), {
					duration: 0
				});
			});

			unlistenComplete = await listen('metadata:sync_all:complete', () => {
				syncingSource = null;
				notify.success(m['pages.config.toast.sync.complete'](), { duration: 0 });
			});

			unlistenError = await listen<any>('metadata:sync_all:error', (event) => {
				syncingSource = null;
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
		comicInfoPreference.loadSavedComicInfoPreference();
	});

	$effect(() => {
		metadataLanguageStore.loadSavedMetadataLanguage();
	});

	// Categorias colapsam/expandem inline na própria lista — nenhuma navega pra outra tela.
	// Mais de uma pode ficar aberta ao mesmo tempo (SvelteSet em vez de string única).
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
		files: 'hover:border-chart-5/60',
		library: 'hover:border-chart-3/60',
		appearance: 'hover:border-chart-1/60',
		metadata: 'hover:border-chart-4/60',
		bookmarks: 'hover:border-chart-2/60'
	};
</script>

<div in:fade={{ duration: 150 }} class="mx-auto w-full max-w-5xl space-y-8 p-8">
	<div>
		<h1 class="text-3xl font-bold tracking-tight text-foreground">
			{m['pages.config.title']()}
		</h1>

		<p class="mt-2 text-muted-foreground">
			{m['pages.config.desc']()}
		</p>
	</div>

	<div class="grid gap-4">
		<!-- Arquivos -->
		<AcerolaAccordionCard
			data={{
				title: m['pages.config.file_system.title'](),
				description: m['pages.config.categories.files.desc']()
			}}
			state={{ expanded: expandedCategories.has('files') }}
			events={{ onToggle: () => toggleCategory('files') }}
			ui={{ class: CATEGORY_HOVER_BORDER.files }}
		>
			{#snippet icon()}
				<FolderIcon class="text-chart-5" size={24} />
			{/snippet}

			{#snippet children()}
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

				<AcerolaHeroButton
					data={{
						title: m['pages.config.file_system.comic_info.title'](),
						description: m['pages.config.file_system.comic_info.desc']()
					}}
					events={{
						onClick: () =>
							comicInfoPreference.selectComicInfoPreference(
								!(comicInfoPreference.comicInfoPreference ?? false)
							)
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
			{/snippet}
		</AcerolaAccordionCard>

		<!-- Biblioteca -->
		<AcerolaAccordionCard
			data={{
				title: m['pages.config.library.title'](),
				description: m['pages.config.categories.library.desc']()
			}}
			state={{ expanded: expandedCategories.has('library') }}
			events={{ onToggle: () => toggleCategory('library') }}
			ui={{ class: CATEGORY_HOVER_BORDER.library }}
		>
			{#snippet icon()}
				<FolderSync class="text-chart-3" size={24} />
			{/snippet}

			{#snippet children()}
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
			{/snippet}
		</AcerolaAccordionCard>

		<!-- Aparência -->
		<AcerolaAccordionCard
			data={{
				title: m['pages.config.components.theme_piker'](),
				description: m['pages.config.categories.appearance.desc']()
			}}
			state={{ expanded: expandedCategories.has('appearance') }}
			events={{ onToggle: () => toggleCategory('appearance') }}
			ui={{ class: CATEGORY_HOVER_BORDER.appearance }}
		>
			{#snippet icon()}
				<PaletteIcon class="text-chart-1" size={24} />
			{/snippet}

			{#snippet children()}
				<ThemePicker
					data={{ theme: ctx.theme, mode: ctx.resolved }}
					events={{ onSelect: ctx.setTheme }}
					ui={{ showHeader: false }}
				/>
			{/snippet}
		</AcerolaAccordionCard>

		<!-- Metadados -->
		<AcerolaAccordionCard
			data={{
				title: m['pages.config.metadata.title'](),
				description: m['pages.config.categories.metadata.desc']()
			}}
			state={{ expanded: expandedCategories.has('metadata') }}
			events={{ onToggle: () => toggleCategory('metadata') }}
			ui={{ class: CATEGORY_HOVER_BORDER.metadata }}
		>
			{#snippet icon()}
				<CloudSync class="text-chart-4" size={24} />
			{/snippet}

			{#snippet children()}
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

				<AcerolaHeroButton
					data={{
						title: m['pages.config.metadata.mangadex.title'](),
						description: m['pages.config.metadata.mangadex.desc']()
					}}
					events={{ onClick: syncingSource ? undefined : handleSyncAllMangadex }}
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
								disabled: syncingSource !== null
							}}
						>
							<RefreshCw class={syncingSource === 'mangadex' ? 'animate-spin' : ''} />
						</AcerolaButtonIcon>
					{/snippet}
				</AcerolaHeroButton>

				<AcerolaHeroButton
					data={{
						title: m['pages.config.metadata.anilist.title'](),
						description: m['pages.config.metadata.anilist.desc']()
					}}
					events={{ onClick: syncingSource ? undefined : handleSyncAllAnilist }}
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
								disabled: syncingSource !== null
							}}
						>
							<RefreshCw class={syncingSource === 'anilist' ? 'animate-spin' : ''} />
						</AcerolaButtonIcon>
					{/snippet}
				</AcerolaHeroButton>
			{/snippet}
		</AcerolaAccordionCard>

		<!-- Marcadores -->
		<AcerolaAccordionCard
			data={{
				title: m['pages.config.bookmarks.title'](),
				description: m['pages.config.categories.bookmarks.desc']()
			}}
			state={{ expanded: expandedCategories.has('bookmarks') }}
			events={{ onToggle: () => toggleCategory('bookmarks') }}
			ui={{ class: CATEGORY_HOVER_BORDER.bookmarks }}
		>
			{#snippet icon()}
				<BookmarkIcon class="text-chart-2" size={24} />
			{/snippet}

			{#snippet children()}
				<AcerolaBookmarkManager />
			{/snippet}
		</AcerolaAccordionCard>
	</div>
</div>
