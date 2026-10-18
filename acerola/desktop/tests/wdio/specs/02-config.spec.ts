import {
	exactTextSelector,
	firstDisplayed,
	getPathname,
	navigateTo,
	waitForAppReady,
	waitForText
} from '../helpers/app';

async function currentTheme() {
	return browser.execute(() => document.documentElement.getAttribute('data-theme') ?? '');
}

async function openConfig() {
	await waitForAppReady();
	await navigateTo('/config');
	await waitForText('Configurações');
}

describe('native config', () => {
	it('renders main sections and controls', async () => {
		await openConfig();

		await waitForText('Personalize sua experiência de leitura.');
		await waitForText('Configuração dos Arquivos');
		await waitForText('Pasta dos quadrinhos');
		await waitForText('Gerar ComicInfo.xml');
		await waitForText('Biblioteca');
		await waitForText('Sincronização rápida');
		await waitForText('Sincronização profunda');
		await waitForText('Configuração de metadados');
		await waitForText('Idioma');
		await waitForText('Sincronização com MangaDex');
		await waitForText('Sincronização com AniList');
	});

	it('applies only the light/dark mode to the document (no custom palettes)', async () => {
		await openConfig();

		expect(['light', 'dark']).toContain(await currentTheme());
	});

	it('opens sidebar language selector without changing route', async () => {
		await openConfig();

		// O select fica dentro do popover do ícone de globo no header — só
		// existe no DOM depois do popover aberto (ver +layout.svelte).
		await (await firstDisplayed('[title="Mudar idioma"]')).click();

		const localeTrigger = await firstDisplayed('[data-slot="select-trigger"]');
		await localeTrigger.click();

		await waitForText('English');
		await waitForText('Português');
		await browser.keys(['Escape']);

		expect(await getPathname()).toBe('/config');
	});

	it('opens metadata languages popover', async () => {
		await openConfig();

		const languageButton = await firstDisplayed(
			`${exactTextSelector('Idioma')}/ancestor::*[@data-slot="item"][1]//button`
		);
		await languageButton.click();

		await waitForText('Português (BR)');
		await waitForText('English');
		await waitForText('日本語');
		await browser.keys(['Escape']);

		expect(await getPathname()).toBe('/config');
	});
});
