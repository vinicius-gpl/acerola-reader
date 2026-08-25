<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import { Dialog } from 'bits-ui';
	import { m } from '$lib/paraglide/messages';

	type PagefindResult = { url: string; excerpt: string; meta: { title?: string } };
	type PagefindApi = {
		search: (query: string) => Promise<{ results: { data: () => Promise<PagefindResult> }[] }>;
	};

	let { open = $bindable(false) }: { open: boolean } = $props();

	let inputEl = $state<HTMLInputElement | null>(null);
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

	$effect(() => {
		if (!open) return;
		ensurePagefind();
		inputEl?.focus();
	});

	async function runSearch(value: string) {
		query = value;

		if (!pagefind || !value) {
			results = [];
			return;
		}

		const search = await pagefind.search(value);
		results = await Promise.all(search.results.slice(0, 8).map((result) => result.data()));
	}

	function handleKeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			open = !open;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 z-50 bg-black/40" />
		<Dialog.Content
			class="fixed top-24 left-1/2 z-50 w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 rounded-xl border border-border bg-popover shadow-2xl"
		>
			<Dialog.Title class="sr-only">{m['search.title']()}</Dialog.Title>
			<Dialog.Description class="sr-only">{m['search.title']()}</Dialog.Description>

			<div class="flex items-center gap-2 border-b border-border px-4 py-3">
				<SearchIcon size={16} class="text-muted-foreground" />
				<input
					bind:this={inputEl}
					class="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
					placeholder={m['search.placeholder']()}
					value={query}
					oninput={(event) => runSearch(event.currentTarget.value)}
				/>
			</div>

			<div class="max-h-96 overflow-y-auto p-2 text-sm">
				{#if status === 'unavailable'}
					<p class="p-3 text-muted-foreground">{m['search.unavailable_dev']()}</p>
				{:else if status === 'loading'}
					<p class="p-3 text-muted-foreground">{m['search.loading']()}</p>
				{:else if query && results.length === 0}
					<p class="p-3 text-muted-foreground">{m['search.no_results']()}</p>
				{:else}
					<ul class="flex flex-col gap-1">
						{#each results as result (result.url)}
							<li>
								<a
									href={result.url}
									class="block rounded-md p-2 hover:bg-accent"
									onclick={() => (open = false)}
								>
									<span class="block font-medium">{result.meta?.title ?? result.url}</span>
									<span class="line-clamp-2 text-xs text-muted-foreground"
										>{@html result.excerpt}</span
									>
								</a>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
