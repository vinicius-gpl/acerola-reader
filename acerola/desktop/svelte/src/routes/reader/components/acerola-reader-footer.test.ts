import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaReaderFooter from './acerola-reader-footer.svelte';

function props(overrides = {}) {
	return {
		data: {
			pageProgressPercent: 45,
			pageProgressWidth: '45%',
			chapterProgressLabel: 'Chapter 2 of 8',
			modeLabel: 'Vertical',
			zoomStatusLabel: 'Zoom 100%',
			chaptersRemainingLabel: '6 chapters remaining'
		},
		state: {
			readingMode: 'vertical'
		},
		events: {
			onReadingModeChange: vi.fn()
		},
		...overrides
	} as const;
}

describe('AcerolaReaderFooter', () => {
	it('renders progress, mode, zoom and remaining chapters', () => {
		render(AcerolaReaderFooter, { props: props() });

		expect(screen.getByText('45% lido')).toBeInTheDocument();
		expect(screen.getByText('Vertical - Zoom 100%')).toBeInTheDocument();
		expect(screen.getByText('6 chapters remaining')).toBeInTheDocument();

		const progress = screen.getByRole('progressbar');
		expect(progress).toHaveAttribute('aria-valuenow', '45');
		expect(progress).toHaveAttribute('title', 'Chapter 2 of 8');
		expect(progress.firstElementChild).toHaveStyle({ width: '45%' });
	});

	it('renders extreme progress values', () => {
		const values = [
			{ percent: 0, width: '0%' },
			{ percent: 100, width: '100%' }
		];

		for (const value of values) {
			const { unmount } = render(AcerolaReaderFooter, {
				props: props({
					data: {
						...props().data,
						pageProgressPercent: value.percent,
						pageProgressWidth: value.width
					}
				})
			});

			const progress = screen.getByRole('progressbar');
			expect(screen.getByText(`${value.percent}% lido`)).toBeInTheDocument();
			expect(progress).toHaveAttribute('aria-valuenow', String(value.percent));
			expect(progress.firstElementChild).toHaveStyle({ width: value.width });
			unmount();
		}
	});

	it('propagates reading mode change through mobile selector', async () => {
		const user = userEvent.setup();
		const footerProps = props();

		render(AcerolaReaderFooter, { props: footerProps });
		await user.click(screen.getByTitle('Paginado horizontal'));
		await user.click(screen.getByTitle('Webtoon'));

		expect(footerProps.events.onReadingModeChange).toHaveBeenCalledWith('horizontal');
		expect(footerProps.events.onReadingModeChange).toHaveBeenCalledWith('webtoon');
	});
});
