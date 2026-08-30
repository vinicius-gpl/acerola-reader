import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaComicMetadataPanel from './acerola-comic-metadata-panel.svelte';

describe('AcerolaComicMetadataPanel', () => {
	const defaultProps = {
		data: {
			title: 'Test Manga',
			author: 'Test Author',
			status: 'Ongoing',
			source: 'LOCAL',
			chaptersCount: 15,
			description: 'Test description',
			cover: 'https://test.com/cover.jpg'
		},
		events: {
			onBack: vi.fn()
		}
	};

	it('renders title, author, category and chapter count', () => {
		render(AcerolaComicMetadataPanel, { props: defaultProps });
		expect(screen.getByRole('heading', { name: 'Test Manga', level: 1 })).toBeInTheDocument();
		expect(screen.getByText('Test Author')).toBeInTheDocument();
		expect(screen.getByText('Ongoing')).toBeInTheDocument();
		expect(screen.getByText('LOCAL')).toBeInTheDocument();
		expect(screen.getByText('15 Caps')).toBeInTheDocument();
		expect(screen.getByText('Test description')).toBeInTheDocument();
	});

	it('calls onBack when back button is clicked', async () => {
		const user = userEvent.setup();
		const onBack = vi.fn();
		const { container } = render(AcerolaComicMetadataPanel, {
			props: {
				...defaultProps,
				events: { onBack }
			}
		});

		const backButton = container.querySelector('button');
		expect(backButton).toBeInTheDocument();
		if (backButton) {
			await user.click(backButton);
			expect(onBack).toHaveBeenCalledOnce();
		}
	});
});
