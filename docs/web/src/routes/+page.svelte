<script lang="ts">
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import ServerIcon from '@lucide/svelte/icons/server';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import Card from '$lib/mdsvex/card.svelte';
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import GithubIcon from '$lib/icons/github.svelte';
	import { GITHUB_URL } from '$lib/constants/site';
	import { m } from '$lib/paraglide/messages';
	import { localizeHref } from '$lib/paraglide/runtime';

	const platforms = [
		{ key: 'android', icon: SmartphoneIcon },
		{ key: 'desktop', icon: MonitorIcon },
		{ key: 'relay', icon: ServerIcon },
		{ key: 'p2p', icon: NetworkIcon },
		{ key: 'relay_lib', icon: RadioIcon }
	] as const;

	const PLATFORM_LABELS = {
		android: {
			title: m['landing.platform.android.title'],
			desc: m['landing.platform.android.desc']
		},
		desktop: {
			title: m['landing.platform.desktop.title'],
			desc: m['landing.platform.desktop.desc']
		},
		relay: { title: m['landing.platform.relay.title'], desc: m['landing.platform.relay.desc'] },
		p2p: { title: m['landing.platform.p2p.title'], desc: m['landing.platform.p2p.desc'] },
		relay_lib: {
			title: m['landing.platform.relay_lib.title'],
			desc: m['landing.platform.relay_lib.desc']
		}
	} as const;
</script>

<svelte:head>
	<title>{m['site.name']()} — {m['landing.hero_badge']()}</title>
</svelte:head>

<section class="mx-auto flex max-w-3xl flex-col items-center px-4 py-24 text-center">
	<span
		class="mb-4 rounded-full border border-border bg-muted/40 px-3 py-1 text-xs text-muted-foreground"
	>
		{m['landing.hero_badge']()}
	</span>
	<h1 class="font-heading text-4xl font-semibold text-balance sm:text-5xl">
		{m['landing.hero_title']()}
	</h1>
	<p class="mt-6 text-lg text-balance text-muted-foreground">
		{m['landing.hero_subtitle']()}
	</p>
	<div class="mt-8 flex flex-wrap items-center justify-center gap-3">
		<a
			href={localizeHref('/docs/getting-started')}
			class="rounded-md bg-primary px-5 py-2.5 text-sm font-medium text-primary-foreground transition-opacity hover:opacity-90"
		>
			{m['landing.cta_get_started']()}
		</a>
		<a
			href={GITHUB_URL}
			target="_blank"
			rel="noreferrer"
			class="flex items-center gap-2 rounded-md border border-border px-5 py-2.5 text-sm font-medium transition-colors hover:bg-accent"
		>
			<GithubIcon size={16} />
			{m['landing.cta_github']()}
		</a>
	</div>
</section>

<section class="mx-auto max-w-5xl px-4 pb-24">
	<div class="mb-8 text-center">
		<h2 class="font-heading text-2xl font-semibold">{m['landing.platforms_title']()}</h2>
		<p class="mt-2 text-muted-foreground">{m['landing.platforms_subtitle']()}</p>
	</div>

	<CardGrid>
		{#each platforms as platform (platform.key)}
			<Card title={PLATFORM_LABELS[platform.key].title()} icon={platform.icon}>
				{PLATFORM_LABELS[platform.key].desc()}
			</Card>
		{/each}
	</CardGrid>
</section>
