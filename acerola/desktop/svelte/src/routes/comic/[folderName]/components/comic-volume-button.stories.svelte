<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ComicVolumeButton from './comic-volume-button.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Comic/ComicVolumeButton',
		component: ComicVolumeButton,
		argTypes: {
			data: { control: 'object' },
			events: { control: 'object' }
		}
	});
</script>

<Story
	name="Default (No Image)"
	args={{
		data: {
			title: 'Volume 1',
			totalChapters: 12,
			viewMode: 'cover',
			isExpanded: false
		},
		events: {
			onClick: () => {}
		}
	}}
/>

<Story
	name="With Cover"
	args={{
		data: {
			title: 'Volume 1',
			totalChapters: 12,
			viewMode: 'cover',
			coverUri: 'https://placehold.co/400x600/png',
			isExpanded: false
		},
		events: {
			onClick: () => {}
		}
	}}
/>

<Story
	name="With Banner"
	args={{
		data: {
			title: 'Volume 1',
			totalChapters: 12,
			viewMode: 'banner',
			bannerUri: 'https://placehold.co/800x400/png',
			isExpanded: false
		},
		events: {
			onClick: () => {}
		}
	}}
/>

<Story
	name="Expanded"
	args={{
		data: {
			title: 'Volume 1',
			totalChapters: 12,
			viewMode: 'cover',
			isExpanded: true
		},
		events: {
			onClick: () => {}
		}
	}}
/>
