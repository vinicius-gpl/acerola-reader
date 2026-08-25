import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
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
});
