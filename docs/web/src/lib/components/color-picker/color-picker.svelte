<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import { DropdownMenu } from 'bits-ui';
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
	<DropdownMenu.Trigger
		class="inline-flex size-9 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
		title={m['color_picker.title']()}
		aria-label={m['color_picker.title']()}
	>
		<PaletteIcon size={17} />
	</DropdownMenu.Trigger>
	<DropdownMenu.Portal>
		<DropdownMenu.Content
			class="z-50 w-64 rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-lg"
			sideOffset={8}
			align="end"
		>
			{#each families as family (family)}
				<DropdownMenu.Item
					class="flex cursor-pointer items-start gap-2 rounded-md p-2 text-sm outline-none hover:bg-accent"
					onSelect={() => themeCtx.setTheme(family)}
				>
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
	</DropdownMenu.Portal>
</DropdownMenu.Root>
