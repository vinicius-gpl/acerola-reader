<script lang="ts">
	import CloudOffIcon from '@lucide/svelte/icons/cloud-off';
	import CodeIcon from '@lucide/svelte/icons/code';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import ServerIcon from '@lucide/svelte/icons/server';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';
	import { Badge } from '$lib/components/ui/badge/index';
	import { Button } from '$lib/components/ui/button/index';
	import AcerolaApkDownloadButton from '$lib/components/acerola-apk-download-button/acerola-apk-download-button.svelte';
	import AcerolaCallout from '$lib/components/acerola-callout/acerola-callout.svelte';
	import AcerolaMicrosoftStoreButton from '$lib/components/acerola-microsoft-store-button/acerola-microsoft-store-button.svelte';
	import AcerolaDotField from '$lib/components/acerola-dot-field/acerola-dot-field.svelte';
	import AcerolaShinyText from '$lib/components/acerola-shiny-text/acerola-shiny-text.svelte';
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import PlatformCard from '$lib/mdsvex/platform-card.svelte';
	import Steps from '$lib/mdsvex/steps.svelte';
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

	const features = [
		{ key: 'no_cloud', icon: CloudOffIcon },
		{ key: 'p2p_sync', icon: WaypointsIcon },
		{ key: 'open_source', icon: CodeIcon }
	] as const;

	const FEATURE_LABELS = {
		no_cloud: {
			title: m['landing.features.no_cloud.title'],
			desc: m['landing.features.no_cloud.desc']
		},
		p2p_sync: {
			title: m['landing.features.p2p_sync.title'],
			desc: m['landing.features.p2p_sync.desc']
		},
		open_source: {
			title: m['landing.features.open_source.title'],
			desc: m['landing.features.open_source.desc']
		}
	} as const;

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

<div class="relative">
	{#if !prefersReducedMotion}
		<div
			class="fixed inset-x-0 top-14 bottom-0 -z-10 transition-opacity duration-500 ease-out"
			class:opacity-0={!bgReady}
			style="mask-image: linear-gradient(to bottom, black 85%, transparent); -webkit-mask-image: linear-gradient(to bottom, black 85%, transparent);"
		>
			<AcerolaDotField
				dotRadius={5}
				dotSpacing={10}
				cursorRadius={600}
				cursorForce={0.4}
				bulgeOnly={true}
				bulgeStrength={150}
				glowRadius={190}
				sparkle={true}
				waveAmplitude={0}
				gradientFrom="rgba(255, 62, 0, 0.35)"
				gradientTo="rgba(255, 176, 137, 0.25)"
				glowColor="#14110E"
				class="h-full w-full"
			/>
		</div>
	{/if}

	<div
		class="relative z-10 mx-auto max-w-[90rem] px-4 sm:px-6 lg:grid lg:grid-cols-[minmax(440px,38%)_1fr] lg:items-start lg:gap-24 lg:px-8"
	>
		<!-- Coluna fixa: título + botões de download, gruda no topo enquanto a
		     coluna de introdução rola ao lado. -->
		<div
			class="flex flex-col items-center py-16 text-center sm:py-20 lg:sticky lg:top-14 lg:items-start lg:py-24 lg:text-left"
		>
			<Badge variant="secondary" class="mb-4">{m['landing.hero_kicker']()}</Badge>

			<h1
				class="font-heading text-3xl font-semibold text-balance sm:text-4xl md:text-5xl lg:text-6xl"
			>
				<AcerolaShinyText text={m['landing.hero_title']()} />
			</h1>

			<p class="mt-4 max-w-xl text-base text-muted-foreground sm:text-lg lg:text-xl">
				{m['landing.hero_subtitle']()}
			</p>

			<div class="mt-8 grid w-full max-w-md grid-cols-2 gap-3">
				<AcerolaApkDownloadButton />
				<AcerolaMicrosoftStoreButton />
			</div>

			<div class="mt-4 flex flex-wrap items-center justify-center gap-3 lg:justify-start">
				<Button href={localizeHref('/docs/getting-started')} variant="link">
					{m['landing.cta_get_started']()}
				</Button>
				<Button href={GITHUB_URL} target="_blank" rel="noreferrer" variant="ghost">
					<GithubIcon size={16} />
					{m['landing.cta_github']()}
				</Button>
			</div>

			<div class="mt-8 flex flex-wrap items-center justify-center gap-2 lg:justify-start">
				<Badge variant="outline">{m['landing.facts.no_cloud']()}</Badge>
				<Badge variant="outline">{m['landing.facts.no_account']()}</Badge>
				<Badge variant="outline">{m['landing.facts.open_source']()}</Badge>
			</div>
		</div>

		<!-- Coluna de introdução: rola normalmente ao lado da coluna fixa. -->
		<div class="flex flex-col gap-24 pb-16 sm:pb-20 lg:py-24">
			<section>
				<h2 class="mb-8 text-center font-heading text-2xl font-semibold lg:text-left">
					{m['landing.features.title']()}
				</h2>

				<CardGrid>
					{#each features as feature (feature.key)}
						<PlatformCard
							title={FEATURE_LABELS[feature.key].title()}
							description={FEATURE_LABELS[feature.key].desc()}
							icon={feature.icon}
						/>
					{/each}
				</CardGrid>
			</section>

			<section>
				<h2 class="mb-8 text-center font-heading text-2xl font-semibold lg:text-left">
					{m['landing.how_it_works.title']()}
				</h2>

				<Steps>
					<ol>
						<li>{m['landing.how_it_works.install']()}</li>
						<li>{m['landing.how_it_works.pair']()}</li>
						<li>{m['landing.how_it_works.confirm']()}</li>
					</ol>
				</Steps>

				<AcerolaCallout type="tip" title={m['landing.how_it_works.note_title']()}>
					<p>{m['landing.how_it_works.note_desc']()}</p>
				</AcerolaCallout>
			</section>

			<section>
				<h2 class="mb-8 text-center font-heading text-2xl font-semibold lg:text-left">
					{m['landing.platforms_title']()}
				</h2>

				<CardGrid>
					{#each platforms as platform (platform.key)}
						<PlatformCard
							title={PLATFORM_LABELS[platform.key].title()}
							description={PLATFORM_LABELS[platform.key].desc()}
							icon={platform.icon}
						/>
					{/each}
				</CardGrid>
			</section>
		</div>
	</div>
</div>
