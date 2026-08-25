---
title: Arquitetura
description: Como Android, Desktop, Relay e as libs compartilhadas se conectam.
section: Conceitos
order: 1
---

<script>
	import Callout from '$lib/mdsvex/callout.svelte';
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import Card from '$lib/mdsvex/card.svelte';
</script>

<Callout type="note" title="Página de exemplo">

Este conteúdo é um placeholder para validar o pipeline de documentação. A documentação real de arquitetura ainda será escrita.

</Callout>

O Acerola é um **monorepo**: cada plataforma vive isolada, com sua própria stack, e nenhuma depende diretamente de outra. Todas consomem as bibliotecas compartilhadas em `lib/` via dependência de caminho local — uma mudança em `lib/p2p/` já reflete direto nos consumidores.

<CardGrid>
	<Card title="acerola/android">Kotlin + Jetpack Compose</Card>
	<Card title="acerola/desktop">Rust + Tauri + Svelte 5</Card>
	<Card title="acerola/relay">Serviço de acesso remoto (Rust)</Card>
	<Card title="lib/p2p">Biblioteca P2P compartilhada (iroh / QUIC)</Card>
	<Card title="lib/relay">Código do relay iroh vendorizado</Card>
</CardGrid>

## Como as peças se conectam

`acerola/android/` e `acerola/desktop/` nunca se enxergam diretamente — cada um implementa sua própria FFI/binding para consumir `lib/p2p/`. Toda comunicação entre dispositivos é P2P direta (LAN via mDNS) ou via o relay quando fora da mesma rede.

```text
Android  ──┐
           ├──> lib/p2p (biblioteca compartilhada) ──> Relay (acesso remoto, sem abrir portas)
Desktop  ──┘
```
