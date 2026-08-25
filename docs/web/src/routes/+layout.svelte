<script lang="ts">
	import '$theme/layout.css';
	import MobileNav from '$lib/components/mobile-nav/mobile-nav.svelte';
	import SearchDialog from '$lib/components/search-dialog/search-dialog.svelte';
	import TopNav from '$lib/components/top-nav/top-nav.svelte';
	import { getSidebar } from '$lib/content/docs';
	import { getLocale } from '$lib/paraglide/runtime';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let searchOpen = $state(false);
	let mobileNavOpen = $state(false);

	const groups = $derived(getSidebar(getLocale()));
	const activeSlug = $derived(page.url.pathname.split('/docs/')[1] ?? '');
</script>

<TopNav onOpenSearch={() => (searchOpen = true)} onOpenMobileNav={() => (mobileNavOpen = true)} />
<SearchDialog bind:open={searchOpen} />
<MobileNav bind:open={mobileNavOpen} {groups} {activeSlug} />

{@render children()}
