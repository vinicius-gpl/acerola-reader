import type { PageLoadEvent } from './$types';
import { describe, expect, it } from 'vitest';
import type { DocEntry, DocFrontmatter } from '$lib/content/docs';
import { load } from './+page';

// getLocale() resolves to the base locale ('pt-br') in this jsdom test environment, which is
// also FALLBACK_LOCALE — matching the two real docs at src/content/docs/pt-br/*.md.
function makeEvent(slug: string): PageLoadEvent {
	return { params: { slug } } as unknown as PageLoadEvent;
}

// `PageLoad`'s generic OutputData defaults to a shape that includes `| void` (SvelteKit can't
// know a given load never actually returns void), so the real return value is cast here to
// what src/routes/docs/[...slug]/+page.ts actually produces on the happy path.
type LoadResult = {
	Doc: unknown;
	frontmatter: DocFrontmatter;
	raw: string;
	prev: DocEntry | null;
	next: DocEntry | null;
};

describe('docs/[...slug] load', () => {
	it('returns the doc component, frontmatter, raw markdown, and prev/next for a valid slug', () => {
		const result = load(makeEvent('getting-started')) as LoadResult;

		expect(result.frontmatter.title).toBe('Primeiros passos');
		expect(result.Doc).toBeDefined();
		expect(result.raw).toContain('Primeiros passos');
		expect(result).toHaveProperty('prev');
		expect(result).toHaveProperty('next');
	});

	it('throws a 404 error for an unknown slug', () => {
		expect(() => load(makeEvent('does-not-exist'))).toThrowError(
			expect.objectContaining({ status: 404 })
		);
	});
});
