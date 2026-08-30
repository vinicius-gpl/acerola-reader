<script lang="ts">
	import MenuIcon from '@lucide/svelte/icons/menu';
	import SearchIcon from '@lucide/svelte/icons/search';
	import { Button } from '$lib/components/ui/button/index.js';
	import NavControls from '$lib/components/nav-controls/nav-controls.svelte';
	import { m } from '$lib/paraglide/messages';
	import { localizeHref } from '$lib/paraglide/runtime';

	let { onOpenSearch, onOpenMobileNav }: { onOpenSearch: () => void; onOpenMobileNav: () => void } =
		$props();
</script>

<header
	class="sticky top-0 z-40 flex h-14 items-center gap-2 border-b border-border bg-background/80 px-3 backdrop-blur sm:gap-3 sm:px-4"
>
	<Button
		variant="ghost"
		size="icon"
		class="shrink-0 md:hidden"
		aria-label={m['nav.menu_toggle']()}
		onclick={onOpenMobileNav}
	>
		<MenuIcon size={18} />
	</Button>

	<a
		href={localizeHref('/')}
		class="flex shrink-0 items-center gap-2 font-heading text-lg font-semibold"
	>
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
		class="ml-2 hidden h-9 min-w-0 flex-1 justify-start gap-2 border-border bg-muted/40 font-normal text-muted-foreground hover:border-primary/40 hover:bg-muted md:ml-4 md:flex md:max-w-xs"
		onclick={onOpenSearch}
	>
		<SearchIcon size={15} class="shrink-0" />
		<span class="min-w-0 flex-1 truncate text-left">{m['nav.search_placeholder']()}</span>
		<span class="hidden shrink-0 items-center gap-0.5 lg:flex">
			<kbd
				class="rounded border border-border bg-background px-1.5 py-0.5 font-sans text-[10px] font-medium"
				>Ctrl</kbd
			>
			<kbd
				class="rounded border border-border bg-background px-1.5 py-0.5 font-sans text-[10px] font-medium"
				>K</kbd
			>
		</span>
	</Button>

	<Button
		variant="ghost"
		size="icon"
		class="ml-auto shrink-0 md:hidden"
		aria-label={m['nav.search_placeholder']()}
		onclick={onOpenSearch}
	>
		<SearchIcon size={18} />
	</Button>

	<div class="ml-auto hidden shrink-0 md:block">
		<NavControls />
	</div>
</header>
