import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { notificationStore } from '$lib/components/acerola-notification/acerola-notification.svelte';
import { HISTORY_COMMANDS } from '$lib/contracts/history/history.commands';
import type { ReadingHistoryPayload } from '$lib/contracts/history/history.payloads';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useHistory } from './use-history.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('svelte-sonner', () => ({
	toast: {
		error: vi.fn()
	}
}));

const invokeMock = vi.mocked(invoke);

async function renderHook() {
	let hook: ReturnType<typeof useHistory> | undefined;

	render(HookHarness, {
		props: {
			create: () => useHistory(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useHistory>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

function historyItem(overrides: Partial<ReadingHistoryPayload> = {}): ReadingHistoryPayload {
	return {
		comicDirectoryId: '1',
		chapterArchiveId: '10',
		lastPage: 5,
		isCompleted: false,
		updatedAt: 1_600_000_000,
		comicName: 'Comic 1',
		comicCover: null,
		chapterName: 'Chapter 1',
		folderName: 'Comic 1',
		chapterPath: '/path/to/chapter1.cbz',
		chapterSort: '1',
		isSpecial: false,
		lastModified: 0,
		...overrides
	};
}

describe('useHistory', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		notificationStore.clearAll();
	});

	it('fetches history items successfully', async () => {
		const items = [historyItem()];
		invokeMock.mockResolvedValueOnce(items);

		const hook = await renderHook();
		expect(hook.loading).toBe(false);

		await hook.fetch();

		expect(invokeMock).toHaveBeenCalledWith(HISTORY_COMMANDS.getAll);
		expect(hook.items).toEqual(items);
		expect(hook.loading).toBe(false);
	});

	it('ignores a fetch call while one is already in flight', async () => {
		let resolveFirst: (value: ReadingHistoryPayload[]) => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveFirst = resolve;
				})
		);

		const hook = await renderHook();

		const first = hook.fetch();
		const second = hook.fetch();

		expect(invokeMock).toHaveBeenCalledTimes(1);

		resolveFirst([]);
		await Promise.all([first, second]);
	});

	it('notifies and toasts when fetch fails', async () => {
		invokeMock.mockRejectedValueOnce('backend offline');

		const hook = await renderHook();
		await hook.fetch();

		expect(hook.loading).toBe(false);
		expect(toast.error).toHaveBeenCalledWith('backend offline');
		expect(notificationStore.notifications[0]?.message).toBe('Erro ao carregar histórico');
	});

	it('clears history successfully', async () => {
		invokeMock.mockResolvedValueOnce([historyItem()]);
		const hook = await renderHook();
		await hook.fetch();

		invokeMock.mockResolvedValueOnce(undefined);
		await hook.clear();

		expect(invokeMock).toHaveBeenCalledWith(HISTORY_COMMANDS.clear);
		expect(hook.items).toEqual([]);
	});

	it('notifies and toasts when clear fails', async () => {
		invokeMock.mockRejectedValueOnce('cannot clear');
		const hook = await renderHook();

		await hook.clear();

		expect(toast.error).toHaveBeenCalledWith('cannot clear');
		expect(notificationStore.notifications[0]?.message).toBe('Erro ao limpar histórico');
	});

	it('updates reading progress', async () => {
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();

		await hook.updateReading('comic-1', 'chapter-1', 12, false);

		expect(invokeMock).toHaveBeenCalledWith(HISTORY_COMMANDS.updateReading, {
			comicId: 'comic-1',
			chapterId: 'chapter-1',
			lastPage: 12,
			isCompleted: false
		});
	});

	it('logs but does not throw when updateReading fails', async () => {
		invokeMock.mockRejectedValueOnce(new Error('network error'));
		const hook = await renderHook();

		await expect(hook.updateReading('comic-1', 'chapter-1', 1, false)).resolves.toBeUndefined();
	});

	it('marks a chapter as read', async () => {
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();

		await hook.markChapterRead('comic-1', 'chapter-1');

		expect(invokeMock).toHaveBeenCalledWith(HISTORY_COMMANDS.markChapterRead, {
			comicId: 'comic-1',
			chapterId: 'chapter-1'
		});
	});

	it('notifies, toasts and rethrows when markChapterRead fails', async () => {
		invokeMock.mockRejectedValueOnce('failed to mark');
		const hook = await renderHook();

		await expect(hook.markChapterRead('comic-1', 'chapter-1')).rejects.toBe('failed to mark');
		expect(toast.error).toHaveBeenCalledWith('failed to mark');
	});

	it('unmarks a chapter as read', async () => {
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();

		await hook.unmarkChapterRead('comic-1', 'chapter-1');

		expect(invokeMock).toHaveBeenCalledWith(HISTORY_COMMANDS.unmarkChapterRead, {
			comicId: 'comic-1',
			chapterId: 'chapter-1'
		});
	});

	it('notifies, toasts and rethrows when unmarkChapterRead fails', async () => {
		invokeMock.mockRejectedValueOnce('failed to unmark');
		const hook = await renderHook();

		await expect(hook.unmarkChapterRead('comic-1', 'chapter-1')).rejects.toBe(
			'failed to unmark'
		);
		expect(toast.error).toHaveBeenCalledWith('failed to unmark');
	});

	it('marks chapters read in batch and returns the affected count', async () => {
		invokeMock.mockResolvedValueOnce(3);
		const hook = await renderHook();

		const count = await hook.markChaptersReadBatch('comic-1', ['ch-1', 'ch-2', 'ch-3']);

		expect(invokeMock).toHaveBeenCalledWith(HISTORY_COMMANDS.markChaptersReadBatch, {
			comicId: 'comic-1',
			chapterIds: ['ch-1', 'ch-2', 'ch-3']
		});
		expect(count).toBe(3);
	});

	it('notifies, toasts and rethrows when markChaptersReadBatch fails', async () => {
		invokeMock.mockRejectedValueOnce('batch failed');
		const hook = await renderHook();

		await expect(hook.markChaptersReadBatch('comic-1', ['ch-1'])).rejects.toBe('batch failed');
		expect(toast.error).toHaveBeenCalledWith('batch failed');
	});

	it('unmarks chapters read in batch and returns the affected count', async () => {
		invokeMock.mockResolvedValueOnce(2);
		const hook = await renderHook();

		const count = await hook.unmarkChaptersReadBatch('comic-1', ['ch-1', 'ch-2']);

		expect(invokeMock).toHaveBeenCalledWith(HISTORY_COMMANDS.unmarkChaptersReadBatch, {
			comicId: 'comic-1',
			chapterIds: ['ch-1', 'ch-2']
		});
		expect(count).toBe(2);
	});

	it('notifies, toasts and rethrows when unmarkChaptersReadBatch fails', async () => {
		invokeMock.mockRejectedValueOnce('batch unmark failed');
		const hook = await renderHook();

		await expect(hook.unmarkChaptersReadBatch('comic-1', ['ch-1'])).rejects.toBe(
			'batch unmark failed'
		);
		expect(toast.error).toHaveBeenCalledWith('batch unmark failed');
	});
});
