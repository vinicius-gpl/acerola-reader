import { expect, test } from '@playwright/test';

// A ordem "achatada" da sidebar (src/lib/content/docs.ts) segue SECTION_ORDER e depois
// `order` dentro de cada seção — hoje isso deixa "architecture" (Conceitos) entre
// "contributing-p2p" (Contribuindo) e "privacy-policy" (Privacidade). Se um novo doc
// for inserido entre essas seções, ou a ordem de SECTION_ORDER mudar, os vizinhos
// esperados aqui também mudam — não é alfabético nem baseado no nome do arquivo.
test.describe('prev/next doc navigation', () => {
	test('prev/next links navigate between adjacent docs', async ({ page }) => {
		await page.goto('/docs/architecture');
		await expect(page.getByRole('heading', { level: 1, name: 'Arquitetura' })).toBeVisible();

		// A navegação Anterior/Próximo aparece duas vezes (acima e abaixo do
		// conteúdo) — `.first()` pega a do topo, mas as duas levam ao mesmo lugar.
		await page
			.getByRole('link', { name: /Próximo/ })
			.first()
			.click();

		await expect(page).toHaveURL(/\/docs\/privacy-policy$/);
		await expect(
			page.getByRole('heading', { level: 1, name: 'Política de Privacidade' })
		).toBeVisible();

		await page
			.getByRole('link', { name: /Anterior/ })
			.first()
			.click();

		await expect(page).toHaveURL(/\/docs\/architecture$/);
		await expect(page.getByRole('heading', { level: 1, name: 'Arquitetura' })).toBeVisible();
	});
});
