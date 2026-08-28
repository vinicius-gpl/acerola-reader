import { expect, test } from '@playwright/test';

test.describe('mobile nav', () => {
	test.use({ viewport: { width: 390, height: 844 } });

	test('the mobile nav toggle opens and closes the nav drawer', async ({ page }) => {
		await page.goto('/docs/architecture');

		const toggle = page.getByRole('button', { name: 'Alternar menu' });
		await toggle.click();

		const dialog = page.getByRole('dialog');
		await expect(dialog).toBeVisible();
		await expect(dialog.getByRole('link', { name: 'Primeiros passos' })).toBeVisible();

		await dialog.getByRole('button', { name: 'Close' }).click();
		await expect(dialog).not.toBeVisible();
	});
});
