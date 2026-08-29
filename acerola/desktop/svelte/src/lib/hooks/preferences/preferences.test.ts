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
		return { defaultRelay: 'relay.default.example', activeRelay: 'relay.default.example', ...overrides };
	}

	it('loads relay info from the backend', async () => {
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);

		await hook.loadRelayInfo();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.getRelayInfo);
		expect(hook.relayInfo?.defaultRelay).toBe('relay.default.example');
	});

	it('reports isOverridden as false when active relay matches the default', async () => {
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);

		await hook.loadRelayInfo();

		expect(hook.isOverridden).toBe(false);
	});

	it('reports isOverridden as true when active relay differs from the default', async () => {
		invokeMock.mockResolvedValue(relayInfo({ activeRelay: 'relay.custom.example' }));
		const hook = await renderHook(useRelaySettings);

		await hook.loadRelayInfo();

		expect(hook.isOverridden).toBe(true);
	});

	it('persists a custom relay override', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.setRelayUrl('relay.custom.example');

		expect(store.set).toHaveBeenCalledWith(STORE_KEYS.relayUrl, 'relay.custom.example');
		expect(store.save).toHaveBeenCalledOnce();
	});

	it('clears the override when the value matches the default relay', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.setRelayUrl('relay.default.example');

		expect(store.delete).toHaveBeenCalledWith(STORE_KEYS.relayUrl);
		expect(store.set).not.toHaveBeenCalled();
		expect(store.save).toHaveBeenCalledOnce();
	});

	it('clears the override when the value is blank', async () => {
		const store = mockStore();
		invokeMock.mockResolvedValue(relayInfo());
		const hook = await renderHook(useRelaySettings);
		await hook.loadRelayInfo();

		await hook.setRelayUrl('   ');

		expect(store.delete).toHaveBeenCalledWith(STORE_KEYS.relayUrl);
		expect(store.set).not.toHaveBeenCalled();
	});

	it('resets the relay override', async () => {
		const store = mockStore();
		const hook = await renderHook(useRelaySettings);

		await hook.resetRelayUrl();

		expect(store.delete).toHaveBeenCalledWith(STORE_KEYS.relayUrl);
		expect(store.save).toHaveBeenCalledOnce();
	});
});
