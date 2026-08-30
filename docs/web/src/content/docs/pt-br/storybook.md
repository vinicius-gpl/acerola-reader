---
title: Storybook
description: Catálogo de componentes de UI do site de docs e do app desktop.
section: Docs externas
order: 1
---

<script>
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import Card from '$lib/mdsvex/card.svelte';
	import { STORYBOOK_WEB_URL, STORYBOOK_DESKTOP_URL } from '$lib/constants/site';
</script>

O Storybook cataloga os componentes de UI do Acerola isolados por variante e estado, fora do fluxo normal do app — útil para revisão visual e testes de componente.

Cada projeto do Acerola publica o seu próprio Storybook:

<CardGrid>
	<Card title="Storybook — Web" href={STORYBOOK_WEB_URL}>
		Componentes do site de documentação (`docs/web`).
	</Card>
	<Card title="Storybook — Desktop" href={STORYBOOK_DESKTOP_URL}>
		Componentes do app desktop (`acerola/desktop`).
	</Card>
</CardGrid>
