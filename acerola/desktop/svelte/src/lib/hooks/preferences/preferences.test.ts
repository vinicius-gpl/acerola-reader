import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';
import { STORE_KEYS } from '$lib/constants/store-plugin';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/library.commands';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import type { RelayInfo } from '$lib/contracts/network/network.payloads';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useComicInfoPreference } from './use-comic-info.svelte';
import { useVolumeViewMode } from './use-volume-view-mode.svelte';
import { useReaderMode } from './use-reader-mode.svelte';
import { useMetadataLanguage } from './use-metadata-language.svelte';
import { useRelaySettings } from './use-relay-settings.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const loadMock = vi.mocked(load);

type StoreMock = {
	get: ReturnType<typeof vi.fn>;
	set: ReturnType<typeof vi.fn>;
	delete: ReturnType<typeof vi.fn>;
	save: ReturnType<typeof vi.fn>;
};

async function renderHook<T>(create: () => T) {
	let hook: T | undefined;

	render(HookHarness, {
		props: {
			create,
			onReady: (value) => {
				hook = value as T;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

function mockStore(values: Record<string, unknown> = {}) {
	const data = { ...values };
	const store: StoreMock = {
		get: vi.fn((key: string) => Promise.resolve(data[key] ?? null)),
		set: vi.fn((key: string, value: unknown) => {
			data[key] = value;
			return Promise.resolve();
		}),
		delete: vi.fn((key: string) => {
			delete data[key];
			return Promise.resolve();
		}),
		save: vi.fn(() => Promise.resolve())
	};

	loadMock.mockResolvedValue(store as never);

	return store;
}

describe('useVolumeViewMode', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('starts with cover as the initial mode before any load', async () => {
		mockStore();
		const hook = await renderHook(useVolumeViewMode);

		expect(hook.volumeViewMode).toBe('cover');
	});

	it('loads saved volume view mode', async () => {
		mockStore({ [STORE_KEYS.volumeViewMode]: 'banner' });
		const hook = await renderHook(useVolumeViewMode);

		await hook.loadVolumeViewMode();

		expect(hook.volumeViewMode).toBe('banner');
	});

	it('uses cover as default mode when there is no saved value', async () => {
		mockStore();
		const hook = await renderHook(useVolumeViewMode);

		await hook.loadVolumeViewMode();

		expect(hook.volumeViewMode).toBe('cover');
	});

	it('saves volume view mode and updates visible state', async () => {
		const store = mockStore();
		const hook = await renderHook(useVolumeViewMode);

		await hook.saveVolumeViewMode('banner');

		expect(hook.volumeViewMode).toBe('banner');
		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.volumeViewMode, 'banner');
		expect(store.save).toHaveBeenCalledOnce();
	});

	it('allows setting the visible mode directly (without persisting)', async () => {
		const store = mockStore();
		const hook = await renderHook(useVolumeViewMode);

		hook.volumeViewMode = 'banner';

		expect(hook.volumeViewMode).toBe('banner');
		expect(store.set).not.toHaveBeenCalled();
	});
});

describe('useComicInfoPreference', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('saves preference with explicit value', async () => {
		const store = mockStore();
		const hook = await renderHook(useComicInfoPreference);

		await hook.selectComicInfoPreference(true);

		expect(hook.comicInfoPreference).toBe(true);
		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.comicInfoPreference, true);
		expect(store.save).toHaveBeenCalledOnce();
	});

	it('toggles preference when value is not passed', async () => {
		const store = mockStore();
		const hook = await renderHook(useComicInfoPreference);

		await hook.selectComicInfoPreference();

		expect(hook.comicInfoPreference).toBe(true);
		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.comicInfoPreference, true);
		expect(store.save).toHaveBeenCalledOnce();
	});

	it('loads saved comic info preference', async () => {
		mockStore({ [STORE_KEYS.comicInfoPreference]: true });
		const hook = await renderHook(useComicInfoPreference);

		await hook.loadSavedComicInfoPreference();

		expect(hook.comicInfoPreference).toBe(true);
	});
});

describe('useReaderMode', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('starts with vertical as the initial mode before any load', async () => {
		mockStore();
		const hook = await renderHook(useReaderMode);

		expect(hook.readerMode).toBe('vertical');
	});

	it('loads saved reader mode', async () => {
		mockStore({ [STORE_KEYS.readerMode]: 'webtoon' });
		const hook = await renderHook(useReaderMode);

		await hook.loadReaderMode();

		expect(hook.readerMode).toBe('webtoon');
	});

	it('falls back to vertical when there is no saved value', async () => {
		mockStore();
		const hook = await renderHook(useReaderMode);

		await hook.loadReaderMode();

		expect(hook.readerMode).toBe('vertical');
	});

	it('saves reader mode and updates visible state', async () => {
		const store = mockStore();
		const hook = await renderHook(useReaderMode);

		await hook.saveReaderMode('horizontal');

		expect(hook.readerMode).toBe('horizontal');
		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.readerMode, 'horizontal');
		expect(store.save).toHaveBeenCalledOnce();
	});

	it('allows setting the mode directly (without persisting)', async () => {
		const store = mockStore();
		const hook = await renderHook(useReaderMode);

		hook.readerMode = 'webtoon';

		expect(hook.readerMode).toBe('webtoon');
		expect(store.set).not.toHaveBeenCalled();
	});
});

describe('useMetadataLanguage', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('starts with pt-br as the initial language before any load', async () => {
		mockStore();
		const hook = await renderHook(useMetadataLanguage);

		expect(hook.metadataLanguage).toBe('pt-br');
	});

	it('loads saved metadata language', async () => {
		mockStore({ [STORE_KEYS.metadataLanguage]: 'en' });
		const hook = await renderHook(useMetadataLanguage);

		await hook.loadSavedMetadataLanguage();

		expect(hook.metadataLanguage).toBe('en');
	});

	it('keeps the default when there is no saved value', async () => {
		mockStore();
		const hook = await renderHook(useMetadataLanguage);

		await hook.loadSavedMetadataLanguage();

		expect(hook.metadataLanguage).toBe('pt-br');
	});

	it('selects and persists a new language', async () => {
		const store = mockStore();
		const hook = await renderHook(useMetadataLanguage);

		await hook.selectMetadataLanguage('es-la');

		expect(hook.metadataLanguage).toBe('es-la');
		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.metadataLanguage, 'es-la');
		expect(store.save).toHaveBeenCalledOnce();
	});
});

describe('useRelaySettings', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	function relayInfo(overrides: Partial<RelayInfo> = {}): RelayInfo {
		return {
			acerolaRelayUrl: 'https://relay.acerola-comic.com',
			useAcerolaRelay: true,
			useIrohPublicNetwork: false,
			customRelayUrls: [],
			hasIrohServicesTicket: false,
			...overrides
		};
	}

	it('loads relay info from the backend', async () => {
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);

		await hook.loadRelayInfo();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.getRelayInfo);
		expect(hook.relayInfo?.acerolaRelayUrl).toBe('https://relay.acerola-comic.com');
	});

	it('reports isMdnsOnly as false while the acerola relay is active', async () => {
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);

		await hook.loadRelayInfo();

		expect(hook.isMdnsOnly).toBe(false);
	});

	it('reports isMdnsOnly as true when every relay source is disabled', async () => {
		invokeMock.mockResolvedValue(relayInfo({ useAcerolaRelay: false }));
		const hook = await renderHook(useRelaySettings);

		await hook.loadRelayInfo();

		expect(hook.isMdnsOnly).toBe(true);
	});

	it('persists toggling the acerola relay off', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.setUseAcerolaRelay(false);

		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.relayUseAcerola, false);
		expect(store.save).toHaveBeenCalledOnce();
		expect(hook.relayInfo?.useAcerolaRelay).toBe(false);
	});

	it('persists toggling the iroh public network on', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.setUseIrohPublicNetwork(true);

		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.relayUseIrohPublic, true);
		expect(hook.relayInfo?.useIrohPublicNetwork).toBe(true);
	});

	it('adds a custom relay url', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.addCustomRelayUrl('https://relay-a.test.local');

		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.relayCustomUrls, [
			'https://relay-a.test.local'
		]);
		expect(hook.relayInfo?.customRelayUrls).toEqual(['https://relay-a.test.local']);
	});

	it('ignores blank or duplicate custom relay urls', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo({ customRelayUrls: ['https://relay-a.test.local'] }));
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.addCustomRelayUrl('   ');
		await hook.addCustomRelayUrl('https://relay-a.test.local');

		expect(store.set).not.toHaveBeenCalled();
	});

	it('removes a custom relay url', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(
			relayInfo({ customRelayUrls: ['https://relay-a.test.local', 'https://relay-b.test.local'] })
		);
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.removeCustomRelayUrl('https://relay-a.test.local');

		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.relayCustomUrls, [
			'https://relay-b.test.local'
		]);
		expect(hook.relayInfo?.customRelayUrls).toEqual(['https://relay-b.test.local']);
	});

	it('sets the iroh services ticket via the backend command, not the store', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.setIrohServicesTicket('services-fake-ticket');

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.setIrohServicesTicket, {
			ticket: 'services-fake-ticket'
		});
		expect(store.set).not.toHaveBeenCalled();
		expect(hook.relayInfo?.hasIrohServicesTicket).toBe(true);
	});

	it('propagates a backend error when the ticket is invalid', async () => {
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		invokeMock.mockRejectedValueOnce(new Error('invalid ticket'));

		await expect(hook.setIrohServicesTicket('not-a-valid-ticket')).rejects.toThrow(
			'invalid ticket'
		);
		expect(hook.relayInfo?.hasIrohServicesTicket).toBe(false);
	});

	it('clears the iroh services ticket via the backend command', async () => {
		invokeMock.mockResolvedValue(relayInfo({ hasIrohServicesTicket: true }));
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.clearIrohServicesTicket();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.clearIrohServicesTicket);
		expect(hook.relayInfo?.hasIrohServicesTicket).toBe(false);
	});

	it('restartP2p calls the restart_p2p command', async () => {
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook(useRelaySettings);

		await hook.restartP2p();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.restartP2p);
	});

	it('restartP2p propagates a backend error', async () => {
		invokeMock.mockRejectedValueOnce(new Error('restart failed'));
		const hook = await renderHook(useRelaySettings);

		await expect(hook.restartP2p()).rejects.toThrow('restart failed');
	});
});
