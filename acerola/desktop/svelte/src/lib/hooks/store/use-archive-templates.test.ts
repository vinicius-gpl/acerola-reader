import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { error as tauriError } from '@tauri-apps/plugin-log';
import { ARCHIVE_TEMPLATE_COMMANDS } from '$lib/contracts/archive/archive-template.commands';
import type { ArchiveTemplate } from '$lib/contracts/archive/archive-template.payloads';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { _resetArchiveTemplatesState, useArchiveTemplates } from './use-archive-templates.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const errorMock = vi.mocked(tauriError);

async function renderHook() {
	let hook: ReturnType<typeof useArchiveTemplates> | undefined;

	render(HookHarness, {
		props: {
			create: () => useArchiveTemplates(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useArchiveTemplates>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

function template(overrides: Partial<ArchiveTemplate> = {}): ArchiveTemplate {
	return {
		id: 1,
		label: 'Default',
		pattern: '{title} #{number}',
		sort_type: 'Chapter',
		is_default: true,
		priority: 0,
		...overrides
	};
}

describe('useArchiveTemplates', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		_resetArchiveTemplatesState();
	});

	it('loads templates from the backend once', async () => {
		const templates = [template()];
		invokeMock.mockResolvedValueOnce(templates);

		const hook = await renderHook();

		expect(hook.isLoading).toBe(false);
		await hook.loadTemplates();

		expect(invokeMock).toHaveBeenCalledWith(ARCHIVE_TEMPLATE_COMMANDS.getArchiveTemplates);
		expect(hook.templates).toEqual(templates);
		expect(hook.isLoading).toBe(false);
	});

	it('does not request again once already initialized', async () => {
		invokeMock.mockResolvedValueOnce([template()]);
		const hook = await renderHook();

		await hook.loadTemplates();
		await hook.loadTemplates();

		expect(invokeMock).toHaveBeenCalledTimes(1);
	});

	it('creates a template and appends it to the list', async () => {
		invokeMock.mockResolvedValueOnce([]);
		const hook = await renderHook();
		await hook.loadTemplates();

		const created = template({ id: 2, label: 'Custom', is_default: false });
		invokeMock.mockResolvedValueOnce(created);

		const result = await hook.createTemplate('Custom', '{title}', 'Chapter');

		expect(invokeMock).toHaveBeenCalledWith(ARCHIVE_TEMPLATE_COMMANDS.createArchiveTemplate, {
			label: 'Custom',
			pattern: '{title}',
			sortType: 'Chapter'
		});
		expect(result).toEqual(created);
		expect(hook.templates).toEqual([created]);
	});

	it('deletes a template and removes it from the list', async () => {
		const existing = template({ id: 5 });
		invokeMock.mockResolvedValueOnce([existing]);
		const hook = await renderHook();
		await hook.loadTemplates();

		invokeMock.mockResolvedValueOnce(undefined);
		await hook.deleteTemplate(5);

		expect(invokeMock).toHaveBeenCalledWith(ARCHIVE_TEMPLATE_COMMANDS.deleteArchiveTemplate, {
			id: 5
		});
		expect(hook.templates).toEqual([]);
	});

	it('logs and rethrows when loading templates fails', async () => {
		const failure = new Error('backend unavailable');
		invokeMock.mockRejectedValueOnce(failure);
		const hook = await renderHook();

		await expect(hook.loadTemplates()).rejects.toThrow(failure);

		expect(errorMock).toHaveBeenCalled();
		expect(hook.isLoading).toBe(false);
		expect(hook.templates).toEqual([]);
	});

	it('logs and rethrows when creating a template fails', async () => {
		const hook = await renderHook();
		const failure = new Error('duplicate label');
		invokeMock.mockRejectedValueOnce(failure);

		await expect(hook.createTemplate('Dup', '{title}', 'Volume')).rejects.toThrow(failure);

		expect(errorMock).toHaveBeenCalled();
		expect(hook.templates).toEqual([]);
	});

	it('logs and rethrows when deleting a template fails', async () => {
		const existing = template({ id: 9 });
		invokeMock.mockResolvedValueOnce([existing]);
		const hook = await renderHook();
		await hook.loadTemplates();

		const failure = new Error('template in use');
		invokeMock.mockRejectedValueOnce(failure);

		await expect(hook.deleteTemplate(9)).rejects.toThrow(failure);

		expect(errorMock).toHaveBeenCalled();
		expect(hook.templates).toEqual([existing]);
	});
});
