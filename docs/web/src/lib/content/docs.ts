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
	raw: string;
};

export const FALLBACK_LOCALE = 'pt-br';

// Section order in the sidebar isn't derived from anything structural (file names,
// locales) — it has to be picked on purpose. Listed here in both locales so the
// index lookup below works regardless of which one is active; anything not listed
// falls back after these, in whatever order it was first encountered.
const SECTION_ORDER = [
	'Primeiros passos',
	'Getting Started',
	'Contribuindo',
	'Contributing',
	'Conceitos',
	'Concepts',
	'Privacidade',
	'Privacy',
	'Bibliotecas',
	'Libraries',
	'Docs externas',
	'External docs'
];

const PATH_PATTERN = /^\/src\/content\/docs\/([^/]+)\/(.+)\.md$/;

// Stryker disable next-line all -- `import.meta.glob` is a Vite build-time macro that
// requires a literal string argument; Stryker's mutation wrapper turns it into a function
// call, which breaks Vite's glob-import parser (see stryker.conf.mjs for context).
const modules = import.meta.glob<DocModule>('/src/content/docs/*/**/*.md', { eager: true });

// Segundo glob do mesmo conteúdo, só que como texto cru — usado pelo botão de
// "copiar como Markdown" (ver copy-markdown-button). Precisa ser um glob à parte
// porque `query: '?raw'` muda o que o Vite entrega por import, não dá pra pedir os
// dois formatos de uma vez só.
// Stryker disable next-line all -- mesmo motivo do glob acima.
const rawModules = import.meta.glob<string>('/src/content/docs/*/**/*.md', {
	eager: true,
	query: '?raw',
	import: 'default'
});

const entries: DocEntry[] = Object.entries(modules).flatMap(([path, mod]) => {
	const match = PATH_PATTERN.exec(path);
	if (!match) return [];

	const [, locale, slug] = match;
	return [
		{ locale, slug, component: mod.default, frontmatter: mod.metadata, raw: rawModules[path] }
	];
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

	return [...bySection.entries()]
		.map(([section, docs]) => ({ section, docs }))
		.sort((a, b) => {
			const indexA = SECTION_ORDER.indexOf(a.section);
			const indexB = SECTION_ORDER.indexOf(b.section);
			if (indexA === -1 && indexB === -1) return 0;
			if (indexA === -1) return 1;
			if (indexB === -1) return -1;
			return indexA - indexB;
		});
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
