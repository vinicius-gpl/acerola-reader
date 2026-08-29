<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ReaderPages from './reader-pages.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/ReaderPages',
		component: ReaderPages,
		tags: ['autodocs'],
		argTypes: {
			data: { control: 'object' },
			services: { control: 'object' }
		}
	});
</script>

<script lang="ts">
	import type { ReaderCachedPagePayload } from '$lib/contracts/reader/reader.payloads';
	import type { ReaderMode } from '../hooks/use-reader-zoom.svelte';
	import type { ReaderPageTracker } from './reader-pages.svelte';

	const images = [
		'https://placehold.co/900x1300/png?text=Page+1',
		'https://placehold.co/900x1300/png?text=Page+2',
		'https://placehold.co/900x1300/png?text=Page+3',
		'https://placehold.co/900x1300/png?text=Page+4'
	];

	const trackPage: ReaderPageTracker = () => ({ destroy: () => {} });

	function pageAt(index: number): ReaderCachedPagePayload | undefined {
		return images[index]
			? {
					index,
					total: images.length,
					mimeType: 'image/png',
					url: images[index],
					cacheHit: false
				}
			: undefined;
	}

	function partialPageAt(index: number): ReaderCachedPagePayload | undefined {
		return index === 1 ? pageAt(index) : undefined;
	}

	function data(mode: ReaderMode) {
		return {
			mode,
			pageCount: images.length,
			currentPage: 1,
			openFailed: false,
			chapterAvailable: true
		};
	}
</script>

<Story name="Vertical" asChild>
	<ReaderPages data={data('vertical')} services={{ pageAt, trackPage }} />
</Story>

<Story name="Horizontal Partial Cache" asChild>
	<ReaderPages data={data('horizontal')} services={{ pageAt: partialPageAt, trackPage }} />
</Story>

<Story name="Webtoon" asChild>
	<ReaderPages data={data('webtoon')} services={{ pageAt, trackPage }} />
</Story>

<Story name="Loading" asChild>
	<ReaderPages
		data={{ ...data('vertical'), pageCount: 0 }}
		services={{ pageAt: () => undefined, trackPage }}
	/>
</Story>

<Story name="Unavailable" asChild>
	<ReaderPages
		data={{ ...data('vertical'), openFailed: true, chapterAvailable: false }}
		services={{ pageAt: () => undefined, trackPage }}
	/>
</Story>
