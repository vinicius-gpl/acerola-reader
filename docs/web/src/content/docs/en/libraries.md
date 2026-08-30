---
title: Libraries used
description: Credits and licenses for every third-party library used across the Acerola ecosystem.
section: Libraries
order: 1
---

<script>
	import Callout from '$lib/mdsvex/callout.svelte';
	import LibraryTable from '$lib/mdsvex/library-table.svelte';

	const desktopRust = [
		{ name: 'acerola-p2p', url: 'https://github.com/Vinicius-Gabriel-P-Leitao/acerola-reader/tree/main/lib/p2p', license: 'MPL-2.0' },
		{ name: 'anyhow', url: 'https://github.com/dtolnay/anyhow', license: 'MIT OR Apache-2.0' },
		{ name: 'async-trait', url: 'https://github.com/dtolnay/async-trait', license: 'MIT OR Apache-2.0' },
		{ name: 'axum', url: 'https://github.com/tokio-rs/axum', license: 'MIT' },
		{ name: 'chrono', url: 'https://github.com/chronotope/chrono', license: 'MIT OR Apache-2.0' },
		{ name: 'futures', url: 'https://github.com/rust-lang/futures-rs', license: 'MIT OR Apache-2.0' },
		{ name: 'aes-gcm', url: 'https://github.com/RustCrypto/AEADs', license: 'Apache-2.0 OR MIT' },
		{ name: 'image', url: 'https://github.com/image-rs/image', license: 'MIT OR Apache-2.0' },
		{ name: 'keyring', url: 'https://github.com/hwchen/keyring-rs', license: 'MIT OR Apache-2.0' },
		{ name: 'log', url: 'https://github.com/rust-lang/log', license: 'MIT OR Apache-2.0' },
		{ name: 'lru', url: 'https://github.com/jeromefroe/lru-rs', license: 'MIT' },
		{ name: 'mdns-sd', url: 'https://github.com/keepsimple1/mdns-sd', license: 'Apache-2.0 OR MIT' },
		{ name: 'once_cell', url: 'https://github.com/matklad/once_cell', license: 'MIT OR Apache-2.0' },
		{ name: 'pdfium-render', url: 'https://github.com/ajrcarey/pdfium-render', license: 'MIT OR Apache-2.0' },
		{ name: 'quick-xml', url: 'https://github.com/tafia/quick-xml', license: 'MIT' },
		{ name: 'rand', url: 'https://github.com/rust-random/rand', license: 'MIT OR Apache-2.0' },
		{ name: 'regex', url: 'https://github.com/rust-lang/regex', license: 'MIT OR Apache-2.0' },
		{ name: 'reqwest', url: 'https://github.com/seanmonstar/reqwest', license: 'MIT OR Apache-2.0' },
		{ name: 'serde', url: 'https://github.com/serde-rs/serde', license: 'MIT OR Apache-2.0' },
		{ name: 'serde_json', url: 'https://github.com/serde-rs/json', license: 'MIT OR Apache-2.0' },
		{ name: 'sha2', url: 'https://github.com/RustCrypto/hashes', license: 'MIT OR Apache-2.0' },
		{ name: 'sqlx', url: 'https://github.com/launchbadge/sqlx', license: 'MIT OR Apache-2.0' },
		{ name: 'tauri', url: 'https://github.com/tauri-apps/tauri', license: 'Apache-2.0 OR MIT' },
		{ name: 'tauri-plugin-dialog / fs / log / opener / sql / store', url: 'https://github.com/tauri-apps/plugins-workspace', license: 'Apache-2.0 OR MIT' },
		{ name: 'thiserror', url: 'https://github.com/dtolnay/thiserror', license: 'MIT OR Apache-2.0' },
		{ name: 'tokio', url: 'https://github.com/tokio-rs/tokio', license: 'MIT' },
		{ name: 'tokio-stream / tokio-util', url: 'https://github.com/tokio-rs/tokio', license: 'MIT' },
		{ name: 'tracing', url: 'https://github.com/tokio-rs/tracing', license: 'MIT' },
		{ name: 'unrar', url: 'https://github.com/muja/unrar.rs', license: 'MIT OR Apache-2.0', note: '¹' },
		{ name: 'zip', url: 'https://github.com/zip-rs/zip2', license: 'MIT' }
	];

	const desktopJs = [
		{ name: '@tauri-apps/api', url: 'https://github.com/tauri-apps/tauri', license: 'Apache-2.0 OR MIT' },
		{ name: '@tauri-apps/plugin-dialog / fs / log / opener / sql / store', url: 'https://github.com/tauri-apps/plugins-workspace', license: 'MIT OR Apache-2.0' },
		{ name: '@tauri-apps/cli', url: 'https://github.com/tauri-apps/tauri', license: 'Apache-2.0 OR MIT' },
		{ name: 'qrcode', url: 'https://github.com/soldair/node-qrcode', license: 'MIT' },
		{ name: 'svelte', url: 'https://github.com/sveltejs/svelte', license: 'MIT' },
		{ name: '@sveltejs/kit', url: 'https://github.com/sveltejs/kit', license: 'MIT' },
		{ name: '@sveltejs/adapter-static', url: 'https://github.com/sveltejs/kit', license: 'MIT' },
		{ name: 'vite', url: 'https://github.com/vitejs/vite', license: 'MIT' },
		{ name: 'typescript', url: 'https://github.com/microsoft/TypeScript', license: 'Apache-2.0' },
		{ name: 'tailwindcss', url: 'https://github.com/tailwindlabs/tailwindcss', license: 'MIT' },
		{ name: 'bits-ui', url: 'https://github.com/huntabyte/bits-ui', license: 'MIT' },
		{ name: 'shadcn-svelte', url: 'https://github.com/huntabyte/shadcn-svelte', license: 'MIT' },
		{ name: 'mode-watcher', url: 'https://github.com/svecosystem/mode-watcher', license: 'MIT' },
		{ name: 'svelte-sonner', url: 'https://github.com/wobsoriano/svelte-sonner', license: 'MIT' },
		{ name: 'tailwind-merge', url: 'https://github.com/dcastil/tailwind-merge', license: 'MIT' },
		{ name: 'tailwind-variants', url: 'https://github.com/heroui-inc/tailwind-variants', license: 'MIT' },
		{ name: 'tw-animate-css', url: 'https://github.com/Wombosvideo/tw-animate-css', license: 'MIT' },
		{ name: '@lucide/svelte', url: 'https://github.com/lucide-icons/lucide', license: 'ISC' },
		{ name: '@internationalized/date', url: 'https://github.com/adobe/react-spectrum/tree/main/packages/@internationalized/date', license: 'Apache-2.0' },
		{ name: '@inlang/paraglide-js', url: 'https://github.com/opral/paraglide-js', license: 'MIT' },
		{ name: '@poppanator/sveltekit-svg', url: 'https://github.com/poppa/sveltekit-svg', license: 'MIT' }
	];

	const android = [
		{ name: 'Kotlin', url: 'https://github.com/JetBrains/kotlin', license: 'Apache-2.0' },
		{ name: 'Jetpack Compose', url: 'https://android.googlesource.com/platform/frameworks/support', license: 'Apache-2.0' },
		{ name: 'AndroidX (core-ktx, lifecycle, navigation, room, datastore, documentfile, security-crypto, work, appcompat, annotation)', url: 'https://android.googlesource.com/platform/frameworks/support', license: 'Apache-2.0' },
		{ name: 'Material Components for Android', url: 'https://github.com/material-components/material-components-android', license: 'Apache-2.0' },
		{ name: 'Hilt (Dagger)', url: 'https://github.com/google/dagger', license: 'Apache-2.0' },
		{ name: 'Coil', url: 'https://github.com/coil-kt/coil', license: 'Apache-2.0' },
		{ name: 'Retrofit', url: 'https://github.com/square/retrofit', license: 'Apache-2.0' },
		{ name: 'OkHttp', url: 'https://github.com/square/okhttp', license: 'Apache-2.0' },
		{ name: 'Moshi', url: 'https://github.com/square/moshi', license: 'Apache-2.0' },
		{ name: 'Apollo Kotlin', url: 'https://github.com/apollographql/apollo-kotlin', license: 'MIT' },
		{ name: 'Arrow', url: 'https://github.com/arrow-kt/arrow', license: 'Apache-2.0' },
		{ name: 'kotlinx.coroutines', url: 'https://github.com/Kotlin/kotlinx.coroutines', license: 'Apache-2.0' },
		{ name: 'kotlinx.collections.immutable', url: 'https://github.com/Kotlin/kotlinx.collections.immutable', license: 'Apache-2.0' },
		{ name: 'Haze', url: 'https://github.com/chrisbanes/haze', license: 'Apache-2.0' },
		{ name: 'JNA', url: 'https://github.com/java-native-access/jna', license: 'Apache-2.0 OR LGPL-2.1' },
		{ name: 'ZXing', url: 'https://github.com/zxing/zxing', license: 'Apache-2.0' },
		{ name: 'Junrar', url: 'https://github.com/junrar/junrar', license: 'UnRAR License', note: '¹' },
		{ name: 'Google Play Services — Code Scanner', url: 'https://developers.google.com/android/reference/com/google/android/gms/common/moduleinstall/ModuleInstall', license: 'Google Play Services terms (not OSS)' }
	];

	const p2p = [
		{ name: 'iroh', url: 'https://github.com/n0-computer/iroh', license: 'MIT OR Apache-2.0' },
		{ name: 'iroh-blobs', url: 'https://github.com/n0-computer/iroh-blobs', license: 'MIT OR Apache-2.0' },
		{ name: 'iroh-mdns-address-lookup', url: 'https://github.com/n0-computer/iroh-address-lookups', license: 'MIT OR Apache-2.0' },
		{ name: 'irpc', url: 'https://github.com/n0-computer/irpc', license: 'Apache-2.0/MIT' },
		{ name: 'n0-error', url: 'https://github.com/n0-computer/n0-error', license: 'MIT OR Apache-2.0' },
		{ name: 'tokio', url: 'https://github.com/tokio-rs/tokio', license: 'MIT' },
		{ name: 'tracing / tracing-subscriber', url: 'https://github.com/tokio-rs/tracing', license: 'MIT' },
		{ name: 'async-trait', url: 'https://github.com/dtolnay/async-trait', license: 'MIT OR Apache-2.0' },
		{ name: 'blake3', url: 'https://github.com/BLAKE3-team/BLAKE3', license: 'CC0-1.0 OR Apache-2.0' },
		{ name: 'futures', url: 'https://github.com/rust-lang/futures-rs', license: 'MIT OR Apache-2.0' },
		{ name: 'getrandom', url: 'https://github.com/rust-random/getrandom', license: 'MIT OR Apache-2.0' },
		{ name: 'rand', url: 'https://github.com/rust-random/rand', license: 'MIT OR Apache-2.0' },
		{ name: 'secrecy', url: 'https://github.com/iqlusioninc/crates/tree/main/secrecy', license: 'Apache-2.0 OR MIT' },
		{ name: 'serde / serde_json', url: 'https://github.com/serde-rs/serde', license: 'MIT OR Apache-2.0' },
		{ name: 'thiserror', url: 'https://github.com/dtolnay/thiserror', license: 'MIT OR Apache-2.0' },
		{ name: 'uuid', url: 'https://github.com/uuid-rs/uuid', license: 'Apache-2.0 OR MIT' }
	];
</script>

Acerola only exists because it stands on a huge ecosystem of free and open-source software. This page lists the main third-party libraries used in each project of the monorepo, with a link and license — a thank-you to the people maintaining that work.

<Callout type="note" title="Scope">

This list covers each project's direct dependencies (not the full transitive tree), and test/lint tooling isn't included here — it's already covered in the [Contributing](/en/docs/contributing-overview) section.

</Callout>

## acerola/desktop — Rust (`src-tauri`)

<LibraryTable libraries={desktopRust} />

¹ The Rust binding itself is MIT/Apache-2.0, but the embedded UnRAR C library follows RARLAB's own license (free to use, with a restriction on building a RAR-compatible archiver).

## acerola/desktop — Frontend (Svelte)

<LibraryTable libraries={desktopJs} />

## acerola/android

<LibraryTable libraries={android} />

¹ RARLAB's own license (NRL) — free to use, with a restriction on building a RAR-compatible archiver.

## lib/p2p

<LibraryTable libraries={p2p} />

## lib/relay

`lib/relay` is [iroh](https://github.com/n0-computer/iroh)'s relay code, vendored (MIT OR Apache-2.0) — it doesn't have dependencies of its own beyond it yet.

<Callout type="tip" title="Something missing or wrong?">

If any library or license is out of date, open an issue or PR — this page should always reflect the real state of each `Cargo.toml`/`package.json`/`libs.versions.toml`'s direct dependencies.

</Callout>
