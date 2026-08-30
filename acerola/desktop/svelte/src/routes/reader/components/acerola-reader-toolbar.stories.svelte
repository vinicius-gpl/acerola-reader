<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaReaderToolbar from './acerola-reader-toolbar.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/AcerolaReaderToolbar',
		component: AcerolaReaderToolbar,
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
	<AcerolaReaderToolbar data={baseData} state={{ readingMode }} {events} />
</Story>

<Story name="Zoom Active" asChild>
	<AcerolaReaderToolbar
		data={{ ...baseData, zoomLevel: 1.65, zoomMode: true }}
		state={{ readingMode }}
		{events}
	/>
</Story>

<Story name="No Subtitle" asChild>
	<AcerolaReaderToolbar
		data={{ ...baseData, subtitle: undefined }}
		state={{ readingMode }}
		{events}
	/>
</Story>

<Story name="Locked Pagination" asChild>
	<AcerolaReaderToolbar
		data={{ ...baseData, pageControlsDisabled: true }}
		state={{ readingMode }}
		{events}
	/>
</Story>

<Story name="Webtoon Mode" asChild>
	<AcerolaReaderToolbar
		data={{ ...baseData, isPaginatedMode: false }}
		state={{ readingMode: 'webtoon' }}
		{events}
	/>
</Story>
