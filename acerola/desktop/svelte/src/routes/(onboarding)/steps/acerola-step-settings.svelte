<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import CheckIcon from '@lucide/svelte/icons/check';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import { useSelectFolder } from '$lib/hooks/store/use-select-folder.svelte';

	import { onMount } from 'svelte';

	const folder = useSelectFolder();

	onMount(() => {
		folder.loadSavedPath();
	});

	$effect(() => {
		folder.loadSavedPath();
	});

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center overflow-y-auto p-6 sm:p-8">
	<div class="w-full max-w-xl space-y-8 text-center sm:text-left">
		<!-- Header -->
		<div class="flex flex-col items-center gap-3 sm:flex-row sm:items-start">
			<div
				class="flex size-14 items-center justify-center rounded-2xl bg-chart-4/10 text-chart-4 shadow-inner"
			>
				<FolderIcon size={28} />
			</div>
			<div>
				<h1 class="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
					{m['onboarding.settings.title']()}
				</h1>
				<p class="mt-1 text-sm text-muted-foreground">
					{m['onboarding.settings.folder.desc']()}
				</p>
			</div>
		</div>

		<!-- Dropzone-style Folder Selection Card -->
		<div
			class="relative flex flex-col items-center rounded-3xl border-2 border-dashed p-8 text-center backdrop-blur-md transition-all duration-300 {folder.folderPath
				? 'border-primary/50 bg-primary/5 shadow-[0_0_24px_rgba(var(--color-primary-rgb),0.12)]'
				: 'border-border/70 bg-card/40 hover:border-primary/40 hover:bg-card/60'}"
		>
			<div
				class="mb-4 flex size-16 items-center justify-center rounded-2xl transition-transform duration-300 hover:scale-105 {folder.folderPath
					? 'bg-primary/20 text-primary'
					: 'bg-muted/60 text-muted-foreground'}"
			>
				<FolderIcon size={32} />
			</div>

			<h2 class="mb-1 text-xl font-bold text-foreground">
				{m['onboarding.settings.folder.title']()}
			</h2>

			<p class="mb-5 max-w-sm text-xs text-muted-foreground">
				{m['onboarding.settings.folder.desc']()}
			</p>

			{#if folder.folderPath}
				<div
					class="mb-5 flex max-w-full items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-1.5 text-xs font-semibold text-primary"
				>
					<CheckIcon size={14} strokeWidth={2.5} />
					<span>{m['onboarding.settings.folder.selected']()}</span>
				</div>
			{/if}

			<Button
				onclick={folder.selectFolder}
				variant={folder.folderPath ? 'outline' : 'default'}
				class="w-full max-w-md truncate rounded-2xl px-6 py-5 text-sm font-semibold shadow-xs"
			>
				{folder.folderPath || m['onboarding.settings.folder.select']()}
			</Button>
		</div>

		<!-- Action Controls -->
		<div class="flex items-center justify-between pt-2">
			<Button onclick={onPrev} variant="outline" class="rounded-xl px-6">
				{m['onboarding.back']()}
			</Button>
			<Button
				onclick={onNext}
				disabled={!folder.folderPath}
				class="rounded-xl px-8 font-semibold shadow-md transition-all hover:shadow-lg disabled:opacity-40"
			>
				{m['onboarding.next']()}
			</Button>
		</div>
	</div>
</div>
