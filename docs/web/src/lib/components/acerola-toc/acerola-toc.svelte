<script lang="ts">
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/cn.util';
	import { slidingIndicator } from '$lib/utils/sliding-indicator.utils';

	type Heading = { id: string; text: string; depth: 2 | 3 };

	let { containerSelector = '.doc-content' }: { containerSelector?: string } = $props();

	let headings = $state<Heading[]>([]);
	let activeId = $state<string | null>(null);

	$effect(() => {
		// Lê a rota atual só para forçar o efeito a rodar de novo a cada navegação
		// entre páginas de doc — o container `.doc-content` não é remontado porque
		// o Toc vive no layout, então sem essa dependência os headings ficam presos
		// na primeira página visitada.
		page.url.pathname;

		const container = document.querySelector(containerSelector);
		if (!container) return;

		activeId = null;

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
		<ul
			use:slidingIndicator={{
				selector: '[aria-current="true"]',
				indicatorClass: 'w-0.5 rounded-full bg-primary',
				track: ['y', 'height']
			}}
			class="flex flex-col gap-1.5 border-l border-border"
		>
			{#each headings as heading (heading.id)}
				<li style={heading.depth === 3 ? 'padding-left: 1rem' : ''}>
					<a
						href="#{heading.id}"
						aria-current={activeId === heading.id ? 'true' : undefined}
						class={cn(
							'-ml-px block border-l-2 border-transparent py-0.5 pl-3 transition-colors',
							activeId === heading.id
								? 'font-medium text-primary'
								: 'text-muted-foreground hover:text-foreground'
						)}
					>
						{heading.text}
					</a>
				</li>
			{/each}
		</ul>
	</nav>
{/if}
