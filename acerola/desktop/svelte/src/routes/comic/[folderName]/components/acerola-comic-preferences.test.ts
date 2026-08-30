import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaComicPreferences from './acerola-comic-preferences.svelte';

describe('AcerolaComicPreferences', () => {
	function defaultProps() {
		return {
			state: {
				volumeViewMode: 'cover' as const,
				bookmarkId: null,
				externalSyncEnabled: true
			},
			events: {
				onVolumeViewModeChange: vi.fn(),
				onBookmarkChange: vi.fn(),
				onExternalSyncChange: vi.fn()
			}
		};
	}

	it('renders preference options', () => {
		render(AcerolaComicPreferences, { props: defaultProps() });

		expect(screen.getByText('Leitura')).toBeInTheDocument();
	});

	it('hides volume preference when there is no volume structure', () => {
		render(AcerolaComicPreferences, { props: defaultProps() });

		expect(screen.queryByText('Destaque do Volume')).not.toBeInTheDocument();
		expect(screen.queryByRole('radio', { name: 'Capa' })).not.toBeInTheDocument();
	});

	it('displays volume preference when volume structure exists', () => {
		render(AcerolaComicPreferences, {
			props: {
				...defaultProps(),
				data: { hasVolumeStructure: true }
			}
		});

		expect(screen.getByText('Destaque do Volume')).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: 'Capa' })).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: 'Banner' })).toBeInTheDocument();
	});

	it('changes volume highlight when clicking banner', async () => {
		const user = userEvent.setup();
		const props = defaultProps();
		render(AcerolaComicPreferences, {
			props: {
				...props,
				data: { hasVolumeStructure: true }
			}
		});

		await user.click(screen.getByRole('radio', { name: 'Banner' }));

		expect(props.events.onVolumeViewModeChange).toHaveBeenCalledWith('banner');
	});
});
