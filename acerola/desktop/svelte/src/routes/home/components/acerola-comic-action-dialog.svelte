<script lang="ts">
	import EyeOff from '@lucide/svelte/icons/eye-off';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Eraser from '@lucide/svelte/icons/eraser';
	import Bookmark from '@lucide/svelte/icons/bookmark';
	import CheckSquare from '@lucide/svelte/icons/check-square';
	import Square from '@lucide/svelte/icons/square';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import Layers from '@lucide/svelte/icons/layers';
	import Check from '@lucide/svelte/icons/check';
	import Search from '@lucide/svelte/icons/search';
	import { slide } from 'svelte/transition';
	import { error } from '@tauri-apps/plugin-log';
	import { toast } from 'svelte-sonner';
	import AcerolaDialog from '$lib/components/acerola-dialog/acerola-dialog.svelte';
	import AcerolaAlertDialog from '$lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { Category } from '$lib/contracts/bookmarks/bookmarks.payloads';

	export type ComicActionDialogProps = {
		state?: { open?: boolean };
		data: {
			selectedIds: (string | number)[];
			totalCount?: number;
			bookmarks: Category[];
		};
		events: {
			onHide: (ids: (string | number)[]) => Promise<void>;
			onDelete: (ids: (string | number)[]) => Promise<void>;
			onClearMetadata: (ids: (string | number)[]) => Promise<void>;
			onBookmark: (ids: (string | number)[], categoryId: number) => Promise<void>;
			onSelectAll?: () => void;
			onClose: () => void;
		};
	};

	let { state: controlState, data, events }: ComicActionDialogProps = $props();

	const open = $derived(controlState?.open ?? false);
	const selectedIds = $derived(data.selectedIds);
	const totalCount = $derived(data.totalCount ?? 0);
	const bookmarks = $derived(data.bookmarks);

	let showHideDialog = $state(false);
	let showDeleteDialog = $state(false);
	let showClearMetadataDialog = $state(false);
	let showBookmarkMenu = $state(false);
	let isProcessing = $state(false);
	let activeAction = $state<'hide' | 'delete' | 'clearMetadata' | 'bookmark' | null>(null);
	let activeCategoryId = $state<number | null>(null);
	let searchQuery = $state('');

	const isAllSelected = $derived(totalCount > 0 && selectedIds.length === totalCount);

	const filteredBookmarks = $derived(
		searchQuery.trim() === ''
			? bookmarks
			: bookmarks.filter((b) => b.name.toLowerCase().includes(searchQuery.toLowerCase().trim()))
	);

	async function handleHide() {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'hide';
		try {
			await events.onHide(selectedIds);
			showHideDialog = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = typeof err === 'object' && err !== null ? JSON.stringify(err) : String(err);
			error(`Failed to hide comics: ${msg}`);
			toast.error(m['pages.home.toast.error.hide']());
		} finally {
			isProcessing = false;
			activeAction = null;
		}
	}

	async function handleDelete() {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'delete';
		try {
			await events.onDelete(selectedIds);
			showDeleteDialog = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = typeof err === 'object' && err !== null ? JSON.stringify(err) : String(err);
			error(`Failed to delete comics: ${msg}`);
			toast.error(m['pages.home.toast.error.delete']());
		} finally {
			isProcessing = false;
			activeAction = null;
		}
	}

	async function handleClearMetadata() {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'clearMetadata';
		try {
			await events.onClearMetadata(selectedIds);
			showClearMetadataDialog = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = typeof err === 'object' && err !== null ? JSON.stringify(err) : String(err);
			error(`Failed to clear comics metadata: ${msg}`);
			toast.error(m['pages.home.toast.error.clear_metadata']());
		} finally {
			isProcessing = false;
			activeAction = null;
		}
	}

	async function handleBookmark(categoryId: number) {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'bookmark';
		activeCategoryId = categoryId;
		try {
			await events.onBookmark(selectedIds, categoryId);
			showBookmarkMenu = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = typeof err === 'object' && err !== null ? JSON.stringify(err) : String(err);
			error(`Failed to bookmark comics: ${msg}`);
			toast.error(m['pages.home.toast.error.bookmark']());
		} finally {
			isProcessing = false;
			activeAction = null;
			activeCategoryId = null;
		}
	}

	function handleHideClick() {
		showHideDialog = true;
	}

	function handleDeleteClick() {
		showDeleteDialog = true;
	}

	function handleClearMetadataClick() {
		showClearMetadataDialog = true;
	}
</script>

<AcerolaDialog
	state={{ open }}
	data={{
		title: m['pages.home.actions.title'](),
		description: m['pages.home.actions.description']({ count: selectedIds.length })
	}}
	events={{
		onOpenChange: (isOpen) => {
			if (!isOpen) events.onClose();
		}
	}}
	ui={{
		contentClass:
			'w-full max-w-[calc(100vw-2rem)] sm:max-w-md border-border/60 bg-background/95 backdrop-blur-xl shadow-2xl p-6 rounded-3xl overflow-hidden'
	}}
>
	<div class="flex w-full min-w-0 flex-col gap-4 py-1">
		<!-- Selection Hero Header & Select All Option -->
		{#if events.onSelectAll && totalCount > 0}
			<div
				class="relative overflow-hidden rounded-2xl border border-primary/20 bg-linear-to-br from-primary/10 via-primary/5 to-transparent p-3.5 transition-all duration-300 hover:border-primary/40"
			>
				<div class="flex min-w-0 items-center justify-between gap-3">
					<div class="flex min-w-0 flex-1 items-center gap-3">
						<div
							class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary/15 text-primary shadow-sm"
						>
							<Layers size={20} class="transition-transform duration-300 group-hover:scale-110" />
						</div>
						<div class="flex min-w-0 flex-1 flex-col">
							<span
								class="truncate text-xs font-semibold tracking-wider text-muted-foreground uppercase"
							>
								{selectedIds.length} / {totalCount} Selecionados
							</span>
							<span class="truncate text-sm font-bold tracking-tight text-foreground">
								{isAllSelected ? 'Todos os itens selecionados' : 'Seleção parcial'}
							</span>
						</div>
					</div>

					<AcerolaButton
						ui={{
							variant: isAllSelected ? 'default' : 'outline',
							class:
								'h-9 px-3.5 rounded-xl font-medium text-xs transition-all duration-200 shadow-sm active:scale-95 shrink-0',
							disabled: isProcessing
						}}
						events={{ onClick: events.onSelectAll }}
					>
						{#if isAllSelected}
							<CheckSquare size={16} class="mr-1.5 shrink-0" />
						{:else}
							<Square size={16} class="mr-1.5 shrink-0 opacity-70" />
						{/if}
						<span class="truncate">
							{isAllSelected
								? m['pages.home.selection.all.deselect']()
								: m['pages.home.selection.all.select_total']({ total: totalCount })}
						</span>
					</AcerolaButton>
				</div>
			</div>
		{/if}

		<!-- Bookmark Accordion Section -->
		<div
			class="group relative overflow-hidden rounded-2xl border border-border/80 bg-muted/40 transition-all duration-300 hover:border-border hover:bg-muted/60"
		>
			<AcerolaButton
				ui={{
					variant: 'ghost',
					class:
						'w-full justify-between h-auto py-3.5 px-4 rounded-2xl text-left hover:bg-transparent min-w-0',
					disabled: isProcessing || selectedIds.length === 0
				}}
				events={{ onClick: () => (showBookmarkMenu = !showBookmarkMenu) }}
			>
				<div class="flex min-w-0 flex-1 items-center gap-3 pr-2">
					<div
						class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary transition-colors group-hover:bg-primary/20"
					>
						<Bookmark size={18} />
					</div>
					<div class="flex min-w-0 flex-1 flex-col">
						<span class="truncate text-sm font-semibold text-foreground">
							{m['pages.home.actions.bookmark']()}
						</span>
						<span class="truncate text-xs text-muted-foreground">
							Organize seus quadrinhos selecionados em coleções
						</span>
					</div>
				</div>

				<div class="flex shrink-0 items-center gap-2">
					{#if bookmarks.length > 0}
						<span
							class="shrink-0 rounded-full border border-border bg-surface/80 px-2 py-0.5 text-[11px] font-semibold text-muted-foreground"
						>
							{bookmarks.length}
						</span>
					{/if}
					<div class="shrink-0 text-muted-foreground transition-transform duration-200">
						{#if showBookmarkMenu}
							<ChevronUp size={18} />
						{:else}
							<ChevronDown size={18} />
						{/if}
					</div>
				</div>
			</AcerolaButton>

			{#if showBookmarkMenu}
				<div
					transition:slide={{ duration: 220 }}
					class="min-w-0 border-t border-border/60 bg-background/60 p-2.5"
				>
					{#if bookmarks.length === 0}
						<div class="flex flex-col items-center justify-center gap-1.5 py-4 text-center">
							<span class="text-xs font-medium text-muted-foreground">
								Nenhuma categoria de bookmark criada
							</span>
						</div>
					{:else}
						<!-- Search Filter for 5+ Bookmarks -->
						{#if bookmarks.length > 5}
							<div class="relative mb-2 w-full">
								<Search
									size={14}
									class="absolute top-1/2 left-3 shrink-0 -translate-y-1/2 text-muted-foreground"
								/>
								<input
									type="text"
									placeholder={m['pages.home.actions.filter_categories_placeholder']()}
									bind:value={searchQuery}
									class="h-8 w-full rounded-xl border border-border/60 bg-muted/60 pr-3 pl-8 text-xs text-foreground placeholder:text-muted-foreground focus:border-primary/50 focus:outline-none"
								/>
							</div>
						{/if}

						<div class="max-h-52 w-full min-w-0 space-y-1 overflow-y-auto pr-1">
							{#if filteredBookmarks.length === 0}
								<div class="truncate px-2 py-3 text-center text-xs text-muted-foreground">
									Nenhuma categoria encontrada para "{searchQuery}"
								</div>
							{:else}
								{#each filteredBookmarks as category}
									{@const hexColor =
										'#' + (category.color & 0xffffff).toString(16).padStart(6, '0')}
									<AcerolaButton
										ui={{
											variant: 'ghost',
											class:
												'group/item w-full justify-between h-auto min-h-10 py-2 px-3 rounded-xl text-sm font-medium transition-all duration-200 hover:translate-x-0.5 hover:bg-surface-elevated min-w-0 overflow-hidden',
											disabled: isProcessing || selectedIds.length === 0
										}}
										events={{ onClick: () => handleBookmark(category.id) }}
									>
										<div class="flex min-w-0 flex-1 items-center gap-2.5 pr-2">
											<span
												class="h-3 w-3 shrink-0 rounded-full shadow-sm transition-transform duration-200 group-hover/item:scale-125"
												style="background-color: {hexColor}; box-shadow: 0 0 8px {hexColor}66;"
											></span>
											<span
												class="min-w-0 truncate font-medium text-foreground"
												title={category.name}
											>
												{category.name}
											</span>
										</div>

										{#if isProcessing && activeAction === 'bookmark' && activeCategoryId === category.id}
											<Loader2 size={16} class="shrink-0 animate-spin text-primary" />
										{:else}
											<Check
												size={14}
												class="shrink-0 text-primary opacity-0 transition-opacity group-hover/item:opacity-100"
											/>
										{/if}
									</AcerolaButton>
								{/each}
							{/if}
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Action Buttons Grid: Hide, Clear Metadata & Delete -->
		<div class="grid w-full grid-cols-1 gap-2.5 pt-1">
			<!-- Hide Action -->
			<AcerolaButton
				ui={{
					variant: 'outline',
					class:
						'w-full justify-start h-12 px-4 rounded-2xl text-amber-500 hover:text-amber-400 hover:bg-amber-500/10 border-amber-500/30 hover:border-amber-500/50 font-semibold text-sm transition-all duration-200 active:scale-[0.98] min-w-0',
					disabled: isProcessing || selectedIds.length === 0
				}}
				events={{ onClick: handleHideClick }}
			>
				<div
					class="mr-3 flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-amber-500/15"
				>
					{#if isProcessing && activeAction === 'hide'}
						<Loader2 size={18} class="shrink-0 animate-spin text-amber-500" />
					{:else}
						<EyeOff size={18} class="shrink-0 text-amber-500" />
					{/if}
				</div>
				<div class="flex min-w-0 flex-1 flex-col text-left">
					<span class="truncate">{m['pages.home.actions.hide']()}</span>
				</div>
			</AcerolaButton>

			<!-- Clear Metadata Action -->
			<AcerolaButton
				ui={{
					variant: 'outline',
					class:
						'w-full justify-start h-12 px-4 rounded-2xl text-destructive hover:text-destructive hover:bg-destructive/10 border-destructive/30 hover:border-destructive/50 font-semibold text-sm transition-all duration-200 active:scale-[0.98] min-w-0',
					disabled: isProcessing || selectedIds.length === 0
				}}
				events={{ onClick: handleClearMetadataClick }}
			>
				<div
					class="mr-3 flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-destructive/15"
				>
					{#if isProcessing && activeAction === 'clearMetadata'}
						<Loader2 size={18} class="shrink-0 animate-spin text-destructive" />
					{:else}
						<Eraser size={18} class="shrink-0 text-destructive" />
					{/if}
				</div>
				<div class="flex min-w-0 flex-1 flex-col text-left">
					<span class="truncate">{m['pages.home.actions.clear_metadata']()}</span>
				</div>
			</AcerolaButton>

			<!-- Delete Action -->
			<AcerolaButton
				ui={{
					variant: 'destructive',
					class:
						'w-full justify-start h-12 px-4 rounded-2xl bg-destructive/15 hover:bg-destructive/25 text-destructive border border-destructive/30 hover:border-destructive/60 font-semibold text-sm transition-all duration-200 active:scale-[0.98] min-w-0',
					disabled: isProcessing || selectedIds.length === 0
				}}
				events={{ onClick: handleDeleteClick }}
			>
				<div
					class="mr-3 flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-destructive/20"
				>
					{#if isProcessing && activeAction === 'delete'}
						<Loader2 size={18} class="shrink-0 animate-spin text-destructive" />
					{:else}
						<Trash2 size={18} class="shrink-0 text-destructive" />
					{/if}
				</div>
				<div class="flex min-w-0 flex-1 flex-col text-left">
					<span class="truncate">{m['pages.home.actions.delete']()}</span>
				</div>
			</AcerolaButton>
		</div>
	</div>
</AcerolaDialog>

<!-- Hide Confirmation Alert Dialog -->
<AcerolaAlertDialog
	state={{ open: showHideDialog }}
	data={{
		title: m['pages.home.actions.hide_confirm.title'](),
		description: m['pages.home.actions.hide_confirm.desc'](),
		cancelText: m['pages.home.actions.cancel'](),
		actionText: m['pages.home.actions.hide_confirm.action']()
	}}
	events={{
		onAction: handleHide,
		onCancel: () => (showHideDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>

<!-- Delete Confirmation Alert Dialog -->
<AcerolaAlertDialog
	state={{ open: showDeleteDialog }}
	data={{
		title: m['pages.home.actions.delete_confirm.title'](),
		description: m['pages.home.actions.delete_confirm.desc'](),
		cancelText: m['pages.home.actions.cancel'](),
		actionText: m['pages.home.actions.delete_confirm.action']()
	}}
	events={{
		onAction: handleDelete,
		onCancel: () => (showDeleteDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>

<!-- Clear Metadata Confirmation Alert Dialog -->
<AcerolaAlertDialog
	state={{ open: showClearMetadataDialog }}
	data={{
		title: m['pages.home.actions.clear_metadata_confirm.title'](),
		description: m['pages.home.actions.clear_metadata_confirm.desc']({
			count: selectedIds.length
		}),
		cancelText: m['pages.home.actions.cancel'](),
		actionText: m['pages.home.actions.clear_metadata_confirm.action']()
	}}
	events={{
		onAction: handleClearMetadata,
		onCancel: () => (showClearMetadataDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>
