<script lang="ts">
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import ServerIcon from '@lucide/svelte/icons/server';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import FaultyTerminal from '$lib/components/faulty-terminal/faulty-terminal.svelte';
	import Card from '$lib/mdsvex/card.svelte';
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import GithubIcon from '$lib/icons/github.svelte';
	import { GITHUB_URL } from '$lib/constants/site';
	import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
	import { m } from '$lib/paraglide/messages';
	import { localizeHref } from '$lib/paraglide/runtime';

	const themeCtx = useTheme();

	// Matches the default theme's dark --primary until the effect below reads the
	// real, currently-active value (light/dark and all 4 palettes have their own).
	let heroTint = $state('#cba6f7');
	let prefersReducedMotion = $state(false);

	$effect(() => {
		prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
	});

	$effect(() => {
		// Re-read whenever the active palette or light/dark mode changes.
		themeCtx.theme;
		themeCtx.resolved;
		heroTint =
			getComputedStyle(document.documentElement).getPropertyValue('--primary').trim() || heroTint;
	});

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

<div class="relative overflow-hidden">
	{#if !prefersReducedMotion}
		<div
			class="pointer-events-none fixed inset-x-0 top-14 bottom-0 -z-10"
			style="mask-image: linear-gradient(to bottom, black 85%, transparent); -webkit-mask-image: linear-gradient(to bottom, black 85%, transparent);"
		>
			<FaultyTerminal
				tint={heroTint}
				scale={2}
				digitSize={1.5}
				timeScale={0.4}
				scanlineIntensity={0.4}
				curvature={0.15}
				brightness={0.85}
				mouseStrength={0.3}
				pageLoadAnimation={false}
				class="h-full w-full"
			/>
		</div>
	{/if}

	<section class="relative">
		<div
			class="relative mx-auto flex max-w-3xl flex-col items-center rounded-3xl px-4 py-24 text-center"
		>
			<div class="absolute inset-0 -z-10 rounded-3xl bg-background/55"></div>
			<Badge variant="outline" class="mb-4 h-auto px-3 py-1 text-xs text-muted-foreground">
				{m['landing.hero_badge']()}
			</Badge>
			<h1 class="font-heading text-4xl font-semibold text-balance sm:text-5xl">
				{m['landing.hero_title']()}
			</h1>
			<p class="mt-6 text-lg text-balance text-muted-foreground">
				{m['landing.hero_subtitle']()}
			</p>
			<div class="mt-8 flex flex-wrap items-center justify-center gap-3">
				<Button href={localizeHref('/docs/getting-started')} size="lg">
					{m['landing.cta_get_started']()}
				</Button>
				<Button href={GITHUB_URL} target="_blank" rel="noreferrer" variant="outline" size="lg">
					<GithubIcon size={16} />
					{m['landing.cta_github']()}
				</Button>
			</div>
		</div>
	</section>

	<section class="relative mx-auto max-w-5xl px-4 pb-24">
		<div class="relative mx-auto mb-8 max-w-xl rounded-2xl bg-background/55 py-3 text-center">
			<h2 class="font-heading text-2xl font-semibold">{m['landing.platforms_title']()}</h2>
			<p class="mt-2 text-muted-foreground">{m['landing.platforms_subtitle']()}</p>
		</div>

		<CardGrid>
			{#each platforms as platform (platform.key)}
				<Card
					title={PLATFORM_LABELS[platform.key].title()}
					icon={platform.icon}
					class="bg-card/60 backdrop-blur-md"
				>
					{PLATFORM_LABELS[platform.key].desc()}
				</Card>
			{/each}
		</CardGrid>
	</section>
</div>
