---
title: Storybook
description: UI component catalog for the docs site and the desktop app.
section: External docs
order: 1
---

<script>
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import Card from '$lib/mdsvex/card.svelte';
	import { STORYBOOK_WEB_URL, STORYBOOK_DESKTOP_URL } from '$lib/constants/site';
</script>

Storybook catalogs Acerola's UI components in isolation, by variant and state, outside the app's normal flow — useful for visual review and component testing.

Each Acerola project publishes its own Storybook:

<CardGrid>
	<Card title="Storybook — Web" href={STORYBOOK_WEB_URL}>
		Components from the docs site (`docs/web`).
	</Card>
	<Card title="Storybook — Desktop" href={STORYBOOK_DESKTOP_URL}>
		Components from the desktop app (`acerola/desktop`).
	</Card>
</CardGrid>
