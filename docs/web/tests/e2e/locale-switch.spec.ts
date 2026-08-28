import { expect, test } from '@playwright/test';

// Locale is controlled by paraglide's `url` strategy: the base locale ("pt-br")
// serves from an unprefixed path, and "en" is served from an "/en" prefix. The
// toggle in nav-controls.svelte cycles through `locales` and shows the current
// locale code (uppercased) as its label.
test.describe('locale switch', () => {
	test('toggling locale changes the URL prefix and re-renders content', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByRole('button', { name: 'PT-BR' })).toBeVisible();

		await page.getByRole('button', { name: 'PT-BR' }).click();

		await expect(page).toHaveURL(/\/en\/?$/);
		await expect(page.getByRole('button', { name: 'EN' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 1 })).toContainText(
			'Read your comics and manga on any device. No server, no account.'
		);
	});
});
