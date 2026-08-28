import { expect, test } from '@playwright/test';

// Theme persistence uses localStorage (THEME_STORAGE_KEY / MODE_STORAGE_KEY from
// src/lib/constants/themes.ts), not cookies. Default state on first load is
// theme "catppuccin" / mode "dark" (see use-theme.svelte.ts). The picker cycles
// mode dark -> system -> light -> dark on each click.
test.describe('theme picker', () => {
	test('persists the selected theme mode across reloads', async ({ page }) => {
		// TODO (contornado, não corrigido): uma doc page, não a landing (`/`) — a landing
		// renderiza `faulty-terminal.svelte` (fundo animado em WebGL/ogl, ver TODO lá) que
		// gera "GPU stall" sob o GPU virtual do Chromium headless e derruba a página,
		// principalmente em `page.reload()`. O theme picker não depende da landing pra
		// funcionar, então testa aqui — mas isso significa que a landing em si nunca é
		// testada com reload/interação pesada, só o smoke test em landing.spec.ts.
		await page.goto('/docs/architecture');

		const button = page.getByRole('button', { name: 'Mudar tema' });
		await button.click(); // dark -> system
		// O ícone dentro do botão é recriado a cada mudança de modo (`{#key themeCtx.mode}`
		// em theme-picker.svelte) — clicar de novo antes desse re-render assentar faz o
		// Playwright ficar esperando indefinidamente a estabilidade do elemento.
		await expect
			.poll(() => page.evaluate(() => localStorage.getItem('acerola-docs:mode')))
			.toBe('system');
		await button.click(); // system -> light

		await expect
			.poll(() => page.evaluate(() => localStorage.getItem('acerola-docs:mode')))
			.toBe('light');
		await expect(page.locator('html')).not.toHaveClass(/dark/);
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'catppuccin-latte');

		await page.reload();

		await expect
			.poll(() => page.evaluate(() => localStorage.getItem('acerola-docs:mode')))
			.toBe('light');
		await expect(page.locator('html')).not.toHaveClass(/dark/);
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'catppuccin-latte');
	});
});
