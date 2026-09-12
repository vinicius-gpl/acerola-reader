---
title: Primeiros passos
description: Instale o Acerola e sincronize sua primeira biblioteca entre dois dispositivos.
section: Primeiros passos
order: 1
---

<script>
	import Callout from '$lib/components/acerola-callout/acerola-callout.svelte';
	import Steps from '$lib/mdsvex/steps.svelte';
	import CardGrid from '$lib/mdsvex/card-grid.svelte';
	import PlatformCard from '$lib/mdsvex/platform-card.svelte';
	import AcerolaMicrosoftStoreButton from '$lib/components/acerola-microsoft-store-button/acerola-microsoft-store-button.svelte';
	import AcerolaApkDownloadButton from '$lib/components/acerola-apk-download-button/acerola-apk-download-button.svelte';
	import MicrosoftStoreIcon from '$lib/assets/icons/microsoft-store.svg?component';
	import AndroidIcon from '$lib/assets/icons/android.svg?component';
</script>

O Acerola é um leitor de quadrinhos e mangás locais com sincronização **100% P2P** entre dispositivos — sem servidor central, sem conta, sem nuvem. Esta página cobre a instalação em cada plataforma e o primeiro pareamento entre dois dispositivos.

## Instalando

Escolha a plataforma que você usa:

<CardGrid>

<PlatformCard title="Windows" description="Instale o Acerola direto pela Microsoft Store." icon={MicrosoftStoreIcon}>
<AcerolaMicrosoftStoreButton label="Baixe na" />
</PlatformCard>

<PlatformCard title="Android" description="Sincronize sua biblioteca direto no celular." icon={AndroidIcon}>
<AcerolaApkDownloadButton
	label="Baixe o"
	fallbackLabel="Indisponível — tente"
	fallbackTitle="O download direto está indisponível no momento — abrindo a última versão no GitHub Releases."
/>
</PlatformCard>

</CardGrid>

**Requisitos:** Windows 10/11 (64-bit) para o app Desktop; Android 8.0 (API 26) ou mais recente para o app mobile.

<Callout type="note" title="Baixando fora da Microsoft Store">

O pacote da Microsoft Store é o único assinado com certificado confiável. Se você baixar o instalador (`.exe`/`.msi`/`.msix`) direto do [GitHub Releases](https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/releases) em vez de usar a Store, o Windows SmartScreen vai avisar que o executável não é reconhecido — isso é esperado, não é sinal de malware. O APK do Android é sempre assinado, mesmo baixado direto do GitHub Releases; ele só ainda não está publicado na Play Store, então o Android também vai pedir para confirmar a instalação de fontes desconhecidas.

</Callout>

### Compilando a partir do código-fonte

Prefere compilar você mesmo, ou precisa de macOS/Linux? O Acerola Desktop é um app Tauri — clone o repositório e rode direto:

```bash
git clone https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader
cd acerola-reader/acerola/desktop
npm install
npm run tauri dev
```

O Android exige o Android Studio + NDK pra compilar a parte nativa em Rust — o setup completo (toolchain, variáveis de ambiente, primeira build) está documentado na seção [Contribuindo](/pt-br/docs/contributing-android).

## Sincronizando dois dispositivos

<Steps>

1. Abra a tela **Rede** em ambos os dispositivos.
2. Escaneie o QR Code exibido em um dos dispositivos usando o outro.
3. Confirme o pareamento — a partir daí, os dois dispositivos trocam biblioteca e histórico de leitura direto, sem passar por um servidor.

</Steps>

<Callout type="tip" title="Sem rede local?">

Quando os dispositivos não estão na mesma rede, o Acerola usa o `acerola/relay` para viabilizar a conexão P2P sem abrir portas manualmente.

</Callout>
