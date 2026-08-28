import { expect, test, type Page } from '@playwright/test';
import { BOOKMARKS_COMMANDS } from '../../svelte/src/lib/contracts/bookmarks/bookmarks.commands';
import { HISTORY_COMMANDS } from '../../svelte/src/lib/contracts/history/history.commands';
import { HOME_COMMANDS } from '../../svelte/src/lib/contracts/home/home.commands';
import { HOME_EVENTS } from '../../svelte/src/lib/contracts/home/home.events';
import { LIBRARY_COMMANDS } from '../../svelte/src/lib/contracts/library/chapter.commands';
import { LIBRARY_EVENTS } from '../../svelte/src/lib/contracts/library/chapter.events';
import { READER_COMMANDS } from '../../svelte/src/lib/contracts/reader/reader.commands';
import {
	e2eComic,
	e2eComicChapters,
	e2eComicSummary,
	e2eReaderPage,
	e2eReaderSession
} from '../../svelte/tests/mocks/acerola-e2e-data';
import {
	collectConsoleErrors,
	installTauriMocks,
	mockedTauriResponse
} from '../../svelte/tests/mocks/tauri-playwright';

async function openReader(page: Page) {
	await page.goto('/home');
	await page.getByRole('button', { name: 'Acerola' }).click();
	await expect(page).toHaveURL(/\/comic\/Acerola$/);
	const firstChapter = page.getByText('Chapter 1', { exact: true }).first();
	await expect(firstChapter).toBeVisible();
	await firstChapter.click();
	await expect(page).toHaveURL(/\/reader$/);
	await expect(page.getByRole('img', { name: 'Página 1' })).toBeVisible();
}

test.describe('reader', () => {
	let consoleErrors: string[];

	test.beforeEach(async ({ page }) => {
		consoleErrors = collectConsoleErrors(page);

		await installTauriMocks(page, {
			[HOME_COMMANDS.getComicSummarySorted]: mockedTauriResponse({
				events: [{ event: HOME_EVENTS.homeData, payload: e2eComicSummary([e2eComic]) }]
			}),
			[LIBRARY_COMMANDS.getComicByFolderName]: e2eComic,
			[LIBRARY_COMMANDS.getComicChapters]: mockedTauriResponse({
				delayMs: 50,
				events: [{ event: LIBRARY_EVENTS.comicChapters, payload: e2eComicChapters() }]
			}),
			[READER_COMMANDS.openChapter]: e2eReaderSession(),
			[READER_COMMANDS.loadPage]: (args) => e2eReaderPage((args as { index: number }).index),
			[READER_COMMANDS.setCurrentPage]: undefined,
			[READER_COMMANDS.prefetchWindow]: undefined,
			[READER_COMMANDS.closeChapter]: undefined,
			[HISTORY_COMMANDS.getComic]: null,
			[HISTORY_COMMANDS.getReadChapters]: [],
			[HISTORY_COMMANDS.updateReading]: undefined,
			[BOOKMARKS_COMMANDS.getComicCategory]: null
		});
	});

	test('navigates pages, opens command palette and applies zoom', async ({ page }) => {
		test.setTimeout(10_000);

		await openReader(page);

		await page.keyboard.press('ArrowRight');
		await expect(page.getByText(/2 \/ 3 páginas/)).toBeVisible();

		await page.keyboard.press('Control+K');
		await expect(page.getByText('Aumentar zoom')).toBeVisible();

		await page.keyboard.press('Escape');
		await expect(page.getByText('Aumentar zoom')).not.toBeVisible();
		await expect(page).not.toHaveURL(/\/home$/);

		await page.getByRole('button', { name: 'Aplicar zoom' }).click();
		await expect(page.getByText(/Zoom 165%/)).toBeVisible();

		// Navegação por teclado fica desabilitada de propósito enquanto a página está com
		// zoom aplicado no modo paginado (`pageControlsDisabled` em reader/+page.svelte) —
		// resetar o zoom antes de testar a troca de página, como um usuário real faria.
		await page.keyboard.press('Control+0');
		await expect(page.getByRole('button', { name: 'Aplicar zoom' })).toBeVisible();

		await page.keyboard.press('ArrowLeft');
		await expect(page.getByText(/1 \/ 3 páginas/)).toBeVisible();
		expect(consoleErrors).toEqual([]);
	});
});
