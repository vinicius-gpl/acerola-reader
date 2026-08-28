import { describe, expect, it } from 'vitest';
import { GITHUB_EDIT_BASE, GITHUB_URL, OG_IMAGE_URL, SITE_URL } from './site';

describe('site constants', () => {
	it('exposes well-formed absolute URLs', () => {
		for (const url of [GITHUB_URL, GITHUB_EDIT_BASE, SITE_URL, OG_IMAGE_URL]) {
			expect(() => new URL(url)).not.toThrow();
		}
	});

	it('derives GITHUB_EDIT_BASE and OG_IMAGE_URL from their base URLs', () => {
		expect(GITHUB_EDIT_BASE.startsWith(GITHUB_URL)).toBe(true);
		expect(GITHUB_EDIT_BASE).toBe(`${GITHUB_URL}/edit/main/docs/web/src/content/docs`);
		expect(OG_IMAGE_URL.startsWith(SITE_URL)).toBe(true);
	});
});
