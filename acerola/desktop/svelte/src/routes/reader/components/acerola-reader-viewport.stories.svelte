<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { ReaderZoomController } from '../hooks/acerola-use-reader-zoom.svelte';
	import ReaderViewport from './acerola-reader-viewport.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/ReaderViewport',
		component: ReaderViewport,
		tags: ['autodocs'],
		argTypes: {
			data: { control: 'object' },
			context: { control: 'object' }
		}
	});

	function zoom(overrides: Partial<ReaderZoomController> = {}): ReaderZoomController {
		return {
			setViewport: () => {},
			clampPan: () => {},
			resetPan: () => {},
			zoomIn: () => {},
			zoomOut: () => {},
			resetZoom: () => {},
			forceResetZoom: () => {},
			toggleQuickZoom: () => {},
			toggleZoomMode: () => {},
			handleWheel: () => {},
			handlePointerDown: () => {},
			handlePointerMove: () => {},
			stopPan: () => {},
			isZoomed: false,
			zoomLevel: 1,
			zoomMode: false,
			isPanning: false,
			zoomLabel: '100%',
			zoomStatusLabel: 'Zoom 100%' as ReaderZoomController['zoomStatusLabel'],
			zoomLayerStyle: 'transform: translate3d(0px, 0px, 0) scale(1);',
			...overrides
		};
	}
</script>

<Story name="Vertical" asChild>
	<ReaderViewport data={{ mode: 'vertical' }} context={{ zoom: zoom() }}>
		{#snippet children()}
			<div class="grid h-full place-items-center p-8">Vertical viewport</div>
		{/snippet}
	</ReaderViewport>
</Story>

<Story name="Horizontal" asChild>
	<ReaderViewport data={{ mode: 'horizontal' }} context={{ zoom: zoom() }}>
		{#snippet children()}
			<div class="grid h-full w-screen place-items-center p-8">Horizontal viewport</div>
		{/snippet}
	</ReaderViewport>
</Story>

<Story name="Webtoon" asChild>
	<ReaderViewport data={{ mode: 'webtoon' }} context={{ zoom: zoom() }}>
		{#snippet children()}
			<div class="mx-auto grid min-h-[36rem] max-w-3xl place-items-center p-8">
				Webtoon viewport
			</div>
		{/snippet}
	</ReaderViewport>
</Story>

<Story name="Zoomed" asChild>
	<ReaderViewport
		data={{ mode: 'vertical' }}
		context={{
			zoom: zoom({
				isZoomed: true,
				zoomLevel: 1.65,
				zoomLabel: '165%',
				zoomStatusLabel: 'Zoom 165%' as ReaderZoomController['zoomStatusLabel'],
				zoomLayerStyle: 'transform: translate3d(24px, 12px, 0) scale(1.65);'
			})
		}}
	>
		{#snippet children()}
			<div class="grid h-full place-items-center p-8">Zoomed viewport</div>
		{/snippet}
	</ReaderViewport>
</Story>
