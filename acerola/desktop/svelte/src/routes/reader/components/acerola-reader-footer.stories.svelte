<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaReaderFooter from './acerola-reader-footer.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/AcerolaReaderFooter',
		component: AcerolaReaderFooter,
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
		onReadingModeChange: (nextMode: typeof readingMode) => (readingMode = nextMode)
	};
</script>

<Story name="Default" asChild>
	<AcerolaReaderFooter
		data={{
			pageProgressPercent: 45,
			pageProgressWidth: '45%',
			chapterProgressLabel: 'Chapter 3 of 8',
			modeLabel: 'Vertical',
			zoomStatusLabel: 'Zoom 100%',
			chaptersRemainingLabel: '5 chapters remaining'
		}}
		state={{ readingMode }}
		{events}
	/>
</Story>

<Story name="Start" asChild>
	<AcerolaReaderFooter
		data={{
			pageProgressPercent: 0,
			pageProgressWidth: '0%',
			chapterProgressLabel: 'Chapter 1 of 8',
			modeLabel: 'Horizontal',
			zoomStatusLabel: 'Zoom 100%',
			chaptersRemainingLabel: '7 chapters remaining'
		}}
		state={{ readingMode: 'horizontal' }}
		{events}
	/>
</Story>

<Story name="Complete" asChild>
	<AcerolaReaderFooter
		data={{
			pageProgressPercent: 100,
			pageProgressWidth: '100%',
			chapterProgressLabel: 'Chapter 8 of 8',
			modeLabel: 'Webtoon',
			zoomStatusLabel: 'Zoom 165%',
			chaptersRemainingLabel: '0 chapters remaining'
		}}
		state={{ readingMode: 'webtoon' }}
		{events}
	/>
</Story>
