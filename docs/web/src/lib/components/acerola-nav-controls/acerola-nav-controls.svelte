<script lang="ts">
	import { Button } from '$lib/components/ui/button/index';
	import AcerolaColorPicker from '$lib/components/acerola-color-picker/acerola-color-picker.svelte';
	import AcerolaThemePicker from '$lib/components/acerola-theme-picker/acerola-theme-picker.svelte';
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
	<AcerolaThemePicker />
	<AcerolaColorPicker />
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
