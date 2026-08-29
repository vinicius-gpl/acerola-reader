import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { error as tauriError } from '@tauri-apps/plugin-log';
import { load } from '@tauri-apps/plugin-store';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useMetadataSync } from './use-metadata-sync.svelte';
import { METADATA_COMMANDS } from '$lib/contracts/metadata/metadata.commands';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn().mockResolvedValue({
		get: vi.fn().mockResolvedValue('test')
	})
}));

const invokeMock = vi.mocked(invoke);
const errorMock = vi.mocked(tauriError);
const loadMock = vi.mocked(load);

async function renderMetadataSyncHook() {
	let hook: ReturnType<typeof useMetadataSync> | undefined;

	render(HookHarness, {
		props: {
			create: () => useMetadataSync(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useMetadataSync>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

describe('useMetadataSync', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('syncs metadata with MangaDex successfully', async () => {
		const mockResponse = { id: '1', title: 'Test', hasComicInfo: false };
		invokeMock.mockResolvedValueOnce(mockResponse);

		const hook = await renderMetadataSyncHook();

		expect(hook.isSyncing).toBe(false);
		const result = await hook.syncMangadex('Naruto', '123');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.syncMangadex, {
			title: 'Naruto',
			comicId: '123',
			generateComicInfo: 'test',
			language: 'test'
		});
		expect(result).toEqual(mockResponse);
		expect(hook.isSyncing).toBe(false);
	});

	it('is syncing while the MangaDex request is in flight', async () => {
		let resolveSync: (value: unknown) => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = resolve;
				})
		);
		const hook = await renderMetadataSyncHook();

		const pending = hook.syncMangadex('Naruto', '123');
		// `syncMangadex`/`syncAnilist` esperam `load()` + 2x `store.get()` antes de chegar
		// no `invoke` — um macrotask garante que o mock já capturou `resolveSync` de verdade.
		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(hook.isSyncing).toBe(true);

		resolveSync({ id: '1', title: 'Test', hasComicInfo: false });
		await pending;
		expect(hook.isSyncing).toBe(false);
	});

	it('handles error when syncing metadata with MangaDex', async () => {
		invokeMock.mockRejectedValueOnce('Network error');

		const hook = await renderMetadataSyncHook();

		await expect(hook.syncMangadex('Naruto', '123')).rejects.toEqual('Network error');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.syncMangadex, {
			title: 'Naruto',
			comicId: '123',
			generateComicInfo: 'test',
			language: 'test'
		});
		expect(errorMock).toHaveBeenCalledWith('Failed to sync MangaDex: "Network error"');
		expect(hook.isSyncing).toBe(false);
	});

	it('falls back to pt-br/false when the store has no saved language/preference (MangaDex)', async () => {
		loadMock.mockResolvedValueOnce({ get: vi.fn().mockResolvedValue(null) } as never);
		invokeMock.mockResolvedValueOnce({ id: '1', title: 'Test', hasComicInfo: false });

		const hook = await renderMetadataSyncHook();
		await hook.syncMangadex('Naruto', '123');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.syncMangadex, {
			title: 'Naruto',
			comicId: '123',
			generateComicInfo: false,
			language: 'pt-br'
		});
	});

	it('syncs metadata with AniList successfully', async () => {
		const mockResponse = { id: '2', title: 'Test 2', hasComicInfo: false };
		invokeMock.mockResolvedValueOnce(mockResponse);

		const hook = await renderMetadataSyncHook();

		expect(hook.isSyncing).toBe(false);
		const result = await hook.syncAnilist('Bleach', '456');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.syncAnilist, {
			title: 'Bleach',
			comicId: '456',
			generateComicInfo: 'test',
			language: 'test'
		});
		expect(result).toEqual(mockResponse);
		expect(hook.isSyncing).toBe(false);
	});

	it('is syncing while the AniList request is in flight', async () => {
		let resolveSync: (value: unknown) => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = resolve;
				})
		);
		const hook = await renderMetadataSyncHook();

		const pending = hook.syncAnilist('Bleach', '456');
		// `syncMangadex`/`syncAnilist` esperam `load()` + 2x `store.get()` antes de chegar
		// no `invoke` — um macrotask garante que o mock já capturou `resolveSync` de verdade.
		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(hook.isSyncing).toBe(true);

		resolveSync({ id: '2', title: 'Test 2', hasComicInfo: false });
		await pending;
		expect(hook.isSyncing).toBe(false);
	});

	it('handles error when syncing metadata with AniList', async () => {
		invokeMock.mockRejectedValueOnce('API error');

		const hook = await renderMetadataSyncHook();

		await expect(hook.syncAnilist('Bleach', '456')).rejects.toEqual('API error');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.syncAnilist, {
			title: 'Bleach',
			comicId: '456',
			generateComicInfo: 'test',
			language: 'test'
		});
		expect(errorMock).toHaveBeenCalledWith('Failed to sync AniList: "API error"');
		expect(hook.isSyncing).toBe(false);
	});

	it('falls back to pt-br/false when the store has no saved language/preference (AniList)', async () => {
		loadMock.mockResolvedValueOnce({ get: vi.fn().mockResolvedValue(null) } as never);
		invokeMock.mockResolvedValueOnce({ id: '2', title: 'Test 2', hasComicInfo: false });

		const hook = await renderMetadataSyncHook();
		await hook.syncAnilist('Bleach', '456');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.syncAnilist, {
			title: 'Bleach',
			comicId: '456',
			generateComicInfo: false,
			language: 'pt-br'
		});
	});

	it('syncs ComicInfo.xml successfully', async () => {
		const mockResponse = { id: '3', title: 'Test 3', hasComicInfo: true };
		invokeMock.mockResolvedValueOnce(mockResponse);

		const hook = await renderMetadataSyncHook();
		const result = await hook.syncComicInfo('789');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.syncComicInfo, { comicId: '789' });
		expect(result).toEqual(mockResponse);
		expect(hook.isSyncing).toBe(false);
	});

	it('is syncing while syncComicInfo is in flight', async () => {
		let resolveSync: (value: unknown) => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = resolve;
				})
		);
		const hook = await renderMetadataSyncHook();

		const pending = hook.syncComicInfo('789');
		await Promise.resolve();
		expect(hook.isSyncing).toBe(true);

		resolveSync({ id: '3', title: 'Test 3', hasComicInfo: true });
		await pending;
		expect(hook.isSyncing).toBe(false);
	});

	it('handles error when syncing ComicInfo.xml', async () => {
		invokeMock.mockRejectedValueOnce('Parse error');

		const hook = await renderMetadataSyncHook();

		await expect(hook.syncComicInfo('789')).rejects.toEqual('Parse error');

		expect(errorMock).toHaveBeenCalledWith('Failed to sync ComicInfo.xml: "Parse error"');
		expect(hook.isSyncing).toBe(false);
	});

	it('clears comic metadata successfully', async () => {
		invokeMock.mockResolvedValueOnce(undefined);

		const hook = await renderMetadataSyncHook();
		await hook.clearMetadata('789');

		expect(invokeMock).toHaveBeenCalledWith(METADATA_COMMANDS.clearComicMetadata, {
			comicId: '789'
		});
		expect(hook.isSyncing).toBe(false);
	});

	it('is syncing while clearMetadata is in flight', async () => {
		let resolveSync: (value: unknown) => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = resolve;
				})
		);
		const hook = await renderMetadataSyncHook();

		const pending = hook.clearMetadata('789');
		await Promise.resolve();
		expect(hook.isSyncing).toBe(true);

		resolveSync(undefined);
		await pending;
		expect(hook.isSyncing).toBe(false);
	});

	it('handles error when clearing comic metadata', async () => {
		invokeMock.mockRejectedValueOnce('Delete failed');

		const hook = await renderMetadataSyncHook();

		await expect(hook.clearMetadata('789')).rejects.toEqual('Delete failed');

		expect(errorMock).toHaveBeenCalledWith('Failed to clear comic metadata: "Delete failed"');
		expect(hook.isSyncing).toBe(false);
	});
});
