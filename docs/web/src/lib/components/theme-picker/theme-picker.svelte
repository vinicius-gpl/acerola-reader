<script lang="ts" module>
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import type { ThemeModeOption } from '$lib/hooks/theme/use-theme.svelte';
	import type { Component } from 'svelte';

	const MODE_CONFIG: Record<ThemeModeOption, { icon: Component; next: ThemeModeOption }> = {
		light: { icon: SunIcon, next: 'dark' },
		dark: { icon: MoonIcon, next: 'system' },
		system: { icon: MonitorIcon, next: 'light' }
	};
</script>

<script lang="ts">
	import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
	import { m } from '$lib/paraglide/messages';

	const themeCtx = useTheme();

	function nextMode() {
		themeCtx.setMode(MODE_CONFIG[themeCtx.mode].next);
	}
</script>

<button
	type="button"
	class="inline-flex size-9 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
	title={m['theme_picker.title']()}
	aria-label={m['theme_picker.title']()}
	onclick={nextMode}
>
	{#key themeCtx.mode}
		{@const Icon = MODE_CONFIG[themeCtx.mode].icon}
		<Icon size={17} />
	{/key}
</button>
