import { expect, test } from '@playwright/test';

test.describe('sitemap', () => {
	test('sitemap.xml is a valid, non-empty sitemap', async ({ page }) => {
		const response = await page.request.get('/sitemap.xml');
		expect(response.status()).toBe(200);
		expect(response.headers()['content-type']).toContain('xml');

		const body = await response.text();
		const urlCount = await page.evaluate((xml) => {
			const doc = new DOMParser().parseFromString(xml, 'application/xml');
			if (doc.querySelector('parsererror')) throw new Error('sitemap.xml is not valid XML');
			return doc.querySelectorAll('url').length;
		}, body);

		expect(urlCount).toBeGreaterThan(0);
	});
});
