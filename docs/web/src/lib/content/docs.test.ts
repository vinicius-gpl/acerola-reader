import { describe, expect, it } from 'vitest';
import { FALLBACK_LOCALE, getDoc, getFlatOrder, getPrevNext, getSidebar } from './docs';

// Real content lives at src/content/docs/{en,pt-br}/*.md and grows over time — tests below
// avoid hardcoding the doc count or which slug ends up first in glob order, and instead assert
// on the relative/boundary behavior described by the module's contract.

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
	it('groups every doc under its frontmatter section, with no doc lost or duplicated', () => {
		const sidebar = getSidebar('pt-br');
		const allDocs = sidebar.flatMap((group) => group.docs);

		expect(allDocs.length).toBeGreaterThan(0);
		expect(new Set(allDocs.map((doc) => doc.slug)).size).toBe(allDocs.length);
		expect(sidebar.every((group) => group.docs.length > 0)).toBe(true);
	});

	it('puts Getting Started first and Contributing right after it', () => {
		const sections = getSidebar('pt-br').map((group) => group.section);

		expect(sections[0]).toBe('Primeiros passos');
		expect(sections[1]).toBe('Contribuindo');
	});

	it('sorts sections not in the canonical order after the ones that are', () => {
		const sections = getSidebar('pt-br').map((group) => group.section);
		const canonical = [
			'Primeiros passos',
			'Contribuindo',
			'Conceitos',
			'Privacidade',
			'Bibliotecas'
		];
		const canonicalPositions = canonical
			.map((section) => sections.indexOf(section))
			.filter((index) => index !== -1);

		expect(canonicalPositions).toEqual([...canonicalPositions].sort((a, b) => a - b));
	});

	it('falls back to FALLBACK_LOCALE sections for an unknown locale', () => {
		expect(getSidebar('fr')).toEqual(getSidebar(FALLBACK_LOCALE));
	});
});

describe('getFlatOrder', () => {
	it('flattens every sidebar group into a single list with no doc lost or duplicated', () => {
		const sidebar = getSidebar('pt-br');
		const flat = getFlatOrder('pt-br');

		expect(flat).toHaveLength(sidebar.flatMap((group) => group.docs).length);
		expect(flat.map((doc) => doc.slug).sort()).toEqual(
			sidebar
				.flatMap((group) => group.docs)
				.map((doc) => doc.slug)
				.sort()
		);
	});

	it('starts with the first doc of the first section', () => {
		const [firstSection] = getSidebar('pt-br');
		const [firstDoc] = getFlatOrder('pt-br');

		expect(firstDoc.slug).toBe(firstSection.docs[0].slug);
	});
});

describe('getPrevNext', () => {
	it('gives the first doc a null prev and the second doc as next', () => {
		const [first, second] = getFlatOrder('pt-br');
		const { prev, next } = getPrevNext('pt-br', first.slug);

		expect(prev).toBeNull();
		expect(next?.slug).toBe(second.slug);
	});

	it('gives the last doc the second-to-last doc as prev and a null next', () => {
		const flat = getFlatOrder('pt-br');
		const last = flat[flat.length - 1];
		const secondToLast = flat[flat.length - 2];
		const { prev, next } = getPrevNext('pt-br', last.slug);

		expect(prev?.slug).toBe(secondToLast.slug);
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
