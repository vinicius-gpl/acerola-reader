<script lang="ts">
	import MenuIcon from '@lucide/svelte/icons/menu';
	import SearchIcon from '@lucide/svelte/icons/search';
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
	<button
		type="button"
		class="inline-flex size-9 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground md:hidden"
		aria-label={m['nav.menu_toggle']()}
		onclick={onOpenMobileNav}
	>
		<MenuIcon size={18} />
	</button>

	<a href={localizeHref('/')} class="flex items-center gap-2 font-heading text-lg font-semibold">
		<img src="/logo.svg" alt="" class="size-6" />
		<span>{m['site.name']()}</span>
	</a>

	<nav class="ml-2 hidden items-center gap-4 text-sm text-muted-foreground md:flex">
		<a href={localizeHref('/docs/getting-started')} class="hover:text-foreground"
			>{m['nav.docs']()}</a
		>
	</nav>

	<button
		type="button"
		class="ml-4 flex flex-1 items-center gap-2 rounded-md border border-border bg-muted/40 px-3 py-1.5 text-sm text-muted-foreground hover:border-primary/40 md:max-w-xs"
		onclick={onOpenSearch}
	>
		<SearchIcon size={15} />
		<span class="flex-1 text-left">{m['nav.search_placeholder']()}</span>
		<kbd class="rounded border border-border bg-background px-1.5 py-0.5 text-[10px]">Ctrl K</kbd>
	</button>

	<div class="ml-auto flex items-center gap-1">
		<button
			type="button"
			class="rounded-md px-2 py-1 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-foreground"
			onclick={toggleLocale}
		>
			{getLocale().toUpperCase()}
		</button>
		<ThemePicker />
		<ColorPicker />
		<a
			href={GITHUB_URL}
			target="_blank"
			rel="noreferrer"
			class="inline-flex size-9 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
			aria-label={m['nav.github']()}
		>
			<GithubIcon size={17} />
		</a>
	</div>
</header>
