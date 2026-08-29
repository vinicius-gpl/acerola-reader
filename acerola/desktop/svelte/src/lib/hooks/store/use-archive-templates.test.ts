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

	it('starts with an empty template list, not loading, and not initialized', async () => {
		// O state é um singleton a nível de módulo, resetado pra valores conhecidos por
		// `_resetArchiveTemplatesState()` no `beforeEach`, o que esconderia os valores
		// iniciais de verdade. Forçamos uma instância nova do módulo (e um mock de `invoke`
		// atrelado a ela) pra observar os valores como estão declarados no source, antes de
		// qualquer reset ou load rodar.
		vi.resetModules();
		const freshCore = await import('@tauri-apps/api/core');
		const freshInvoke = vi.mocked(freshCore.invoke);
		const freshHookModule = await import('./use-archive-templates.svelte');

		let hook: ReturnType<typeof freshHookModule.useArchiveTemplates> | undefined;
		render(HookHarness, {
			props: {
				create: () => freshHookModule.useArchiveTemplates(),
				onReady: (value) => {
					hook = value as ReturnType<typeof freshHookModule.useArchiveTemplates>;
				}
			}
		});
		await tick();

		expect(hook!.templates).toEqual([]);
		expect(hook!.isLoading).toBe(false);

		// Prova que `isInitialized` começa `false` de verdade: se começasse `true`,
		// `loadTemplates()` sairia mais cedo e nunca chamaria o backend.
		freshInvoke.mockResolvedValueOnce([template()]);
		await hook!.loadTemplates();

		expect(freshInvoke).toHaveBeenCalledWith(ARCHIVE_TEMPLATE_COMMANDS.getArchiveTemplates);
		expect(hook!.templates).toEqual([template()]);
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

	it('is loading while the request is in flight, then settles back to false', async () => {
		let resolveGet: (value: ArchiveTemplate[]) => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveGet = resolve;
				})
		);
		const hook = await renderHook();

		const pending = hook.loadTemplates();
		await Promise.resolve();
		expect(hook.isLoading).toBe(true);

		resolveGet([]);
		await pending;
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

	it('deletes a template and removes only that one from the list', async () => {
		const kept = template({ id: 4, label: 'Kept' });
		const existing = template({ id: 5 });
		invokeMock.mockResolvedValueOnce([kept, existing]);
		const hook = await renderHook();
		await hook.loadTemplates();

		invokeMock.mockResolvedValueOnce(undefined);
		await hook.deleteTemplate(5);

		expect(invokeMock).toHaveBeenCalledWith(ARCHIVE_TEMPLATE_COMMANDS.deleteArchiveTemplate, {
			id: 5
		});
		expect(hook.templates).toEqual([kept]);
	});

	it('logs and rethrows when loading templates fails', async () => {
		const failure = new Error('backend unavailable');
		invokeMock.mockRejectedValueOnce(failure);
		const hook = await renderHook();

		await expect(hook.loadTemplates()).rejects.toThrow(failure);

		expect(errorMock).toHaveBeenCalledWith(
			expect.stringContaining('Failed to load archive templates: Error: backend unavailable')
		);
		expect(hook.isLoading).toBe(false);
		expect(hook.templates).toEqual([]);
	});

	it('logs and rethrows when creating a template fails', async () => {
		const hook = await renderHook();
		const failure = new Error('duplicate label');
		invokeMock.mockRejectedValueOnce(failure);

		await expect(hook.createTemplate('Dup', '{title}', 'Volume')).rejects.toThrow(failure);

		expect(errorMock).toHaveBeenCalledWith(
			expect.stringContaining('Failed to create archive template: Error: duplicate label')
		);
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

		expect(errorMock).toHaveBeenCalledWith(
			expect.stringContaining('Failed to delete archive template: Error: template in use')
		);
		expect(hook.templates).toEqual([existing]);
	});
});
