<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaReaderCommandPalette from './acerola-reader-command-palette.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/AcerolaReaderCommandPalette',
		component: AcerolaReaderCommandPalette,
		tags: ['autodocs'],
		argTypes: {
			data: { control: 'object' },
			state: { control: 'object' },
			events: { control: 'object' }
		}
	});
</script>

<script lang="ts">
	let value = $state('');
	let open = $state(true);
	let readingMode = $state<'vertical' | 'horizontal' | 'webtoon'>('vertical');
	let zoomMode = $state(false);

	const events = {
		onOpenChange: (nextOpen: boolean) => (open = nextOpen),
		onValueChange: (nextValue: string) => (value = nextValue),
		onReadingModeChange: (nextMode: typeof readingMode) => (readingMode = nextMode),
		onToggleZoomMode: () => (zoomMode = !zoomMode),
		onZoomIn: () => {},
		onZoomOut: () => {},
		onResetZoom: () => {}
	};
</script>

<Story name="Open" asChild>
	<AcerolaReaderCommandPalette data={{ zoomMode }} state={{ open, value, readingMode }} {events} />
</Story>

<Story name="Zoom Mode Active" asChild>
	<AcerolaReaderCommandPalette
		data={{ zoomMode: true }}
		state={{ open: true, value: 'zoom', readingMode: 'horizontal' }}
		{events}
	/>
</Story>

<Story name="Closed" asChild>
	<div class="relative h-64 overflow-hidden rounded-lg border border-surface/40 bg-base">
		<AcerolaReaderCommandPalette
			data={{ zoomMode: false }}
			state={{ open: false, value: '', readingMode: 'vertical' }}
			{events}
		/>
	</div>
</Story>
