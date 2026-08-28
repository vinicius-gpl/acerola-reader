import type { Page } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { BOOKMARKS_COMMANDS } from '../../src/lib/contracts/bookmarks/bookmarks.commands';
import { NETWORK_COMMANDS } from '../../src/lib/contracts/network/network.commands';
import { STORE_FILE, STORE_KEYS } from '../../src/lib/constants/store-plugin';

const require = createRequire(import.meta.url);
const tauriMocksPath = require.resolve('@tauri-apps/api/mocks');

export type MockedTauriEvent = {
	event: string;
	payload: unknown;
};

export type MockedTauriResponse = {
	__acerolaTauriResponse: true;
	value?: unknown;
	events?: MockedTauriEvent[];
	error?: string;
	delayMs?: number;
};

export type MockedTauriHandler =
	unknown | ((args: unknown, command: string) => unknown | Promise<unknown>);

declare global {
	interface Window {
		__acerolaTauriHandleCommand?: (command: string, args?: unknown) => Promise<unknown>;
		__acerolaTauriCalls?: Record<string, number>;
	}
}

export function mockedTauriResponse(
	response: Omit<MockedTauriResponse, '__acerolaTauriResponse'>
): MockedTauriResponse {
	return {
		__acerolaTauriResponse: true,
		...response
	};
}

export type InstallTauriMocksOptions = {
	/** Pré-popula o `@tauri-apps/plugin-store` mockado antes do app iniciar — chave é o
	 *  nome do arquivo de store (ex. `settings.json`, ver `STORE_FILE`), valor é o mapa
	 *  chave/valor já persistido (ex. `{ onboarding_completed: true }`, ver `STORE_KEYS`).
	 *  Sem isso o store some vazio e o app sempre mostra o onboarding, já que
	 *  `useOnboarding().isCompleted` lê `onboarding_completed` do store. */
	storeSeed?: Record<string, Record<string, unknown>>;
};

// Chamadas de IPC feitas incondicionalmente pelo `+layout.svelte` (bookmarks + identidade
// do pacote em DEV) e pela home (`peers.startListening()`/`sync.startListening()`) em toda
// navegação — sem isso toda spec precisaria mockar esse boilerplate só pra não poluir
// `collectConsoleErrors` com "unmocked: ...".
const LAYOUT_BOILERPLATE_HANDLERS: Record<string, MockedTauriHandler> = {
	[BOOKMARKS_COMMANDS.getCategories]: [],
	[BOOKMARKS_COMMANDS.getAllComicCategories]: [],
	get_package_family_name: 'No package identity',
	[NETWORK_COMMANDS.getNetworkStatus]: { mode: 'local', peers: [] },
	[NETWORK_COMMANDS.getPairedPeers]: [],
	[NETWORK_COMMANDS.getSyncHistoryLog]: []
};

export async function installTauriMocks(
	page: Page,
	handlers: Record<string, MockedTauriHandler>,
	options: InstallTauriMocksOptions = {}
) {
	const mergedHandlers = { ...LAYOUT_BOILERPLATE_HANDLERS, ...handlers };

	// Por padrão toda spec pula o onboarding (o gate em `+layout.svelte` mostra o wizard
	// pra qualquer rota enquanto `onboarding_completed` não estiver setado) — uma spec que
	// realmente quer testar o onboarding sobrescreve isso explicitamente via `storeSeed`.
	const storeSeed = {
		...options.storeSeed,
		[STORE_FILE]: {
			[STORE_KEYS.onboardingCompleted]: true,
			...options.storeSeed?.[STORE_FILE]
		}
	};

	await page.route('http://asset.localhost/**', (route) =>
		route.fulfill({
			status: 200,
			contentType: 'image/svg+xml',
			body: '<svg xmlns="http://www.w3.org/2000/svg" width="320" height="480"><rect width="320" height="480" fill="#222"/><text x="160" y="240" fill="#fff" font-family="Arial" font-size="32" text-anchor="middle">Acerola</text></svg>'
		})
	);

	await page.exposeFunction(
		'__acerolaTauriHandleCommand',
		async (command: string, args?: unknown) => {
			if (!(command in mergedHandlers)) {
				throw new Error(`unmocked: ${command}`);
			}

			const handler = mergedHandlers[command];

			return typeof handler === 'function' ? await handler(args, command) : handler;
		}
	);

	const tauriMocksSource = await readFile(tauriMocksPath, 'utf8');

	await page.addInitScript({
		content: `
(() => {
  const module = { exports: {} };
  const exports = module.exports;
  ${tauriMocksSource}

  module.exports.clearMocks();
  module.exports.mockWindows('main');
  module.exports.mockConvertFileSrc('windows');

  window.__acerolaTauriCalls = {};

  let nextStoreRid = 1;
  const storePathToRid = new Map();
  const stores = new Map();

  function storeForPath(path) {
    if (storePathToRid.has(path)) return storePathToRid.get(path);

    const rid = nextStoreRid++;
    storePathToRid.set(path, rid);
    stores.set(rid, new Map());
    return rid;
  }

  function getStore(rid) {
    const store = stores.get(rid);
    if (!store) throw new Error('unknown store rid: ' + rid);
    return store;
  }

  const storeSeed = ${JSON.stringify(storeSeed)};
  for (const [path, entries] of Object.entries(storeSeed)) {
    const rid = storeForPath(path);
    for (const [key, value] of Object.entries(entries)) {
      stores.get(rid).set(key, value);
    }
  }

  function scheduleEvents(events, delayMs = 0) {
    if (!events?.length) return;

    window.setTimeout(() => {
      for (const item of events) {
        window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {
          event: item.event,
          payload: item.payload
        });
      }
    }, delayMs);
  }

  function handleStoreCommand(command, args) {
    if (command === 'plugin:store|load') return storeForPath(args.path);
    if (command === 'plugin:store|get_store') return storePathToRid.get(args.path) ?? null;

    if (command === 'plugin:store|get') {
      const store = getStore(args.rid);
      const exists = store.has(args.key);
      return [exists ? store.get(args.key) : null, exists];
    }

    if (command === 'plugin:store|set') {
      getStore(args.rid).set(args.key, args.value);
      return undefined;
    }

    if (command === 'plugin:store|has') return getStore(args.rid).has(args.key);

    if (command === 'plugin:store|delete') {
      getStore(args.rid).delete(args.key);
      return undefined;
    }

    if (command === 'plugin:store|clear' || command === 'plugin:store|reset') {
      getStore(args.rid).clear();
      return undefined;
    }

    if (command === 'plugin:store|keys') return Array.from(getStore(args.rid).keys());
    if (command === 'plugin:store|values') return Array.from(getStore(args.rid).values());
    if (command === 'plugin:store|entries') return Array.from(getStore(args.rid).entries());
    if (command === 'plugin:store|length') return getStore(args.rid).size;

    if (
      command === 'plugin:store|save' ||
      command === 'plugin:store|reload' ||
      command === 'plugin:resources|close'
    ) {
      return undefined;
    }
  }

  module.exports.mockIPC(async (command, args) => {
    if (command.startsWith('plugin:store|') || command === 'plugin:resources|close') {
      return handleStoreCommand(command, args ?? {});
    }

    if (command === 'plugin:window|theme') return 'dark';
    if (command.startsWith('plugin:window|') || command === 'plugin:log|log') return undefined;

    window.__acerolaTauriCalls[command] = (window.__acerolaTauriCalls[command] ?? 0) + 1;

    let response;
    try {
      response = await window.__acerolaTauriHandleCommand(command, args);
    } catch (error) {
      console.error('[tauri mock] ' + command + ': ' + (error?.message ?? String(error)));
      throw error;
    }

    if (response?.__acerolaTauriResponse) {
      scheduleEvents(response.events, response.delayMs ?? 0);

      if (response.error) {
        throw new Error(response.error);
      }

      return response.value;
    }

    return response;
  }, { shouldMockEvents: true });
})();
`
	});
}

export async function emitTauriEvent(page: Page, event: string, payload: unknown = null) {
	await page.evaluate(
		({ event: eventName, payload: eventPayload }) =>
			window.__TAURI_INTERNALS__?.invoke('plugin:event|emit', {
				event: eventName,
				payload: eventPayload
			}),
		{ event, payload }
	);
}

export async function tauriCommandCalls(page: Page, command: string) {
	return page.evaluate((cmd) => window.__acerolaTauriCalls?.[cmd] ?? 0, command);
}

export function collectConsoleErrors(page: Page) {
	const errors: string[] = [];

	page.on('console', (message) => {
		if (message.type() === 'error') {
			errors.push(message.text());
		}
	});

	page.on('pageerror', (error) => {
		errors.push(error.message);
	});

	return errors;
}
