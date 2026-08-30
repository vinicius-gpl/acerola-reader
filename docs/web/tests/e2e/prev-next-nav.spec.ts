import { expect, test } from '@playwright/test';

// "architecture" and "getting-started" are the only two docs, so they are each
// other's only adjacent entry in the flattened sidebar order from
// src/lib/content/docs.ts (architecture sorts first alphabetically).
test.describe('prev/next doc navigation', () => {
	test('prev/next links navigate between adjacent docs', async ({ page }) => {
		await page.goto('/docs/architecture');
		await expect(page.getByRole('heading', { level: 1, name: 'Arquitetura' })).toBeVisible();

		// A navegação Anterior/Próximo aparece duas vezes (acima e abaixo do
		// conteúdo) — `.first()` pega a do topo, mas as duas levam ao mesmo lugar.
		await page.getByRole('link', { name: /Próximo/ }).first().click();

		await expect(page).toHaveURL(/\/docs\/getting-started$/);
		await expect(page.getByRole('heading', { level: 1, name: 'Primeiros passos' })).toBeVisible();

		await page.getByRole('link', { name: /Anterior/ }).first().click();

		await expect(page).toHaveURL(/\/docs\/architecture$/);
		await expect(page.getByRole('heading', { level: 1, name: 'Arquitetura' })).toBeVisible();
	});
});
