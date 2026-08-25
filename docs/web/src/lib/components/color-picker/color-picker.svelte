<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { THEMES } from '$lib/constants/themes';
	import { useTheme, type ThemeColor } from '$lib/hooks/theme/use-theme.svelte';
	import { m } from '$lib/paraglide/messages';

	const themeCtx = useTheme();
	const families = Object.keys(THEMES) as ThemeColor[];

	const LABELS: Record<ThemeColor, { name: () => string; desc: () => string }> = {
		catppuccin: {
			name: m['color_picker.catppuccin.name'],
			desc: m['color_picker.catppuccin.desc']
		},
		nord: { name: m['color_picker.nord.name'], desc: m['color_picker.nord.desc'] },
		dracula: { name: m['color_picker.dracula.name'], desc: m['color_picker.dracula.desc'] },
		'tokyo-night': {
			name: m['color_picker.tokyo-night.name'],
			desc: m['color_picker.tokyo-night.desc']
		}
	};
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="ghost"
				size="icon"
				title={m['color_picker.title']()}
				aria-label={m['color_picker.title']()}
			>
				<PaletteIcon size={17} />
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content class="w-64" align="end">
		{#each families as family (family)}
			<DropdownMenu.Item class="items-start gap-2 py-2" onSelect={() => themeCtx.setTheme(family)}>
				<span class="mt-0.5 w-4 shrink-0">
					{#if themeCtx.theme === family}
						<CheckIcon size={16} class="text-primary" />
					{/if}
				</span>
				<span>
					<span class="block font-medium">{LABELS[family].name()}</span>
					<span class="block text-xs text-muted-foreground">{LABELS[family].desc()}</span>
				</span>
			</DropdownMenu.Item>
		{/each}
	</DropdownMenu.Content>
</DropdownMenu.Root>
