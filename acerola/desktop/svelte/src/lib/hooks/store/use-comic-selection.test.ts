import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it } from 'vitest';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { _resetComicSelectionState, useComicSelection } from './use-comic-selection.svelte';

async function renderHook() {
	let hook: ReturnType<typeof useComicSelection> | undefined;

	render(HookHarness, {
		props: {
			create: () => useComicSelection(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useComicSelection>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

describe('useComicSelection', () => {
	beforeEach(() => {
		_resetComicSelectionState();
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

		hook.toggleSelection(1);
		expect(hook.isSelected(1)).toBe(true);
		expect(hook.isSelectionMode).toBe(true);

		hook.toggleSelection(1);
		expect(hook.isSelected(1)).toBe(false);
		expect(hook.isSelectionMode).toBe(false);
	});

	it('supports mixed string and numeric ids', async () => {
		const hook = await renderHook();

		hook.selectAll(['comic-a', 2, 'comic-b']);

		expect(hook.selectedCount).toBe(3);
		expect(hook.isSelected('comic-a')).toBe(true);
		expect(hook.isSelected(2)).toBe(true);
	});

	it('selectSingle replaces the whole selection', async () => {
		const hook = await renderHook();

		hook.selectAll([1, 2, 3]);
		hook.selectSingle(9);

		expect(hook.selectedIdsArray).toEqual([9]);
	});

	it('selectAll with empty list exits selection mode', async () => {
		const hook = await renderHook();

		hook.selectAll([1, 2]);
		hook.selectAll([]);

		expect(hook.selectedCount).toBe(0);
		expect(hook.isSelectionMode).toBe(false);
	});

	it('deselectAll clears selection and exits selection mode', async () => {
		const hook = await renderHook();

		hook.selectAll([1]);
		hook.deselectAll();

		expect(hook.selectedIdsArray).toEqual([]);
		expect(hook.isSelectionMode).toBe(false);
	});

	it('enterSelectionMode/exitSelectionMode toggle mode independently of content', async () => {
		const hook = await renderHook();

		hook.enterSelectionMode();
		expect(hook.isSelectionMode).toBe(true);

		hook.exitSelectionMode();
		expect(hook.isSelectionMode).toBe(false);
		expect(hook.selectedIdsArray).toEqual([]);
	});

	it('isSelected returns false for an id that was never selected', async () => {
		const hook = await renderHook();

		expect(hook.isSelected('missing')).toBe(false);
	});
});
