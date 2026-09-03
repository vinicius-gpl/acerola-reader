---
title: Getting Started
description: Install Acerola and sync your first library between two devices.
section: Getting Started
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

<Callout type="note" title="Example page">

This content is a placeholder to validate the documentation pipeline (mdsvex + components + i18n). The real per-platform documentation is still being written.

</Callout>

Acerola is a local comic and manga reader with **100% P2P** sync between devices — no central server, no account, no cloud.

## Installing

Pick the platform you use:

<CardGrid>

<PlatformCard title="Windows" description="Install Acerola straight from the Microsoft Store." icon={MicrosoftStoreIcon}>
<AcerolaMicrosoftStoreButton label="Get it from" />
</PlatformCard>

<PlatformCard title="Android" description="Sync your library straight to your phone." icon={AndroidIcon}>
<AcerolaApkDownloadButton
	label="Get the"
	fallbackLabel="Unavailable — try"
	fallbackTitle="Direct download is temporarily unavailable — opening the latest GitHub release instead."
/>
</PlatformCard>

</CardGrid>

### Building from source

Prefer to build it yourself, or need macOS/Linux? Acerola is a Tauri app — clone the repo and run it directly:

```bash
git clone https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader
cd acerola-reader/acerola/desktop
npm install
npm run tauri dev
```

## Syncing two devices

<Steps>

1. Open the **Network** screen on both devices.
2. Scan the QR code shown on one device using the other.
3. Confirm the pairing — from then on, both devices exchange library and reading history directly, with no server in between.

</Steps>

<Callout type="tip" title="No local network?">

When devices aren't on the same network, Acerola uses `acerola/relay` to make the P2P connection work without manually opening ports.

</Callout>
