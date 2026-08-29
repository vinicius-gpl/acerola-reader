<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ReaderToolbar from './reader-toolbar.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/ReaderToolbar',
		component: ReaderToolbar,
		tags: ['autodocs'],
		argTypes: {
			data: { control: 'object' },
			state: { control: 'object' },
			events: { control: 'object' }
		}
	});
</script>

<script lang="ts">
	let readingMode = $state<'vertical' | 'horizontal' | 'webtoon'>('vertical');

	const events = {
		onBack: () => {},
		onReadingModeChange: (nextMode: typeof readingMode) => (readingMode = nextMode),
		onToggleQuickZoom: () => {},
		onToggleZoomMode: () => {},
		onOpenCommandPalette: () => {},
		onPreviousPage: () => {},
		onNextPage: () => {}
	};

	const baseData = {
		title: 'Chapter 12',
		subtitle: 'Volume 2',
		zoomLevel: 1,
		zoomMode: false,
		isPaginatedMode: true,
		pageControlsDisabled: false,
		canPreviousPage: true,
		canNextPage: true
	};
</script>

<Story name="Default" asChild>
	<ReaderToolbar data={baseData} state={{ readingMode }} {events} />
</Story>

<Story name="Zoom Active" asChild>
	<ReaderToolbar
		data={{ ...baseData, zoomLevel: 1.65, zoomMode: true }}
		state={{ readingMode }}
		{events}
	/>
</Story>

<Story name="No Subtitle" asChild>
	<ReaderToolbar data={{ ...baseData, subtitle: undefined }} state={{ readingMode }} {events} />
</Story>

<Story name="Locked Pagination" asChild>
	<ReaderToolbar
		data={{ ...baseData, pageControlsDisabled: true }}
		state={{ readingMode }}
		{events}
	/>
</Story>

<Story name="Webtoon Mode" asChild>
	<ReaderToolbar
		data={{ ...baseData, isPaginatedMode: false }}
		state={{ readingMode: 'webtoon' }}
		{events}
	/>
</Story>
