import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it } from 'vitest';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { _resetChapterSelectionState, useChapterSelection } from './use-chapter-selection.svelte';

async function renderHook() {
	let hook: ReturnType<typeof useChapterSelection> | undefined;

	render(HookHarness, {
		props: {
			create: () => useChapterSelection(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useChapterSelection>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

describe('useChapterSelection', () => {
	beforeEach(() => {
		_resetChapterSelectionState();
	});

	it('starts with no selection and not in selection mode', async () => {
		const hook = await renderHook();

		expect(hook.selectedIds.size).toBe(0);
		expect(hook.selectedCount).toBe(0);
		expect(hook.isSelectionMode).toBe(false);
		expect(hook.selectedIdsArray).toEqual([]);
	});

	it('toggles selection on and off, entering/exiting selection mode', async () => {
		const hook = await renderHook();

		hook.toggleSelection('ch-1');
		expect(hook.isSelected('ch-1')).toBe(true);
		expect(hook.isSelectionMode).toBe(true);
		expect(hook.selectedCount).toBe(1);

		hook.toggleSelection('ch-1');
		expect(hook.isSelected('ch-1')).toBe(false);
		expect(hook.isSelectionMode).toBe(false);
		expect(hook.selectedCount).toBe(0);
	});

	it('selectSingle replaces the whole selection with a single id', async () => {
		const hook = await renderHook();

		hook.selectAll(['ch-1', 'ch-2']);
		hook.selectSingle('ch-3');

		expect(hook.selectedIdsArray).toEqual(['ch-3']);
		expect(hook.isSelectionMode).toBe(true);
	});

	it('selectAll replaces the selection and exits selection mode for an empty list', async () => {
		const hook = await renderHook();

		hook.selectAll(['ch-1', 'ch-2', 'ch-3']);
		expect(hook.selectedCount).toBe(3);
		expect(hook.isSelectionMode).toBe(true);

		hook.selectAll([]);
		expect(hook.selectedCount).toBe(0);
		expect(hook.isSelectionMode).toBe(false);
	});

	it('deselectAll clears selection and exits selection mode', async () => {
		const hook = await renderHook();

		hook.selectAll(['ch-1']);
		hook.deselectAll();

		expect(hook.selectedIdsArray).toEqual([]);
		expect(hook.isSelectionMode).toBe(false);
	});

	it('enterSelectionMode turns on selection mode without altering selection', async () => {
		const hook = await renderHook();

		hook.enterSelectionMode();

		expect(hook.isSelectionMode).toBe(true);
		expect(hook.selectedIdsArray).toEqual([]);
	});

	it('exitSelectionMode clears both selection and selection mode', async () => {
		const hook = await renderHook();

		hook.selectAll(['ch-1', 'ch-2']);
		hook.exitSelectionMode();

		expect(hook.isSelectionMode).toBe(false);
		expect(hook.selectedIdsArray).toEqual([]);
	});

	it('isSelected returns false for an id that was never selected', async () => {
		const hook = await renderHook();

		expect(hook.isSelected('never-selected')).toBe(false);
	});
});
