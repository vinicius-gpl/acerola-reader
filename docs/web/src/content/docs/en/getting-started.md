---
title: Getting Started
description: Install Acerola and sync your first library between two devices.
section: Getting Started
order: 1
---

<script>
	import Callout from '$lib/components/acerola-callout/acerola-callout.svelte';
	import Steps from '$lib/mdsvex/steps.svelte';
	import CodeGroup from '$lib/mdsvex/code-group.svelte';
</script>

<Callout type="note" title="Example page">

This content is a placeholder to validate the documentation pipeline (mdsvex + components + i18n). The real per-platform documentation is still being written.

</Callout>

Acerola is a local comic and manga reader with **100% P2P** sync between devices — no central server, no account, no cloud.

## Installing

Pick the platform you use:

{#snippet desktopInstall()}

```bash
git clone https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader
cd acerola-reader/acerola/desktop
npm install
npm run tauri dev
```

{/snippet}

{#snippet androidInstall()}

Download the latest APK [here](https://binary.acerola-comic.com/android/latest.apk).

{/snippet}

<CodeGroup
	items={[
		{ value: 'desktop', label: 'Desktop', content: desktopInstall },
		{ value: 'android', label: 'Android', content: androidInstall }
	]}
/>

## Syncing two devices

<Steps>

1. Open the **Network** screen on both devices.
2. Scan the QR code shown on one device using the other.
3. Confirm the pairing — from then on, both devices exchange library and reading history directly, with no server in between.

</Steps>

<Callout type="tip" title="No local network?">

When devices aren't on the same network, Acerola uses `acerola/relay` to make the P2P connection work without manually opening ports.

</Callout>
