<script lang="ts">
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import type { DocEntry } from '$lib/content/docs';
	import { m } from '$lib/paraglide/messages';
	import { localizeHref } from '$lib/paraglide/runtime';
	import { cn } from '$lib/cn.util';

	let {
		prev,
		next,
		position = 'bottom'
	}: { prev: DocEntry | null; next: DocEntry | null; position?: 'top' | 'bottom' } = $props();
</script>

{#if prev || next}
	<div
		class={cn(
			'grid grid-cols-1 gap-3 sm:grid-cols-2',
			position === 'bottom'
				? 'mt-12 border-t border-border pt-6'
				: 'mb-10 border-b border-border pb-6'
		)}
	>
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
