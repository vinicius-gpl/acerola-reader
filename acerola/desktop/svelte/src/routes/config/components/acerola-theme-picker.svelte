<script lang="ts" module>
	import { type ThemeColor, type ThemeMode } from '$lib/hooks/theme/use-theme.svelte';
	import { m } from '$lib/paraglide/messages';

	export type ThemePickerProps = {
		id: ThemeColor;
		name: () => string;
		description: () => string;
		colors: Record<ThemeMode, string[]>;
	};

	const themes: ThemePickerProps[] = [
		{
			id: 'catppuccin',
			name: m['theme.catppuccin.name'],
			description: m['theme.catppuccin.desc'],
			colors: {
				light: ['#8839EF', '#EA76CB', '#1E66F5', '#EFF1F5'],
				dark: ['#CBA6F7', '#F5C2E7', '#89B4FA', '#1E1E2E']
			}
		},
		{
			id: 'nord',
			name: m['theme.nord.name'],
			description: m['theme.nord.desc'],
			colors: {
				light: ['#5E81AC', '#81A1C1', '#88C0D0', '#ECEFF4'],
				dark: ['#88C0D0', '#81A1C1', '#5E81AC', '#2E3440']
			}
		},
		{
			id: 'dracula',
			name: m['theme.dracula.name'],
			description: m['theme.dracula.desc'],
			colors: {
				light: ['#2D005F', '#6272A4', '#005A5F', '#F8F8F2'],
				dark: ['#BD93F9', '#FF79C6', '#8BE9FD', '#282A36']
			}
		},
		{
			id: 'tokyo-night',
			name: m['theme.tokyo-night.name'],
			description: m['theme.tokyo-night.desc'],
			colors: {
				light: ['#2E7DE9', '#7847BD', '#007197', '#E1E2E7'],
				dark: ['#7AA2F7', '#BB9AF7', '#7DCFFF', '#24283B']
			}
		}
	];

	export type ThemePickerComponentProps = {
		data: {
			theme: ThemeColor;
			mode: ThemeMode;
		};
		ui?: {
			showHeader?: boolean;
		};
		events: {
			onSelect: (name: ThemeColor) => void;
		};
	};
</script>

<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import PaletteIcon from '@lucide/svelte/icons/palette';

	let { data, ui = { showHeader: true }, events }: ThemePickerComponentProps = $props();
	let showHeader = $derived(ui.showHeader ?? true);
</script>

<section class="space-y-4">
	{#if showHeader}
		<div
			class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
		>
			<PaletteIcon size={16} />
			{m['pages.config.components.theme_piker']()}
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-3.5 sm:grid-cols-2 lg:grid-cols-2 xl:grid-cols-4">
		{#each themes as it}
			<button
				type="button"
				onclick={() => events.onSelect(it.id)}
				class="group relative flex cursor-pointer flex-col justify-between overflow-hidden rounded-2xl border-2 p-4 text-left transition-all duration-200 focus-visible:ring-2 focus-visible:ring-primary/50 focus-visible:outline-none active:scale-[0.98]
          {data.theme === it.id
					? 'border-primary bg-primary/5 shadow-md shadow-primary/10'
					: 'border-border/60 bg-card hover:border-muted-foreground/50 hover:bg-card/90 hover:shadow-sm'}"
			>
				<div>
					<div class="mb-3 flex items-center justify-between">
						<div class="flex -space-x-1.5 overflow-hidden p-0.5">
							{#each it.colors[data.mode] as color}
								<div
									class="h-7 w-7 rounded-full border-2 border-card shadow-sm transition-transform duration-200 group-hover:scale-105"
									style="background-color: {color}"
								></div>
							{/each}
						</div>
					</div>

					<h3 class="text-sm font-bold text-foreground transition-colors group-hover:text-primary">
						{it.name()}
					</h3>
					<p class="mt-1 line-clamp-2 text-xs leading-snug text-muted-foreground">
						{it.description()}
					</p>
				</div>

				{#if data.theme === it.id}
					<div
						class="absolute top-3.5 right-3.5 flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-sm"
					>
						<CheckIcon size={12} strokeWidth={3} />
					</div>
				{/if}
			</button>
		{/each}
	</div>
</section>
