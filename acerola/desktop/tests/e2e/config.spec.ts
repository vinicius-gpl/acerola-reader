import { expect, test } from '@playwright/test';
import { HOME_COMMANDS } from '../../svelte/src/lib/contracts/home/home.commands';
import { HOME_EVENTS } from '../../svelte/src/lib/contracts/home/home.events';
import {
	DIRECTORY_SCAN_COMMANDS,
	LIBRARY_COMMANDS
} from '../../svelte/src/lib/contracts/library/library.commands';
import { LIBRARY_EVENTS } from '../../svelte/src/lib/contracts/library/library.events';
import { e2eComicSummary } from '../../svelte/tests/mocks/acerola-e2e-data';
import {
	collectConsoleErrors,
	emitTauriEvent,
	installTauriMocks,
	mockedTauriResponse
} from '../../svelte/tests/mocks/tauri-playwright';

test.describe('library configuration', () => {
	let consoleErrors: string[];

	test.beforeEach(async ({ page }) => {
		consoleErrors = collectConsoleErrors(page);

		await installTauriMocks(page, {
			[LIBRARY_COMMANDS.selectFolder]: 'C:\\Comics',
			[DIRECTORY_SCAN_COMMANDS.refreshLibrary]: undefined,
			[HOME_COMMANDS.getComicSummarySorted]: mockedTauriResponse({
				events: [{ event: HOME_EVENTS.homeData, payload: e2eComicSummary() }]
			})
		});
	});

	test('selects folder, executes quick sync and returns to library', async ({ page }) => {
		test.setTimeout(10_000);

		await page.goto('/config');

		// Categorias vêm colapsadas por padrão (accordion) — precisa expandir antes de
		// conseguir clicar em itens que ficam dentro do corpo de cada card.
		await page.getByText('Configuração dos Arquivos').click();
		await page.getByText('Pasta dos quadrinhos').click();
		await expect(page).not.toHaveURL(/\/home$/);
		await expect(page.getByText(/C:\\Comics/)).toBeVisible();

		await page.getByText('Biblioteca', { exact: true }).click();
		await page.getByText('Sincronização rápida').click();
		await expect(page).not.toHaveURL(/\/home$/);

		await emitTauriEvent(page, LIBRARY_EVENTS.scanComplete);

		await page.goto('/home');

		await expect(page.getByRole('heading', { name: 'Acerola' })).toBeVisible();
		expect(consoleErrors).toEqual([]);
	});
});
