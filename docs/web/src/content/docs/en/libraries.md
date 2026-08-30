---
title: Libraries used
description: Credits and licenses for every third-party library used across the Acerola ecosystem.
section: Libraries
order: 1
---

<script>
	import Callout from '$lib/mdsvex/callout.svelte';
</script>

Acerola only exists because it stands on a huge ecosystem of free and open-source software. This page lists the main third-party libraries used in each project of the monorepo, with a link and license — a thank-you to the people maintaining that work.

<Callout type="note" title="Scope">

This list covers each project's direct dependencies (not the full transitive tree), and test/lint tooling isn't included here — it's already covered in the [Contributing](/en/docs/contributing-overview) section.

</Callout>

## acerola/desktop — Rust (`src-tauri`)

| Library | License |
| --- | --- |
| [acerola-p2p](https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/tree/main/lib/p2p) | MPL-2.0 |
| [anyhow](https://github.com/dtolnay/anyhow) | MIT OR Apache-2.0 |
| [async-trait](https://github.com/dtolnay/async-trait) | MIT OR Apache-2.0 |
| [axum](https://github.com/tokio-rs/axum) | MIT |
| [chrono](https://github.com/chronotope/chrono) | MIT OR Apache-2.0 |
| [futures](https://github.com/rust-lang/futures-rs) | MIT OR Apache-2.0 |
| [aes-gcm](https://github.com/RustCrypto/AEADs) | Apache-2.0 OR MIT |
| [image](https://github.com/image-rs/image) | MIT OR Apache-2.0 |
| [keyring](https://github.com/hwchen/keyring-rs) | MIT OR Apache-2.0 |
| [log](https://github.com/rust-lang/log) | MIT OR Apache-2.0 |
| [lru](https://github.com/jeromefroe/lru-rs) | MIT |
| [mdns-sd](https://github.com/keepsimple1/mdns-sd) | Apache-2.0 OR MIT |
| [once_cell](https://github.com/matklad/once_cell) | MIT OR Apache-2.0 |
| [pdfium-render](https://github.com/ajrcarey/pdfium-render) | MIT OR Apache-2.0 |
| [quick-xml](https://github.com/tafia/quick-xml) | MIT |
| [rand](https://github.com/rust-random/rand) | MIT OR Apache-2.0 |
| [regex](https://github.com/rust-lang/regex) | MIT OR Apache-2.0 |
| [reqwest](https://github.com/seanmonstar/reqwest) | MIT OR Apache-2.0 |
| [serde](https://github.com/serde-rs/serde) | MIT OR Apache-2.0 |
| [serde_json](https://github.com/serde-rs/json) | MIT OR Apache-2.0 |
| [sha2](https://github.com/RustCrypto/hashes) | MIT OR Apache-2.0 |
| [sqlx](https://github.com/launchbadge/sqlx) | MIT OR Apache-2.0 |
| [tauri](https://github.com/tauri-apps/tauri) | Apache-2.0 OR MIT |
| [tauri-plugin-dialog / fs / log / opener / sql / store](https://github.com/tauri-apps/plugins-workspace) | Apache-2.0 OR MIT |
| [thiserror](https://github.com/dtolnay/thiserror) | MIT OR Apache-2.0 |
| [tokio](https://github.com/tokio-rs/tokio) | MIT |
| [tokio-stream / tokio-util](https://github.com/tokio-rs/tokio) | MIT |
| [tracing](https://github.com/tokio-rs/tracing) | MIT |
| [unrar](https://github.com/muja/unrar.rs) | MIT OR Apache-2.0¹ |
| [zip](https://github.com/zip-rs/zip2) | MIT |

¹ The Rust binding itself is MIT/Apache-2.0, but the embedded UnRAR C library follows RARLAB's own license (free to use, with a restriction on building a RAR-compatible archiver).

## acerola/desktop — Frontend (Svelte)

| Library | License |
| --- | --- |
| [@tauri-apps/api](https://github.com/tauri-apps/tauri) | Apache-2.0 OR MIT |
| [@tauri-apps/plugin-dialog / fs / log / opener / sql / store](https://github.com/tauri-apps/plugins-workspace) | MIT OR Apache-2.0 |
| [@tauri-apps/cli](https://github.com/tauri-apps/tauri) | Apache-2.0 OR MIT |
| [qrcode](https://github.com/soldair/node-qrcode) | MIT |
| [svelte](https://github.com/sveltejs/svelte) | MIT |
| [@sveltejs/kit](https://github.com/sveltejs/kit) | MIT |
| [@sveltejs/adapter-static](https://github.com/sveltejs/kit) | MIT |
| [vite](https://github.com/vitejs/vite) | MIT |
| [typescript](https://github.com/microsoft/TypeScript) | Apache-2.0 |
| [tailwindcss](https://github.com/tailwindlabs/tailwindcss) | MIT |
| [bits-ui](https://github.com/huntabyte/bits-ui) | MIT |
| [shadcn-svelte](https://github.com/huntabyte/shadcn-svelte) | MIT |
| [mode-watcher](https://github.com/svecosystem/mode-watcher) | MIT |
| [svelte-sonner](https://github.com/wobsoriano/svelte-sonner) | MIT |
| [tailwind-merge](https://github.com/dcastil/tailwind-merge) | MIT |
| [tailwind-variants](https://github.com/heroui-inc/tailwind-variants) | MIT |
| [tw-animate-css](https://github.com/Wombosvideo/tw-animate-css) | MIT |
| [@lucide/svelte](https://github.com/lucide-icons/lucide) | ISC |
| [@internationalized/date](https://github.com/adobe/react-spectrum/tree/main/packages/@internationalized/date) | Apache-2.0 |
| [@inlang/paraglide-js](https://github.com/opral/paraglide-js) | MIT |
| [@poppanator/sveltekit-svg](https://github.com/poppa/sveltekit-svg) | MIT |

## acerola/android

| Library | License |
| --- | --- |
| [Kotlin](https://github.com/JetBrains/kotlin) | Apache-2.0 |
| [Jetpack Compose](https://android.googlesource.com/platform/frameworks/support) | Apache-2.0 |
| [AndroidX](https://android.googlesource.com/platform/frameworks/support) (core-ktx, lifecycle, navigation, room, datastore, documentfile, security-crypto, work, appcompat, annotation) | Apache-2.0 |
| [Material Components for Android](https://github.com/material-components/material-components-android) | Apache-2.0 |
| [Hilt](https://github.com/google/dagger) (Dagger) | Apache-2.0 |
| [Coil](https://github.com/coil-kt/coil) | Apache-2.0 |
| [Retrofit](https://github.com/square/retrofit) | Apache-2.0 |
| [OkHttp](https://github.com/square/okhttp) | Apache-2.0 |
| [Moshi](https://github.com/square/moshi) | Apache-2.0 |
| [Apollo Kotlin](https://github.com/apollographql/apollo-kotlin) | MIT |
| [Arrow](https://github.com/arrow-kt/arrow) | Apache-2.0 |
| [kotlinx.coroutines](https://github.com/Kotlin/kotlinx.coroutines) | Apache-2.0 |
| [kotlinx.collections.immutable](https://github.com/Kotlin/kotlinx.collections.immutable) | Apache-2.0 |
| [Haze](https://github.com/chrisbanes/haze) | Apache-2.0 |
| [JNA](https://github.com/java-native-access/jna) | Apache-2.0 OR LGPL-2.1 |
| [ZXing](https://github.com/zxing/zxing) | Apache-2.0 |
| [Junrar](https://github.com/junrar/junrar) | UnRAR License¹ |
| [Google Play Services — Code Scanner](https://developers.google.com/android/reference/com/google/android/gms/common/moduleinstall/ModuleInstall) | Google Play Services terms (not OSS) |

¹ RARLAB's own license (NRL) — free to use, with a restriction on building a RAR-compatible archiver.

## lib/p2p

| Library | License |
| --- | --- |
| [iroh](https://github.com/n0-computer/iroh) | MIT OR Apache-2.0 |
| [iroh-blobs](https://github.com/n0-computer/iroh-blobs) | MIT OR Apache-2.0 |
| [iroh-mdns-address-lookup](https://github.com/n0-computer/iroh-address-lookups) | MIT OR Apache-2.0 |
| [irpc](https://github.com/n0-computer/irpc) | Apache-2.0/MIT |
| [n0-error](https://github.com/n0-computer/n0-error) | MIT OR Apache-2.0 |
| [tokio](https://github.com/tokio-rs/tokio) | MIT |
| [tracing](https://github.com/tokio-rs/tracing) / [tracing-subscriber](https://github.com/tokio-rs/tracing) | MIT |
| [async-trait](https://github.com/dtolnay/async-trait) | MIT OR Apache-2.0 |
| [blake3](https://github.com/BLAKE3-team/BLAKE3) | CC0-1.0 OR Apache-2.0 |
| [futures](https://github.com/rust-lang/futures-rs) | MIT OR Apache-2.0 |
| [getrandom](https://github.com/rust-random/getrandom) | MIT OR Apache-2.0 |
| [rand](https://github.com/rust-random/rand) | MIT OR Apache-2.0 |
| [secrecy](https://github.com/iqlusioninc/crates/tree/main/secrecy) | Apache-2.0 OR MIT |
| [serde](https://github.com/serde-rs/serde) / [serde_json](https://github.com/serde-rs/json) | MIT OR Apache-2.0 |
| [thiserror](https://github.com/dtolnay/thiserror) | MIT OR Apache-2.0 |
| [uuid](https://github.com/uuid-rs/uuid) | Apache-2.0 OR MIT |

## lib/relay

`lib/relay` is [iroh](https://github.com/n0-computer/iroh)'s relay code, vendored (MIT OR Apache-2.0) — it doesn't have dependencies of its own beyond it yet.

<Callout type="tip" title="Something missing or wrong?">

If any library or license is out of date, open an issue or PR — this page should always reflect the real state of each `Cargo.toml`/`package.json`/`libs.versions.toml`'s direct dependencies.

</Callout>
