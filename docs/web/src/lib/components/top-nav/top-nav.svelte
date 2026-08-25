<script lang="ts">
	import MenuIcon from '@lucide/svelte/icons/menu';
	import SearchIcon from '@lucide/svelte/icons/search';
	import { Button } from '$lib/components/ui/button/index.js';
	import ColorPicker from '$lib/components/color-picker/color-picker.svelte';
	import ThemePicker from '$lib/components/theme-picker/theme-picker.svelte';
	import GithubIcon from '$lib/icons/github.svelte';
	import { GITHUB_URL } from '$lib/constants/site';
	import { m } from '$lib/paraglide/messages';
	import { getLocale, localizeHref, locales, setLocale, type Locale } from '$lib/paraglide/runtime';

	let { onOpenSearch, onOpenMobileNav }: { onOpenSearch: () => void; onOpenMobileNav: () => void } =
		$props();

	function toggleLocale() {
		const currentIndex = locales.indexOf(getLocale());
		const next = locales[(currentIndex + 1) % locales.length] as Locale;
		setLocale(next);
	}
</script>

<header
	class="sticky top-0 z-40 flex h-14 items-center gap-3 border-b border-border bg-background/80 px-4 backdrop-blur"
>
	<Button
		variant="ghost"
		size="icon"
		class="md:hidden"
		aria-label={m['nav.menu_toggle']()}
		onclick={onOpenMobileNav}
	>
		<MenuIcon size={18} />
	</Button>

	<a href={localizeHref('/')} class="flex items-center gap-2 font-heading text-lg font-semibold">
		<img src="/logo.svg" alt="" class="size-6" />
		<span>{m['site.name']()}</span>
	</a>

	<nav class="ml-2 hidden items-center gap-4 text-sm text-muted-foreground md:flex">
		<a href={localizeHref('/docs/getting-started')} class="hover:text-foreground"
			>{m['nav.docs']()}</a
		>
	</nav>

	<Button
		variant="outline"
		class="ml-4 flex-1 justify-start gap-2 font-normal text-muted-foreground hover:border-primary/40 hover:bg-background md:max-w-xs"
		onclick={onOpenSearch}
	>
		<SearchIcon size={15} />
		<span class="flex-1 text-left">{m['nav.search_placeholder']()}</span>
		<kbd class="rounded border border-border bg-background px-1.5 py-0.5 text-[10px]">Ctrl K</kbd>
	</Button>

	<div class="ml-auto flex items-center gap-1">
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
</header>
