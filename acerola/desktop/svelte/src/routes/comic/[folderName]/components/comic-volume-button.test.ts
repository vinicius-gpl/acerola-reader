import { describe, expect, it, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ComicVolumeButton from './acerola-comic-volume-button.svelte';

describe('ComicVolumeButton', () => {
	it('should render the title and chapter count correctly', () => {
		const { getByText } = render(ComicVolumeButton, {
			props: {
				data: {
					title: 'Volume 1',
					totalChapters: 10
				},
				events: {
					onClick: vi.fn()
				}
			}
		});

		expect(getByText('Volume 1')).toBeInTheDocument();
		expect(getByText('10 capítulos inclusos')).toBeInTheDocument();
	});

	it('should call onclick when button is clicked', async () => {
		const mockClick = vi.fn();
		const { getByRole } = render(ComicVolumeButton, {
			props: {
				data: {
					title: 'Volume 1',
					totalChapters: 10
				},
				events: {
					onClick: mockClick
				}
			}
		});

		const button = getByRole('button');
		await fireEvent.click(button);

		expect(mockClick).toHaveBeenCalledOnce();
	});

	it('should render cover image when viewMode is cover and coverUri is provided', () => {
		const { getByAltText } = render(ComicVolumeButton, {
			props: {
				data: {
					title: 'Volume 1',
					totalChapters: 10,
					viewMode: 'cover',
					coverUri: 'cover.jpg'
				},
				events: {
					onClick: vi.fn()
				}
			}
		});

		const img = getByAltText('Volume 1');
		expect(img).toBeInTheDocument();
		expect(img.getAttribute('src')).toBe('cover.jpg');
	});

	it('should fallback to bannerUri when viewMode is cover but coverUri is missing', () => {
		const { getByAltText } = render(ComicVolumeButton, {
			props: {
				data: {
					title: 'Volume 1',
					totalChapters: 10,
					viewMode: 'cover',
					bannerUri: 'banner.jpg'
				},
				events: {
					onClick: vi.fn()
				}
			}
		});

		const img = getByAltText('Volume 1');
		expect(img).toBeInTheDocument();
		expect(img.getAttribute('src')).toBe('banner.jpg');
	});

	it('should render banner image when viewMode is banner and bannerUri is provided', () => {
		const { getAllByAltText } = render(ComicVolumeButton, {
			props: {
				data: {
					title: 'Volume 1',
					totalChapters: 10,
					viewMode: 'banner',
					bannerUri: 'banner.jpg'
				},
				events: {
					onClick: vi.fn()
				}
			}
		});

		const img = getAllByAltText('Volume 1').at(-1);
		expect(img).toBeInTheDocument();
		expect(img?.getAttribute('src')).toBe('banner.jpg');
	});

	it('should fallback to coverUri when viewMode is banner but bannerUri is missing', () => {
		const { getAllByAltText } = render(ComicVolumeButton, {
			props: {
				data: {
					title: 'Volume 1',
					totalChapters: 10,
					viewMode: 'banner',
					coverUri: 'cover.jpg'
				},
				events: {
					onClick: vi.fn()
				}
			}
		});

		const img = getAllByAltText('Volume 1').at(-1);
		expect(img).toBeInTheDocument();
		expect(img?.getAttribute('src')).toBe('cover.jpg');
	});
});
