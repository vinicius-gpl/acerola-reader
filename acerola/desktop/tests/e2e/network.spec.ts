import { expect, test } from '@playwright/test';
import { NETWORK_COMMANDS } from '../../svelte/src/lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '../../svelte/src/lib/contracts/network/network.events';
import {
	e2eLocalAddr,
	e2eLocalDeviceInfo,
	e2eLocalPeerId,
	e2ePairedPeer,
	e2eRelayInfo
} from '../../svelte/tests/mocks/acerola-e2e-data';
import {
	collectConsoleErrors,
	installTauriMocks,
	mockedTauriResponse
} from '../../svelte/tests/mocks/tauri-playwright';

// Substitui a antiga history-network.spec.ts: aquele teste esperava uma troca manual de
// modo local/relay em `/history` (texto "Modo: local" + botão "Relay") que não existe mais
// no app — `switch_to_local`/`switch_to_relay` não são chamados por nenhum componente hoje.
// O status de rede (somente leitura) e a lista de peers pareados vivem em `/network`, que
// não tinha nenhuma cobertura e2e até então.
test.describe('network page', () => {
	let consoleErrors: string[];

	test.beforeEach(async ({ page }) => {
		consoleErrors = collectConsoleErrors(page);

		await installTauriMocks(page, {
			[NETWORK_COMMANDS.getSecurityStatus]: false,
			[NETWORK_COMMANDS.getLocalId]: e2eLocalPeerId,
			[NETWORK_COMMANDS.getLocalAddr]: e2eLocalAddr,
			[NETWORK_COMMANDS.getLocalDeviceInfo]: e2eLocalDeviceInfo,
			[NETWORK_COMMANDS.getRelayInfo]: e2eRelayInfo,
			[NETWORK_COMMANDS.getPairedPeers]: [e2ePairedPeer],
			[NETWORK_COMMANDS.getNetworkStatus]: mockedTauriResponse({
				// `refreshStatus()` ignora o valor de retorno do invoke — o status só chega pelo
				// evento `network:status`, exatamente como o backend real se comporta (ver o
				// comentário em `network.events.ts` sobre esse evento nunca ser espontâneo).
				events: [{ event: NETWORK_EVENTS.status, payload: { mode: 'local', peers: [] } }]
			})
		});
	});

	test('shows device identity, local mode and paired peer', async ({ page }) => {
		test.setTimeout(10_000);

		await page.goto('/network');

		await expect(page.getByText('Meu PC')).toBeVisible();
		await expect(page.getByText('Rede local')).toBeVisible();

		await expect(page.getByText('Meu Celular')).toBeVisible();
		await expect(page.getByText('Nunca sincronizado')).toBeVisible();

		expect(consoleErrors).toEqual([]);
	});
});
