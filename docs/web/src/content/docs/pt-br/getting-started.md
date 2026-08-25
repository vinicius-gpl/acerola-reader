---
title: Primeiros passos
description: Instale o Acerola e sincronize sua primeira biblioteca entre dois dispositivos.
section: Primeiros passos
order: 1
---

<script>
	import Callout from '$lib/mdsvex/callout.svelte';
	import Steps from '$lib/mdsvex/steps.svelte';
	import CodeGroup from '$lib/mdsvex/code-group.svelte';
</script>

<Callout type="note" title="Página de exemplo">

Este conteúdo é um placeholder para validar o pipeline de documentação (mdsvex + componentes + i18n). A documentação real de cada plataforma ainda será escrita.

</Callout>

O Acerola é um leitor de quadrinhos e mangás locais com sincronização **100% P2P** entre dispositivos — sem servidor central, sem conta, sem nuvem.

## Instalando

Escolha a plataforma que você usa:

{#snippet desktopInstall()}

```bash
git clone https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader
cd acerola-reader/acerola/desktop
npm install
npm run tauri dev
```

{/snippet}

{#snippet androidInstall()}

Baixe o APK mais recente na página de [releases do GitHub](https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/releases).

{/snippet}

<CodeGroup
	items={[
		{ value: 'desktop', label: 'Desktop', content: desktopInstall },
		{ value: 'android', label: 'Android', content: androidInstall }
	]}
/>

## Sincronizando dois dispositivos

<Steps>

1. Abra a tela **Rede** em ambos os dispositivos.
2. Escaneie o QR Code exibido em um dos dispositivos usando o outro.
3. Confirme o pareamento — a partir daí, os dois dispositivos trocam biblioteca e histórico de leitura direto, sem passar por um servidor.

</Steps>

<Callout type="tip" title="Sem rede local?">

Quando os dispositivos não estão na mesma rede, o Acerola usa o `acerola/relay` para viabilizar a conexão P2P sem abrir portas manualmente.

</Callout>
