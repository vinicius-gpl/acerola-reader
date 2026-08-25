<script lang="ts">
	import type { SidebarGroup } from '$lib/content/docs';
	import { localizeHref } from '$lib/paraglide/runtime';
	import { cn } from '$lib/utils';

	let { groups, activeSlug }: { groups: SidebarGroup[]; activeSlug: string } = $props();
</script>

<nav class="flex flex-col gap-6 text-sm">
	{#each groups as group (group.section)}
		<div>
			<p class="mb-2 px-2 text-xs font-semibold tracking-wide text-muted-foreground uppercase">
				{group.section}
			</p>
			<ul class="flex flex-col gap-0.5">
				{#each group.docs as doc (doc.slug)}
					<li>
						<a
							href={localizeHref(`/docs/${doc.slug}`)}
							class={cn(
								'block rounded-md px-2 py-1.5 transition-colors',
								doc.slug === activeSlug
									? 'bg-primary/10 font-medium text-primary'
									: 'text-muted-foreground hover:bg-accent hover:text-foreground'
							)}
						>
							{doc.frontmatter.title}
						</a>
					</li>
				{/each}
			</ul>
		</div>
	{/each}
</nav>
