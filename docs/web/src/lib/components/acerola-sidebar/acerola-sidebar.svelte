<script lang="ts">
	import type { Component } from 'svelte';
	import type { SidebarGroup } from '$lib/content/docs';
	import { localizeHref } from '$lib/paraglide/runtime';
	import { cn } from '$lib/cn.util';

	import RocketIcon from '@lucide/svelte/icons/rocket';
	import GitPullRequestIcon from '@lucide/svelte/icons/git-pull-request';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import PackageIcon from '@lucide/svelte/icons/package';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import PaletteIcon from '@lucide/svelte/icons/palette';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';

	let { groups, activeSlug }: { groups: SidebarGroup[]; activeSlug: string } = $props();

	const SECTION_ICONS: Record<string, Component> = {
		'Primeiros passos': RocketIcon,
		'Getting Started': RocketIcon,
		Contribuindo: GitPullRequestIcon,
		Contributing: GitPullRequestIcon,
		Conceitos: NetworkIcon,
		Concepts: NetworkIcon,
		Privacidade: ShieldCheckIcon,
		Privacy: ShieldCheckIcon,
		Bibliotecas: PackageIcon,
		Libraries: PackageIcon,
		'Docs externas': ExternalLinkIcon,
		'External docs': ExternalLinkIcon
	};

	const DOC_ICONS: Record<string, Component> = {
		'getting-started': RocketIcon,
		'contributing-overview': FileTextIcon,
		'contributing-desktop': MonitorIcon,
		'contributing-android': SmartphoneIcon,
		'contributing-p2p': NetworkIcon,
		architecture: CpuIcon,
		'privacy-policy': ShieldCheckIcon,
		libraries: PackageIcon,
		storybook: PaletteIcon
	};
</script>

<nav class="flex flex-col gap-6 text-sm">
	{#each groups as group (group.section)}
		{@const SectionIcon = SECTION_ICONS[group.section]}
		<div>
			<p
				class="mb-2 flex items-center gap-2 px-2 text-xs font-bold tracking-wider text-muted-foreground uppercase"
			>
				{#if SectionIcon}
					<SectionIcon size={13} class="text-primary/70" aria-hidden="true" />
				{/if}
				<span>{group.section}</span>
			</p>
			<ul class="flex flex-col gap-0.5">
				{#each group.docs as doc (doc.slug)}
					{@const DocIcon = DOC_ICONS[doc.slug] ?? BookOpenIcon}
					{@const isActive = doc.slug === activeSlug}
					<li>
						<a
							href={localizeHref(`/docs/${doc.slug}`)}
							class={cn(
								'group flex items-center gap-2.5 rounded-xl px-2.5 py-1.5 text-xs font-medium transition-all',
								isActive
									? 'bg-primary/10 font-semibold text-primary shadow-xs'
									: 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'
							)}
						>
							<DocIcon
								size={14}
								aria-hidden="true"
								class={cn(
									'shrink-0 transition-transform duration-200 group-hover:scale-110',
									isActive ? 'text-primary' : 'text-muted-foreground/70 group-hover:text-foreground'
								)}
							/>
							<span class="truncate">{doc.frontmatter.title}</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>
	{/each}
</nav>
