<script lang="ts">
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import type { DocEntry } from '$lib/content/docs';
	import { m } from '$lib/paraglide/messages';
	import { localizeHref } from '$lib/paraglide/runtime';

	let { prev, next }: { prev: DocEntry | null; next: DocEntry | null } = $props();
</script>

{#if prev || next}
	<div class="mt-12 grid grid-cols-1 gap-3 border-t border-border pt-6 sm:grid-cols-2">
		{#if prev}
			<a
				href={localizeHref(`/docs/${prev.slug}`)}
				class="group rounded-lg border border-border p-4 transition-colors hover:border-primary/50 hover:bg-accent"
			>
				<span class="flex items-center gap-1 text-xs text-muted-foreground">
					<ArrowLeftIcon size={14} />
					{m['prev_next.previous']()}
				</span>
				<span class="mt-1 block font-medium">{prev.frontmatter.title}</span>
			</a>
		{:else}
			<div></div>
		{/if}

		{#if next}
			<a
				href={localizeHref(`/docs/${next.slug}`)}
				class="group rounded-lg border border-border p-4 text-right transition-colors hover:border-primary/50 hover:bg-accent"
			>
				<span class="flex items-center justify-end gap-1 text-xs text-muted-foreground">
					{m['prev_next.next']()}
					<ArrowRightIcon size={14} />
				</span>
				<span class="mt-1 block font-medium">{next.frontmatter.title}</span>
			</a>
		{/if}
	</div>
{/if}
