<script lang="ts">
	import '$theme/layout.css';
	import MobileNav from '$lib/components/mobile-nav/mobile-nav.svelte';
	import SearchDialog from '$lib/components/search-dialog/search-dialog.svelte';
	import TopNav from '$lib/components/top-nav/top-nav.svelte';
	import { GITHUB_URL, OG_IMAGE_URL, SITE_URL } from '$lib/constants/site';
	import { getSidebar } from '$lib/content/docs';
	import { m } from '$lib/paraglide/messages';
	import { getLocale, localizeHref, locales } from '$lib/paraglide/runtime';
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let searchOpen = $state(false);
	let mobileNavOpen = $state(false);

	const groups = $derived(getSidebar(getLocale()));
	const activeSlug = $derived(page.url.pathname.split('/docs/')[1] ?? '');

	const jsonLd = $derived({
		'@context': 'https://schema.org',
		'@graph': [
			{
				'@type': 'WebSite',
				'@id': `${SITE_URL}/#website`,
				url: SITE_URL,
				name: m['site.name'](),
				description: m['site.description'](),
				inLanguage: locales.map((locale) => (locale === 'pt-br' ? 'pt-BR' : 'en-US'))
			},
			{
				'@type': 'SoftwareApplication',
				'@id': `${SITE_URL}/#software`,
				name: 'Acerola',
				applicationCategory: 'MultimediaApplication',
				operatingSystem: 'Android, Windows, macOS, Linux',
				description: m['site.description'](),
				url: SITE_URL,
				codeRepository: GITHUB_URL,
				license: 'https://www.mozilla.org/en-US/MPL/2.0/',
				isPartOf: { '@id': `${SITE_URL}/#website` }
			},
			{
				'@type': 'WebPage',
				'@id': page.url.href,
				url: page.url.href,
				name: m['site.name'](),
				isPartOf: { '@id': `${SITE_URL}/#website` },
				about: { '@id': `${SITE_URL}/#software` }
			}
		]
	});

	// Svelte's compiler scans the whole file for script-tag boundaries as plain
	// text before it understands JS string context, so the tag markup can't be
	// written as a literal anywhere in this block, even split across a template
	// literal or tucked in a comment — it has to be built from separate pieces
	// that never form the literal substring. This goes through {@html} (not a
	// `use:` action) so the tag is actually present in the server-rendered HTML;
	// a `use:` action only runs client-side, leaving it empty for crawlers that
	// don't execute JS (GPTBot, ClaudeBot, PerplexityBot). The payload also has
	// its opening angle brackets escaped so no value can prematurely close the tag.
	const TAG_OPEN = ['<', 'script type="application/ld+json">'].join('');
	const TAG_CLOSE = ['<', '/', 'script>'].join('');
	const jsonLdScript = $derived(
		TAG_OPEN + JSON.stringify(jsonLd).replace(/</g, '\\u003c') + TAG_CLOSE
	);
</script>

<svelte:head>
	<!-- Core SEO Meta Tags -->
	<meta name="description" content={m['site.description']()} />
	<meta name="keywords" content={m['site.keywords']()} />
	<meta name="author" content="Vinícius GPL" />
	<meta name="robots" content="index, follow" />
	<meta name="theme-color" content="#1e1e2e" />

	<!-- Canonical & Multilingual Alternate Links -->
	<link rel="canonical" href={page.url.href} />
	{#each locales as loc (loc)}
		<link rel="alternate" hreflang={loc} href={localizeHref(page.url.pathname, { locale: loc })} />
	{/each}
	<link
		rel="alternate"
		hreflang="x-default"
		href={localizeHref(page.url.pathname, { locale: 'pt-br' })}
	/>

	<!-- Open Graph Meta Tags -->
	<meta property="og:site_name" content={m['site.name']()} />
	<meta property="og:title" content={m['site.og_title']()} />
	<meta property="og:description" content={m['site.og_description']()} />
	<meta property="og:type" content="website" />
	<meta property="og:url" content={page.url.href} />
	<meta property="og:locale" content={getLocale() === 'pt-br' ? 'pt_BR' : 'en_US'} />
	<meta property="og:image" content={OG_IMAGE_URL} />

	<!-- Twitter Card Meta Tags -->
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content={m['site.og_title']()} />
	<meta name="twitter:description" content={m['site.og_description']()} />
	<meta name="twitter:image" content={OG_IMAGE_URL} />

	<!-- JSON-LD Structured Data for Google & AI Engines. -->
	{@html jsonLdScript}
</svelte:head>

<TopNav onOpenSearch={() => (searchOpen = true)} onOpenMobileNav={() => (mobileNavOpen = true)} />
<SearchDialog bind:open={searchOpen} />
<MobileNav bind:open={mobileNavOpen} {groups} {activeSlug} />

{@render children()}
