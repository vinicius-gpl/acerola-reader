import { expect, test } from '@playwright/test';

// This spec needs the production build + Pagefind index to exist on disk (see
// playwright.config.ts) — `npm run build` must run before `playwright test`.
test.describe('search dialog', () => {
	test('opens with Control+K and returns real Pagefind results', async ({ page }) => {
		await page.goto('/');

		await page.keyboard.press('Control+k');
		const dialog = page.getByRole('dialog');
		// A home roda o fundo WebGL (faulty-terminal.svelte) desde o goto acima; sob o GPU
		// virtual do Chromium headless no CI isso compete com o Ctrl+K e passa fácil dos 5s
		// padrão antes do dialog abrir — mesma margem já dada à asserção do Pagefind abaixo.
		await expect(dialog).toBeVisible({ timeout: 10_000 });

		await dialog.getByPlaceholder('Digite para pesquisar...').fill('monorepo');

		// A primeira consulta carrega o WASM/índice do Pagefind sob demanda — mais lenta que
		// o timeout padrão de 5s, então essa asserção específica ganha mais margem.
		const result = dialog.locator('a[href*="/docs/architecture"]');
		await expect(result).toBeVisible({ timeout: 10_000 });
		await expect(result).toContainText('Arquitetura');
	});
});
