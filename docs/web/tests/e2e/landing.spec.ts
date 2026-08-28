import { expect, test } from '@playwright/test';

test.describe('landing page', () => {
	test('renders hero content with no console errors', async ({ page }) => {
		const consoleErrors: string[] = [];
		page.on('console', (msg) => {
			if (msg.type() === 'error') consoleErrors.push(msg.text());
		});
		page.on('pageerror', (err) => consoleErrors.push(err.message));

		await page.goto('/');

		await expect(page).toHaveTitle('Acerola — Docs');
		await expect(page.getByRole('heading', { level: 1 })).toContainText(
			'Leia seus quadrinhos e mangás em qualquer dispositivo. Sem servidor, sem conta.'
		);
		await expect(page.getByRole('link', { name: 'Começar' })).toBeVisible();
		await expect(page.getByRole('link', { name: 'Ver no GitHub' })).toBeVisible();

		expect(consoleErrors).toEqual([]);
	});
});
