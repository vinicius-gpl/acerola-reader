import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import AcerolaNetworkTransfersLog from './acerola-network-transfers-log.svelte';
import type { TransferLogEntry } from '$lib/hooks/store/use-network-sync.svelte';

function entry(overrides: Partial<TransferLogEntry> = {}): TransferLogEntry {
	return {
		id: 1,
		peerId: 'peer-1',
		kind: 'history',
		status: 'started',
		message: 'peer-1',
		timestamp: Date.now(),
		...overrides
	};
}

describe('AcerolaNetworkTransfersLog', () => {
	it('shows the empty state when there are no entries', () => {
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: { entries: [], peerLabel: (id: string) => id }
			}
		});

		expect(screen.getByText(/No transfers|Nenhuma transfer/i)).toBeInTheDocument();
	});

	it('resolves the peer label for a "started" entry', () => {
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: {
					entries: [entry({ kind: 'history', status: 'started', message: 'peer-1' })],
					peerLabel: () => 'Meu Notebook'
				}
			}
		});

		expect(screen.getByText(/Meu Notebook/)).toBeInTheDocument();
	});

	it('renders the raw message for a "progress" entry (no peer label lookup)', () => {
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: {
					entries: [entry({ kind: 'files', status: 'progress', message: 'chapter1.cbz' })],
					peerLabel: () => 'unused'
				}
			}
		});

		expect(screen.getByText(/chapter1\.cbz/)).toBeInTheDocument();
	});

	it('falls back to the raw message when a kind has no formatter for that status', () => {
		// 'history' não tem status 'progress' definido — describeEntry deve cair no fallback.
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: {
					entries: [entry({ kind: 'history', status: 'progress', message: 'raw-fallback-text' })],
					peerLabel: () => 'unused'
				}
			}
		});

		expect(screen.getByText('raw-fallback-text')).toBeInTheDocument();
	});

	it('appends the conflicts suffix to a "complete" entry that reports real conflicts', () => {
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: {
					entries: [
						entry({ kind: 'files', status: 'complete', message: 'peer-1', conflicts: 2 })
					],
					peerLabel: () => 'Meu Notebook'
				}
			}
		});

		expect(screen.getByText(/2 conflict|2 conflito/i)).toBeInTheDocument();
	});

	it('does not append the conflicts suffix when there were none', () => {
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: {
					entries: [entry({ kind: 'files', status: 'complete', message: 'peer-1' })],
					peerLabel: () => 'Meu Notebook'
				}
			}
		});

		expect(screen.queryByText(/conflict|conflito/i)).not.toBeInTheDocument();
	});

	it('calls onRefresh when the refresh button is clicked', async () => {
		const user = userEvent.setup();
		const onRefresh = vi.fn();
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: { entries: [], peerLabel: (id: string) => id },
				events: { onRefresh, onClear: vi.fn() }
			}
		});

		await user.click(screen.getByRole('button', { name: /Refresh|Atualizar/i }));

		expect(onRefresh).toHaveBeenCalled();
	});

	it('hides the clear button when there are no entries', () => {
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: { entries: [], peerLabel: (id: string) => id },
				events: { onRefresh: vi.fn(), onClear: vi.fn() }
			}
		});

		expect(screen.queryByRole('button', { name: /Clear|Limpar/i })).not.toBeInTheDocument();
	});

	it('calls onClear only after confirming the destructive dialog', async () => {
		const user = userEvent.setup();
		const onClear = vi.fn();
		render(AcerolaNetworkTransfersLog, {
			props: {
				data: {
					entries: [entry()],
					peerLabel: () => 'Meu Notebook'
				},
				events: { onRefresh: vi.fn(), onClear }
			}
		});

		// bits-ui AlertDialog.Trigger envolve o elemento filho em seu próprio <button>, então dois
		// botões com o mesmo nome acessível existem — o gatilho externo é o índice 0 (mesmo padrão
		// de `routes/history/__tests__/history.test.ts`).
		const clearButtons = screen.getAllByRole('button', { name: /Clear|Limpar/i });
		await user.click(clearButtons[0]);

		expect(onClear).not.toHaveBeenCalled();

		const confirmButton = await screen.findByRole('button', {
			name: /Yes, clear everything|Sim, limpar tudo/i
		});
		await user.click(confirmButton);

		expect(onClear).toHaveBeenCalled();
	});
});
