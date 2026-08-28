import { render, screen, fireEvent } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ARCHIVE_TEMPLATE_COMMANDS } from '$lib/contracts/archive/archive-template.commands';
import { _resetArchiveTemplatesState } from '$lib/hooks/store/use-archive-templates.svelte';
import type { ArchiveTemplate } from '$lib/contracts/archive/archive-template.payloads';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

vi.mock('svelte-sonner', () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

import TemplatesPage from './+page.svelte';
import { toast } from 'svelte-sonner';

const invokeMock = vi.mocked(invoke);

/** The delete trigger is an icon-only button (no accessible name) — locate it by
 *  scoping to the template row that contains the given label instead. */
function deleteButtonForRow(label: string) {
	const row = screen.getByText(label).closest('div.flex.items-center.justify-between');
	const button = row?.querySelector('button');
	if (!button) throw new Error(`could not find the delete button for row "${label}"`);
	return button;
}

const defaultChapterTemplate: ArchiveTemplate = {
	id: 1,
	label: 'Base Capítulo',
	pattern: 'Ch. {chapter}{decimal}.*.{extension}',
	sort_type: 'Chapter',
	is_default: true,
	priority: 0
};

const customVolumeTemplate: ArchiveTemplate = {
	id: 2,
	label: 'Meu Volume',
	pattern: 'Vol. {volume}{decimal}',
	sort_type: 'Volume',
	is_default: false,
	priority: 1
};

describe('config/templates +page', () => {
	beforeEach(() => {
		// resetAllMocks (not clearAllMocks) also drops any queued mockResolvedValueOnce
		// values — important so a mid-test failure in one test can't leak an unconsumed
		// queued response into the next test's invoke() call.
		vi.resetAllMocks();
		_resetArchiveTemplatesState();
	});

	it('loads and renders templates grouped by sort type on mount', async () => {
		invokeMock.mockResolvedValueOnce([defaultChapterTemplate, customVolumeTemplate]);

		render(TemplatesPage);

		expect(await screen.findByText('Base Capítulo')).toBeInTheDocument();
		expect(screen.getByText('Meu Volume')).toBeInTheDocument();
		expect(invokeMock).toHaveBeenCalledWith(ARCHIVE_TEMPLATE_COMMANDS.getArchiveTemplates);
	});

	it('shows an error toast when loading templates fails', async () => {
		invokeMock.mockRejectedValueOnce(new Error('boom'));

		render(TemplatesPage);

		await vi.waitFor(() => expect(toast.error).toHaveBeenCalled());
	});

	it('does not render a delete button for default templates', async () => {
		invokeMock.mockResolvedValueOnce([defaultChapterTemplate]);

		render(TemplatesPage);

		const row = (await screen.findByText('Base Capítulo')).closest(
			'div.flex.items-center.justify-between'
		);
		expect(row?.querySelector('button')).toBeNull();
	});

	it('creates a new chapter template with the auto-appended extension macro', async () => {
		invokeMock.mockResolvedValueOnce([]);
		const created: ArchiveTemplate = {
			id: 3,
			label: 'Scanlator X',
			pattern: 'Ch. {chapter}{decimal}{extension}',
			sort_type: 'Chapter',
			is_default: false,
			priority: 2
		};
		invokeMock.mockResolvedValueOnce(created);

		const user = userEvent.setup();
		render(TemplatesPage);

		await screen.findByText(/how to build a template|como montar um template/i);

		const labelInput = screen.getByLabelText(/template name|nome do template/i);
		await user.type(labelInput, 'Scanlator X');

		const patternInput = screen.getByLabelText(/^pattern$|^padrão$/i);
		await user.type(patternInput, 'Ch. ');
		await user.click(screen.getByRole('button', { name: '+ {chapter}' }));
		await user.click(screen.getByRole('button', { name: '+ {decimal}' }));

		const createButton = screen.getByRole('button', { name: /create template|criar template/i });
		await user.click(createButton);

		expect(invokeMock).toHaveBeenCalledWith(
			ARCHIVE_TEMPLATE_COMMANDS.createArchiveTemplate,
			expect.objectContaining({
				label: 'Scanlator X',
				pattern: 'Ch. {chapter}{decimal}{extension}',
				sortType: 'Chapter'
			})
		);
		expect(await screen.findByText('Scanlator X')).toBeInTheDocument();
	});

	it('deletes a custom template after confirming the alert dialog', async () => {
		invokeMock.mockResolvedValueOnce([customVolumeTemplate]);
		invokeMock.mockResolvedValueOnce(undefined);

		const user = userEvent.setup();
		render(TemplatesPage);

		await screen.findByText('Meu Volume');

		await user.click(deleteButtonForRow('Meu Volume'));

		const confirmButton = screen.getByRole('button', { name: /^remover$|^remove$/i });
		await user.click(confirmButton);

		expect(invokeMock).toHaveBeenCalledWith(ARCHIVE_TEMPLATE_COMMANDS.deleteArchiveTemplate, {
			id: 2
		});
		await vi.waitFor(() => expect(screen.queryByText('Meu Volume')).not.toBeInTheDocument());
	});

	it('shows an empty state when there are no custom templates for a group', async () => {
		invokeMock.mockResolvedValueOnce([defaultChapterTemplate]);

		render(TemplatesPage);

		await screen.findByText('Base Capítulo');

		expect(
			screen.getByText(/no custom volume templates yet|nenhum template de volume/i)
		).toBeInTheDocument();
	});

	it('does not send create request when required fields are empty', async () => {
		invokeMock.mockResolvedValueOnce([]);

		render(TemplatesPage);
		await screen.findByText(/how to build a template|como montar um template/i);

		const createButton = screen.getByRole('button', { name: /create template|criar template/i });
		await fireEvent.click(createButton);

		expect(invokeMock).toHaveBeenCalledTimes(1); // only the initial getArchiveTemplates call
	});
});
