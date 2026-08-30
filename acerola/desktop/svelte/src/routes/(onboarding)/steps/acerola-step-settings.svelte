<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Button } from '$lib/components/ui/button';
	import CheckIcon from '@lucide/svelte/icons/check';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import { useSelectFolder } from '$lib/hooks/store/use-select-folder.svelte';
	import { useTheme, type ThemeColor } from '$lib/hooks/theme/use-theme.svelte';
	import ThemePicker from '../../config/components/acerola-theme-picker.svelte';

	import { onMount } from 'svelte';

	const themeCtx = useTheme();
	const folder = useSelectFolder();

	onMount(() => {
		folder.loadSavedPath();
	});

	$effect(() => {
		folder.loadSavedPath();
	});

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center overflow-y-auto p-8">
	<div class="w-full max-w-2xl space-y-8">
		<div class="flex items-center justify-center gap-3">
			<PaletteIcon class="text-chart-4" size={32} />
			<h1 class="text-3xl font-bold text-foreground">{m['onboarding.settings.title']()}</h1>
		</div>

		<div class="space-y-6">
			<div class="rounded-2xl border border-border/40 bg-card/50 p-6 backdrop-blur-sm">
				<div class="mb-4 flex items-center gap-3">
					<PaletteIcon class="text-chart-5" size={24} />
					<h2 class="text-xl font-semibold text-foreground">{m['onboarding.settings.theme']()}</h2>
				</div>
				<ThemePicker
					data={{ theme: themeCtx.theme, mode: themeCtx.resolved }}
					ui={{ showHeader: false }}
					events={{ onSelect: (name: ThemeColor) => themeCtx.setTheme(name) }}
				/>
			</div>

			<div class="rounded-2xl border border-border/40 bg-card/50 p-6 backdrop-blur-sm">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-3">
						<FolderIcon class="text-chart-2" size={24} />
						<h2 class="text-xl font-semibold text-foreground">
							{m['onboarding.settings.folder.title']()}
						</h2>
					</div>
					{#if folder.folderPath}
						<div class="flex items-center gap-2 text-sm text-primary">
							<CheckIcon size={16} />
							<span>{m['onboarding.settings.folder.selected']()}</span>
						</div>
					{/if}
				</div>
				<p class="mb-4 text-sm text-muted-foreground">{m['onboarding.settings.folder.desc']()}</p>
				<Button onclick={folder.selectFolder} variant="outline" class="w-full rounded-xl">
					{folder.folderPath || m['onboarding.settings.folder.select']()}
				</Button>
			</div>
		</div>

		<div class="flex justify-between">
			<Button onclick={onPrev} variant="outline" class="rounded-xl">
				{m['onboarding.back']()}
			</Button>
			<Button onclick={onNext} disabled={!folder.folderPath} class="rounded-xl">
				{m['onboarding.next']()}
			</Button>
		</div>
	</div>
</div>
