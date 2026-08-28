import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import type { ComicSummary } from '$lib/contracts/network/network.payloads';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useRemoteLibrary } from './use-remote-library.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(),
	convertFileSrc: vi.fn((path: string) => `asset://${path}`)
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);

async function renderHook() {
	let hook: ReturnType<typeof useRemoteLibrary> | undefined;

	render(HookHarness, {
		props: {
			create: () => useRemoteLibrary(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useRemoteLibrary>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

function setupListeners() {
	const callbacks = new Map<string, (event: { payload: unknown }) => void>();
	const unlisteners = new Map<string, ReturnType<typeof vi.fn>>();

	listenMock.mockImplementation((event, callback) => {
		callbacks.set(String(event), callback as (event: { payload: unknown }) => void);
		const unlisten = vi.fn();
		unlisteners.set(String(event), unlisten);
		return Promise.resolve(unlisten);
	});

	return { callbacks, unlisteners };
}

function comic(overrides: Partial<ComicSummary> = {}): ComicSummary {
	return { comicName: 'One Piece', chapterCount: 10, coverVersion: 1, ...overrides };
}

describe('useRemoteLibrary', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		invokeMock.mockResolvedValue(undefined);
	});

	it('registers listeners for library and cover query events', async () => {
		const { unlisteners } = setupListeners();
		const hook = await renderHook();

		await hook.startListening();

		expect(unlisteners.size).toBe(4);
	});

	it('queryRemoteLibrary marks the peer as loading and calls the backend', async () => {
		setupListeners();
		const hook = await renderHook();

		const pending = hook.queryRemoteLibrary('peer-1', [1, 2]);
		expect(hook.isLoading('peer-1')).toBe(true);
		await pending;

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.queryRemoteLibrary, {
			peerId: 'peer-1',
			addrs: [1, 2]
		});
	});

	it('sets the error and rethrows when queryRemoteLibrary fails', async () => {
		setupListeners();
		invokeMock.mockRejectedValueOnce(new Error('peer offline'));
		const hook = await renderHook();

		await expect(hook.queryRemoteLibrary('peer-2', [])).rejects.toThrow('peer offline');

		expect(hook.isLoading('peer-2')).toBe(false);
		expect(hook.errorFor('peer-2')).toBe('Error: peer offline');
	});

	it('libraryQueryResult stores comics for the peer and clears loading/errors', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-3', [1]);

		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({ peerId: 'peer-3', comics: [comic()] })
		});
		await Promise.resolve();

		expect(hook.comicsFor('peer-3')).toEqual([comic()]);
		expect(hook.isLoading('peer-3')).toBe(false);
		expect(hook.errorFor('peer-3')).toBeUndefined();
	});

	it('automatically queries covers for comics received in the library result', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-4', [9, 9]);
		invokeMock.mockClear();

		callbacks.get(NETWORK_EVENTS.libraryQueryResult)?.({
			payload: JSON.stringify({ peerId: 'peer-4', comics: [comic({ comicName: 'Naruto' })] })
		});
		await Promise.resolve();
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.queryRemoteCover, {
			peerId: 'peer-4',
			addrs: [9, 9],
			comicName: 'Naruto',
			knownVersion: null
		});
	});

	it('libraryQueryError sets the error message and clears loading for that peer', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		await hook.queryRemoteLibrary('peer-5', []);

		callbacks.get(NETWORK_EVENTS.libraryQueryError)?.({
			payload: JSON.stringify({ peerId: 'peer-5', message: 'timed out' })
		});

		expect(hook.isLoading('peer-5')).toBe(false);
		expect(hook.errorFor('peer-5')).toBe('timed out');
	});

	it('coverQueryResult resolves and stores the cover path when status is changed', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-6',
				comicName: 'Bleach',
				status: 'changed',
				coverVersion: 3,
				path: 'C:\\covers\\bleach.jpg'
			})
		});

		expect(hook.coverPathFor('peer-6', 'Bleach')).toBe('asset://C:/covers/bleach.jpg');
	});

	it('coverQueryResult does not store a path when status is not_modified', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.coverQueryResult)?.({
			payload: JSON.stringify({
				peerId: 'peer-7',
				comicName: 'Bleach',
				status: 'not_modified',
				coverVersion: 3,
				path: null
			})
		});

		expect(hook.coverPathFor('peer-7', 'Bleach')).toBeUndefined();
	});

	it('coverQueryError is a silent no-op', async () => {
		const { callbacks } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		expect(() =>
			callbacks.get(NETWORK_EVENTS.coverQueryError)?.({ payload: 'ignored' })
		).not.toThrow();
	});

	it('comicsFor/errorFor/coverPathFor default to empty/undefined for unknown peers', async () => {
		const hook = await renderHook();

		expect(hook.comicsFor('unknown')).toEqual([]);
		expect(hook.errorFor('unknown')).toBeUndefined();
		expect(hook.coverPathFor('unknown', 'x')).toBeUndefined();
		expect(hook.isLoading('unknown')).toBe(false);
	});

	it('stopListening unregisters every listener', async () => {
		const { unlisteners } = setupListeners();
		const hook = await renderHook();
		await hook.startListening();

		hook.stopListening();

		for (const unlisten of unlisteners.values()) {
			expect(unlisten).toHaveBeenCalledOnce();
		}
	});
});
