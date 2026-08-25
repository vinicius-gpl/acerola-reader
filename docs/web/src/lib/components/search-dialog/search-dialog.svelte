<script lang="ts">
	import * as Command from '$lib/components/ui/command/index.js';
	import { m } from '$lib/paraglide/messages';

	type PagefindResult = { url: string; excerpt: string; meta: { title?: string } };
	type PagefindApi = {
		search: (query: string) => Promise<{ results: { data: () => Promise<PagefindResult> }[] }>;
	};

	let { open = $bindable(false) }: { open: boolean } = $props();

	let query = $state('');
	let results = $state<PagefindResult[]>([]);
	let status = $state<'idle' | 'loading' | 'unavailable'>('idle');
	let pagefind: PagefindApi | null = null;

	async function ensurePagefind() {
		if (pagefind || status === 'unavailable') return;

		// The Pagefind index only exists in the built output (see `npm run build`), so
		// there's nothing to fetch in dev — skip straight to the "unavailable" state
		// instead of letting Vite's dev-server import analysis choke on a missing file.
		if (import.meta.env.DEV) {
			status = 'unavailable';
			return;
		}

		status = 'loading';
		try {
			// Built from a runtime string (not a literal) so Vite/Rollup can't try to
			// resolve this at build time — the file only exists in the deployed output.
			const pagefindEntry = ['', 'pagefind', 'pagefind.js'].join('/');
			pagefind = (await import(/* @vite-ignore */ pagefindEntry)) as unknown as PagefindApi;
			status = 'idle';
		} catch {
			status = 'unavailable';
		}
	}

	async function runSearch(value: string) {
		if (!pagefind || !value) {
			results = [];
			return;
		}

		const search = await pagefind.search(value);
		results = await Promise.all(search.results.slice(0, 8).map((result) => result.data()));
	}

	$effect(() => {
		if (!open) return;
		ensurePagefind();
	});

	$effect(() => {
		runSearch(query);
	});

	function handleKeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			open = !open;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<Command.Dialog
	bind:open
	shouldFilter={false}
	title={m['search.title']()}
	description={m['search.title']()}
>
	<Command.Input bind:value={query} placeholder={m['search.placeholder']()} />
	<Command.List>
		{#if status === 'unavailable'}
			<Command.Empty>{m['search.unavailable_dev']()}</Command.Empty>
		{:else if status === 'loading'}
			<Command.Empty>{m['search.loading']()}</Command.Empty>
		{:else if query && results.length === 0}
			<Command.Empty>{m['search.no_results']()}</Command.Empty>
		{:else}
			{#each results as result (result.url)}
				<Command.LinkItem href={result.url} onclick={() => (open = false)}>
					<div class="flex min-w-0 flex-col gap-0.5">
						<span class="truncate font-medium">{result.meta?.title ?? result.url}</span>
						<span class="line-clamp-2 text-xs text-muted-foreground">{@html result.excerpt}</span>
					</div>
				</Command.LinkItem>
			{/each}
		{/if}
	</Command.List>
</Command.Dialog>
