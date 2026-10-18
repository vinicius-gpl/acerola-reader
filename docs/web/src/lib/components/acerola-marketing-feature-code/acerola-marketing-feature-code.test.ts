import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaMarketingFeatureCode from './acerola-marketing-feature-code.svelte';

describe('AcerolaMarketingFeatureCode', () => {
	it('renders title, features, and tab content correctly', () => {
		render(AcerolaMarketingFeatureCode, {
			props: {
				title: 'P2P Architecture',
				description: 'Zero cloud reliance with direct Iroh synchronization.',
				features: [
					{
						title: 'Direct QUIC Stream',
						description: 'High throughput binary streams between paired devices.'
					}
				],
				tabs: [
					{
						id: 'rust',
						label: 'Rust',
						code: 'fn sync() {}'
					}
				]
			}
		});

		expect(screen.getByText('P2P Architecture')).toBeInTheDocument();
		expect(screen.getByText('Direct QUIC Stream')).toBeInTheDocument();
		expect(screen.getByText('fn sync() {}')).toBeInTheDocument();
	});

	it('copies active code tab on click', async () => {
		const user = userEvent.setup();
		if (!navigator.clipboard) {
			Object.defineProperty(navigator, 'clipboard', {
				value: { writeText: vi.fn() },
				writable: true
			});
		}
		const writeTextMock = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue(undefined);

		render(AcerolaMarketingFeatureCode, {
			props: {
				title: 'Copy Test',
				features: [],
				tabs: [
					{
						id: 'caddy',
						label: 'Caddyfile',
						code: 'relay.acerola.app { reverse_proxy :80 }'
					}
				]
			}
		});

		const copyBtn = screen.getByRole('button', { name: /copiar/i });
		await user.click(copyBtn);

		expect(writeTextMock).toHaveBeenCalledWith('relay.acerola.app { reverse_proxy :80 }');
	});
});
