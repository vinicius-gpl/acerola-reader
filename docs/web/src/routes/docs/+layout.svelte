<script lang="ts">
	import Sidebar from '$lib/components/sidebar/sidebar.svelte';
	import Toc from '$lib/components/toc/toc.svelte';
	import { getSidebar } from '$lib/content/docs';
	import { getLocale } from '$lib/paraglide/runtime';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	const groups = $derived(getSidebar(getLocale()));
	const activeSlug = $derived(typeof page.params.slug === 'string' ? page.params.slug : '');
</script>

<div class="mx-auto flex max-w-7xl gap-8 px-4 py-8">
	<aside class="hidden w-56 shrink-0 md:block">
		<div class="sticky top-20">
			<Sidebar {groups} {activeSlug} />
		</div>
	</aside>

	<main class="min-w-0 flex-1">
		{@render children()}
	</main>

	<aside class="hidden w-56 shrink-0 lg:block">
		<div class="sticky top-20">
			<Toc />
		</div>
	</aside>
</div>
