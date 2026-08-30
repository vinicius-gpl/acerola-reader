<script lang="ts">
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import ServerIcon from '@lucide/svelte/icons/server';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import { Button } from '$lib/components/ui/button/index.js';
	import AcerolaFaultyTerminal from '$lib/components/acerola-faulty-terminal/acerola-faulty-terminal.svelte';
	import AcerolaShinyText from '$lib/components/acerola-shiny-text/acerola-shiny-text.svelte';
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
	let bgReady = $state(false);

	$effect(() => {
		prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
	});

	$effect(() => {
		// FaultyTerminal has no "ready" signal of its own — a short delay before
		// fading it in avoids the pop-in flash while its WebGL context spins up.
		const timeout = setTimeout(() => (bgReady = true), 300);
		return () => clearTimeout(timeout);
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
	<title>{m['site.name']()} — {m['nav.docs']()}</title>
</svelte:head>

<div class="relative overflow-hidden">
	{#if !prefersReducedMotion}
		<div
			class="fixed inset-x-0 top-14 bottom-0 -z-10 transition-opacity duration-500 ease-out"
			class:opacity-0={!bgReady}
			style="mask-image: linear-gradient(to bottom, black 85%, transparent); -webkit-mask-image: linear-gradient(to bottom, black 85%, transparent);"
		>
			<AcerolaFaultyTerminal
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
		<div class="mx-auto flex max-w-2xl flex-col items-center px-4 py-16 text-center sm:py-20">
			<h1 class="font-heading text-3xl font-semibold text-balance sm:text-4xl md:text-5xl">
				<AcerolaShinyText text={m['landing.hero_title']()} />
			</h1>
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

	<section class="relative mx-auto max-w-5xl px-4 pb-16 sm:pb-24">
		<div class="mx-auto mb-8 max-w-xl text-center">
			<h2 class="font-heading text-2xl font-semibold">{m['landing.platforms_title']()}</h2>
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
