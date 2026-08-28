import type { Component } from 'svelte';

export type DocFrontmatter = {
	title: string;
	description?: string;
	section: string;
	order?: number;
};

type DocModule = {
	default: Component;
	metadata: DocFrontmatter;
};

export type DocEntry = {
	locale: string;
	slug: string;
	component: Component;
	frontmatter: DocFrontmatter;
};

export const FALLBACK_LOCALE = 'pt-br';

const PATH_PATTERN = /^\/src\/content\/docs\/([^/]+)\/(.+)\.md$/;

// Stryker disable next-line all -- `import.meta.glob` is a Vite build-time macro that
// requires a literal string argument; Stryker's mutation wrapper turns it into a function
// call, which breaks Vite's glob-import parser (see stryker.conf.mjs for context).
const modules = import.meta.glob<DocModule>('/src/content/docs/*/**/*.md', { eager: true });

const entries: DocEntry[] = Object.entries(modules).flatMap(([path, mod]) => {
	const match = PATH_PATTERN.exec(path);
	if (!match) return [];

	const [, locale, slug] = match;
	return [{ locale, slug, component: mod.default, frontmatter: mod.metadata }];
});

function entriesForLocale(locale: string): DocEntry[] {
	const localeEntries = entries.filter((entry) => entry.locale === locale);
	return localeEntries.length
		? localeEntries
		: entries.filter((entry) => entry.locale === FALLBACK_LOCALE);
}

export function getDoc(locale: string, slug: string): DocEntry | null {
	return (
		entries.find((entry) => entry.locale === locale && entry.slug === slug) ??
		entries.find((entry) => entry.locale === FALLBACK_LOCALE && entry.slug === slug) ??
		null
	);
}

export type SidebarGroup = {
	section: string;
	docs: DocEntry[];
};

export function getSidebar(locale: string): SidebarGroup[] {
	const bySection = new Map<string, DocEntry[]>();

	for (const entry of entriesForLocale(locale)) {
		const section = entry.frontmatter.section ?? 'Docs';
		const group = bySection.get(section) ?? [];
		group.push(entry);
		bySection.set(section, group);
	}

	for (const group of bySection.values()) {
		group.sort((a, b) => (a.frontmatter.order ?? 0) - (b.frontmatter.order ?? 0));
	}

	return [...bySection.entries()].map(([section, docs]) => ({ section, docs }));
}

export function getFlatOrder(locale: string): DocEntry[] {
	return getSidebar(locale).flatMap((group) => group.docs);
}

export function getPrevNext(
	locale: string,
	slug: string
): { prev: DocEntry | null; next: DocEntry | null } {
	const flat = getFlatOrder(locale);
	const index = flat.findIndex((entry) => entry.slug === slug);
	if (index === -1) return { prev: null, next: null };

	return {
		prev: flat[index - 1] ?? null,
		next: flat[index + 1] ?? null
	};
}
