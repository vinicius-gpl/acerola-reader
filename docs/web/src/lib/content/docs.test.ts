import { describe, expect, it } from 'vitest';
import { FALLBACK_LOCALE, getDoc, getFlatOrder, getPrevNext, getSidebar } from './docs';

// Real content lives at src/content/docs/{en,pt-br}/{architecture,getting-started}.md — only
// these two docs exist per locale, each alone in its own section. Tests below avoid hardcoding
// which of the two ends up first in glob order and instead assert on the relative/boundary
// behavior described by the module's contract.

describe('FALLBACK_LOCALE', () => {
	it('is pt-br', () => {
		expect(FALLBACK_LOCALE).toBe('pt-br');
	});
});

describe('getDoc', () => {
	it('returns the matching doc for a valid locale + slug', () => {
		const doc = getDoc('pt-br', 'getting-started');
		expect(doc).not.toBeNull();
		expect(doc?.locale).toBe('pt-br');
		expect(doc?.slug).toBe('getting-started');
		expect(doc?.frontmatter.title).toBe('Primeiros passos');
	});

	it('returns null for an unknown slug', () => {
		expect(getDoc('pt-br', 'does-not-exist')).toBeNull();
	});

	it('falls back to FALLBACK_LOCALE when the locale has no entries', () => {
		const doc = getDoc('fr', 'getting-started');
		expect(doc).not.toBeNull();
		expect(doc?.locale).toBe(FALLBACK_LOCALE);
		expect(doc?.slug).toBe('getting-started');
	});

	it('returns null when the slug does not exist even after falling back', () => {
		expect(getDoc('fr', 'does-not-exist')).toBeNull();
	});
});

describe('getSidebar', () => {
	it('groups the two known docs into their own sections', () => {
		const sidebar = getSidebar('pt-br');
		const allDocs = sidebar.flatMap((group) => group.docs);

		expect(allDocs).toHaveLength(2);
		expect(sidebar.every((group) => group.docs.length === 1)).toBe(true);
		expect(sidebar.map((group) => group.section).sort()).toEqual(
			['Conceitos', 'Primeiros passos'].sort()
		);
	});

	it('falls back to FALLBACK_LOCALE sections for an unknown locale', () => {
		expect(getSidebar('fr')).toEqual(getSidebar(FALLBACK_LOCALE));
	});
});

describe('getFlatOrder', () => {
	it('flattens the sidebar groups into a single ordered list of both docs', () => {
		const flat = getFlatOrder('pt-br');
		expect(flat).toHaveLength(2);
		expect(flat.map((doc) => doc.slug).sort()).toEqual(['architecture', 'getting-started']);
	});
});

describe('getPrevNext', () => {
	it('gives the first doc a null prev and the second doc as next', () => {
		const [first, second] = getFlatOrder('pt-br');
		const { prev, next } = getPrevNext('pt-br', first.slug);

		expect(prev).toBeNull();
		expect(next?.slug).toBe(second.slug);
	});

	it('gives the last doc the first doc as prev and a null next', () => {
		const [first, second] = getFlatOrder('pt-br');
		const { prev, next } = getPrevNext('pt-br', second.slug);

		expect(prev?.slug).toBe(first.slug);
		expect(next).toBeNull();
	});

	it('returns null/null for an unknown slug', () => {
		expect(getPrevNext('pt-br', 'does-not-exist')).toEqual({ prev: null, next: null });
	});

	it('falls back to FALLBACK_LOCALE when the locale is unknown', () => {
		const [first] = getFlatOrder(FALLBACK_LOCALE);
		expect(getPrevNext('fr', first.slug)).toEqual(getPrevNext(FALLBACK_LOCALE, first.slug));
	});
});
