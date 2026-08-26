import { expect, test } from '@playwright/test';
import { HOME_COMMANDS } from '../../svelte/src/lib/contracts/home/home.commands';
import { HOME_EVENTS } from '../../svelte/src/lib/contracts/home/home.events';
import { STORE_FILE, STORE_KEYS } from '../../svelte/src/lib/constants/store-plugin';
import {
	collectConsoleErrors,
	installTauriMocks,
	mockedTauriResponse,
	tauriCommandCalls
} from '../../svelte/tests/mocks/tauri-playwright';

test.describe('IPC error on home', () => {
	let consoleErrors: string[];

	test.beforeEach(async ({ page }) => {
		consoleErrors = collectConsoleErrors(page);

		await installTauriMocks(
			page,
			{
				[HOME_COMMANDS.getComicSummarySorted]: mockedTauriResponse({
					events: [
						{
							event: HOME_EVENTS.homeError,
							payload: { errorType: 'Unknown', message: 'db_error' }
						}
					]
				})
			},
			{ storeSeed: { [STORE_FILE]: { [STORE_KEYS.onboardingCompleted]: true } } }
		);
	});

	test('displays library error without infinite retry loop', async ({ page }) => {
		test.setTimeout(10_000);

		await page.goto('/home');

		await expect(page.getByText('db_error')).toBeVisible();
		await expect(page).not.toHaveURL(/\/comic\//);
		await expect
			.poll(() => tauriCommandCalls(page, HOME_COMMANDS.getComicSummarySorted))
			.toBeLessThanOrEqual(1);
		expect(consoleErrors).toEqual([]);
	});
});
