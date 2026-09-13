<script lang="ts" module>
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import type { ThemeModeOption } from '$lib/hooks/theme/use-theme.svelte';
	import type { Component } from 'svelte';

	export type ModePickerProps = {
		icon: Component;
		next: ThemeModeOption;
	};

	const MODE_CONFIG: Record<ThemeModeOption, ModePickerProps> = {
		light: { icon: SunIcon, next: 'dark' },
		dark: { icon: MoonIcon, next: 'system' },
		system: { icon: MonitorIcon, next: 'light' }
	} as const;
</script>

<script lang="ts">
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
	import { m } from '$lib/paraglide/messages';

	const themeCtx = useTheme();

	function nextMode() {
		themeCtx.setMode(MODE_CONFIG[themeCtx.mode].next as any);
	}
</script>

<AcerolaButtonIcon
	events={{ onClick: nextMode }}
	ui={{ title: m['components.mode_picker.title'](), tone: 'accent' }}
>
	{#key themeCtx.mode}
		{@const Icon = MODE_CONFIG[themeCtx.mode].icon}
		<Icon size={16} />
	{/key}
</AcerolaButtonIcon>
