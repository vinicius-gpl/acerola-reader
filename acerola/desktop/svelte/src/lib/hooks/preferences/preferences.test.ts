import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';
import { STORE_KEYS } from '$lib/constants/store-plugin';
import { LIBRARY_COMMANDS } from '$lib/contracts/library/library.commands';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useComicInfoPreference } from './use-comic-info.svelte';
import { useVolumeViewMode } from './use-volume-view-mode.svelte';

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
