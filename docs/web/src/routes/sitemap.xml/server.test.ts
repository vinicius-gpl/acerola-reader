import { describe, expect, it } from 'vitest';
import { SITE_URL } from '$lib/constants/site';
import { getFlatOrder, FALLBACK_LOCALE } from '$lib/content/docs';
import { GET } from './+server';

// This endpoint doesn't touch event.platform/Cloudflare bindings, so a minimal fake
// RequestEvent is enough to exercise it directly, without spinning up a real request. Typed
// off GET's own parameter (rather than the generic `RequestEvent` from '@sveltejs/kit', whose
// default RouteId is the union of every route in the app) so it matches the `'/sitemap.xml'`
// literal this specific handler expects.
const fakeEvent = {} as Parameters<typeof GET>[0];

describe('GET /sitemap.xml', () => {
	it('responds with an application/xml content type', async () => {
		const response = await GET(fakeEvent);
		expect(response.headers.get('Content-Type')).toBe('application/xml');
	});

	it('returns well-formed XML with one <url> per page (home + every doc)', async () => {
		const response = await GET(fakeEvent);
		const body = await response.text();

		const doc = new DOMParser().parseFromString(body, 'application/xml');
		expect(doc.querySelector('parsererror')).toBeNull();

		const urls = doc.querySelectorAll('url');
		const docCount = getFlatOrder(FALLBACK_LOCALE).length;
		expect(urls.length).toBe(1 + docCount);
	});

	it('gives the home page top priority and every other page a lower one', async () => {
		const response = await GET(fakeEvent);
		const body = await response.text();
		const doc = new DOMParser().parseFromString(body, 'application/xml');

		const urls = [...doc.querySelectorAll('url')];
		const home = urls.find((url) => url.querySelector('loc')?.textContent === SITE_URL + '/');
		expect(home?.querySelector('priority')?.textContent).toBe('1.0');

		const others = urls.filter((url) => url !== home);
		expect(others.length).toBeGreaterThan(0);
		for (const url of others) {
			expect(url.querySelector('priority')?.textContent).toBe('0.8');
			expect(url.querySelector('loc')?.textContent?.startsWith(SITE_URL)).toBe(true);
		}
	});
});
