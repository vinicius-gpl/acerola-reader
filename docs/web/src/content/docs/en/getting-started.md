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

Acerola is a local comic and manga reader with **100% P2P** sync between devices — no central server, no account, no cloud. This page covers installing on each platform and pairing two devices for the first time.

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

**Requirements:** Windows 10/11 (64-bit) for the Desktop app; Android 8.0 (API 26) or newer for the mobile app.

<Callout type="note" title="Downloading outside the Microsoft Store">

The Microsoft Store package is the only one signed with a trusted certificate. If you download the installer (`.exe`/`.msi`/`.msix`) directly from [GitHub Releases](https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/releases) instead of using the Store, Windows SmartScreen will warn that the executable isn't recognized — that's expected, not a sign of malware. The Android APK is always signed, even downloaded straight from GitHub Releases; it just isn't published on the Play Store yet, so Android will also ask you to confirm installing from an unknown source.

</Callout>

### Building from source

Prefer to build it yourself, or need macOS/Linux? Acerola Desktop is a Tauri app — clone the repo and run it directly:

```bash
git clone https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader
cd acerola-reader/acerola/desktop
npm install
npm run tauri dev
```

Android needs Android Studio + the NDK to build the native Rust side — the full setup (toolchain, environment variables, first build) is documented in the [Contributing](/en/docs/contributing-android) section.

## Syncing two devices

<Steps>

1. Open the **Network** screen on both devices.
2. Scan the QR code shown on one device using the other.
3. Confirm the pairing — from then on, both devices exchange library and reading history directly, with no server in between.

</Steps>

<Callout type="tip" title="No local network?">

When devices aren't on the same network, Acerola uses `acerola/relay` to make the P2P connection work without manually opening ports.

</Callout>
