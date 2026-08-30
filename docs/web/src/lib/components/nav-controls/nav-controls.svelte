<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import ColorPicker from '$lib/components/color-picker/color-picker.svelte';
	import ThemePicker from '$lib/components/theme-picker/theme-picker.svelte';
	import GithubIcon from '$lib/icons/github.svelte';
	import { GITHUB_URL } from '$lib/constants/site';
	import { m } from '$lib/paraglide/messages';
	import { getLocale, locales, setLocale, type Locale } from '$lib/paraglide/runtime';

	function toggleLocale() {
		const currentIndex = locales.indexOf(getLocale());
		const next = locales[(currentIndex + 1) % locales.length] as Locale;
		setLocale(next);
	}
</script>

<div class="flex items-center gap-1">
	<Button variant="ghost" size="sm" onclick={toggleLocale}>
		{getLocale().toUpperCase()}
	</Button>
	<ThemePicker />
	<ColorPicker />
	<Button
		variant="ghost"
		size="icon"
		href={GITHUB_URL}
		target="_blank"
		rel="noreferrer"
		aria-label={m['nav.github']()}
	>
		<GithubIcon size={17} />
	</Button>
</div>
