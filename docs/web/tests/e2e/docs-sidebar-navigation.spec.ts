import { expect, test } from '@playwright/test';

test.describe('docs sidebar navigation', () => {
	test('clicking a sidebar link navigates to the target doc', async ({ page }) => {
		await page.goto('/docs/architecture');
		await expect(page.getByRole('heading', { level: 1, name: 'Arquitetura' })).toBeVisible();

		const sidebar = page.locator('aside').first();
		await sidebar.getByRole('link', { name: 'Primeiros passos' }).click();

		await expect(page).toHaveURL(/\/docs\/getting-started$/);
		await expect(page.getByRole('heading', { level: 1, name: 'Primeiros passos' })).toBeVisible();
	});
});
