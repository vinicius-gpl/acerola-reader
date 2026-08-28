import { expect, test } from '@playwright/test';
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

test.describe('library comic reader navigation', () => {
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

	test('opens comic card, chooses first chapter and renders first page', async ({ page }) => {
		test.setTimeout(10_000);

		await page.goto('/home');

		await page.getByRole('button', { name: 'Acerola' }).click();

		await expect(page).toHaveURL(/\/comic\/Acerola$/);
		const firstChapter = page.getByText('Chapter 1', { exact: true }).first();
		await expect(firstChapter).toBeVisible();

		await firstChapter.click();

		await expect(page).toHaveURL(/\/reader$/);
		await expect(page).not.toHaveURL(/\/comic\/Acerola$/);
		await expect(page.getByRole('img', { name: 'Página 1' })).toBeVisible();
		expect(consoleErrors).toEqual([]);
	});
});
