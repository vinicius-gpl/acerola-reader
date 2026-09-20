import { convertFileSrc } from '@tauri-apps/api/core';
import { getArtworkVersion } from '$lib/state/artwork-version.svelte';

/**
 * Resolves a file path to a URL suitable for display in the UI.
 * Handles Windows backslashes and converts them to forward slashes.
 * Appends the global artwork cache-bust version (see `artwork-version.svelte.ts`) so any
 * screen showing this URL updates automatically after a cover/banner gets re-downloaded —
 * the file path itself never changes (always `cover.jpg`/`banner.jpg`), so without this the
 * WebView keeps serving the cached image after the file is overwritten on disk.
 */
export function resolveArtworkPath(path: string | null | undefined): string | null {
	if (!path) return null;
	const url = convertFileSrc(path.replaceAll('\\', '/'));
	const version = getArtworkVersion();
	if (!version) return url;
	return url + (url.includes('?') ? '&' : '?') + 'v=' + version;
}

export interface ArtworkData {
	cover?: string | null;
	banner?: string | null;
}

/**
 * Resolves the cover image for a comic.
 * Fallback priority: cover -> banner -> null
 */
export function resolveCover(artwork: ArtworkData): string | null {
	return resolveArtworkPath(artwork.cover ?? artwork.banner);
}

/**
 * Resolves the banner image for a comic.
 * Fallback priority: banner -> cover -> null
 */
export function resolveBanner(artwork: ArtworkData): string | null {
	return resolveArtworkPath(artwork.banner ?? artwork.cover);
}
