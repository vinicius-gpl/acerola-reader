<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	type Heading = { id: string; text: string; depth: 2 | 3 };

	let { containerSelector = '.doc-content' }: { containerSelector?: string } = $props();

	let headings = $state<Heading[]>([]);
	let activeId = $state<string | null>(null);

	$effect(() => {
		const container = document.querySelector(containerSelector);
		if (!container) return;

		const nodes = [...container.querySelectorAll('h2, h3')] as HTMLElement[];
		headings = nodes
			.filter((node) => node.id)
			.map((node) => ({
				id: node.id,
				text: node.textContent ?? '',
				depth: node.tagName === 'H3' ? 3 : 2
			}));

		const observer = new IntersectionObserver(
			(observedEntries) => {
				const visible = observedEntries.find((entry) => entry.isIntersecting);
				if (visible) activeId = visible.target.id;
			},
			{ rootMargin: '-96px 0px -70% 0px' }
		);

		for (const node of nodes) observer.observe(node);

		return () => observer.disconnect();
	});
</script>

{#if headings.length > 0}
	<nav class="text-sm">
		<p class="mb-3 text-xs font-semibold tracking-wide text-muted-foreground uppercase">
			{m['toc.title']()}
		</p>
		<ul class="flex flex-col gap-1.5 border-l border-border">
			{#each headings as heading (heading.id)}
				<li style={heading.depth === 3 ? 'padding-left: 1rem' : ''}>
					<a
						href="#{heading.id}"
						class={[
							'-ml-px block border-l-2 py-0.5 pl-3 transition-colors',
							activeId === heading.id
								? 'border-primary font-medium text-primary'
								: 'border-transparent text-muted-foreground hover:text-foreground'
						]}
					>
						{heading.text}
					</a>
				</li>
			{/each}
		</ul>
	</nav>
{/if}
