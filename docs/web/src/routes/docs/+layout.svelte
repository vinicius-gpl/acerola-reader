<script lang="ts">
	import AcerolaSidebar from '$lib/components/acerola-sidebar/acerola-sidebar.svelte';
	import AcerolaToc from '$lib/components/acerola-toc/acerola-toc.svelte';
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
			<AcerolaSidebar {groups} {activeSlug} />
		</div>
	</aside>

	<main class="min-w-0 flex-1">
		{@render children()}
	</main>

	<aside class="hidden w-56 shrink-0 lg:block">
		<div class="sticky top-20">
			<AcerolaToc />
		</div>
	</aside>
</div>
