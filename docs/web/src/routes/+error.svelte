<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { localizeHref } from '$lib/paraglide/runtime';
</script>

<svelte:head>
	<title>{m['error.404_title']()} — {m['site.name']()}</title>
</svelte:head>

<div class="mx-auto flex max-w-lg flex-col items-center px-4 py-32 text-center">
	<span class="font-heading text-6xl font-semibold text-primary">404</span>
	<h1 class="mt-4 text-2xl font-semibold">{m['error.404_title']()}</h1>
	<p class="mt-2 text-muted-foreground">{m['error.404_desc']()}</p>
	<a
		href={localizeHref('/')}
		class="mt-6 rounded-md bg-primary px-5 py-2.5 text-sm font-medium text-primary-foreground hover:opacity-90"
	>
		{m['error.back_home']()}
	</a>
</div>
