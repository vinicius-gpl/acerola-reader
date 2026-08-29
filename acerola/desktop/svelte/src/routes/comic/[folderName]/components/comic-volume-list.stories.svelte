<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ComicVolumeList, { type Volume } from './comic-volume-list.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Comic/ComicVolumeList',
		component: ComicVolumeList,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Lista expansível de volumes do comic.'
				}
			}
		}
	});
</script>

<script lang="ts">
	const volumes: Volume[] = [
		{
			id: 'v1',
			title: 'Volume 1',
			totalChapters: 2,
			hasMore: false,
			chapters: [
				{ id: 'c1', title: 'Capítulo 1: O Início', fileName: '001.cbz', isRead: true },
				{ id: 'c2', title: 'Capítulo 2: A Descoberta', fileName: '002.cbz', isRead: false }
			]
		},
		{
			id: 'v2',
			title: 'Volume 2',
			totalChapters: 1,
			hasMore: false,
			chapters: [{ id: 'c3', title: 'Capítulo 3: O Desafio', fileName: '003.cbz', isRead: false }]
		}
	];
</script>

<Story name="With Volumes" asChild>
	<ComicVolumeList data={{ volumes }} events={{ onExpand: () => {} }} />
</Story>

<Story name="Empty" asChild>
	<ComicVolumeList data={{ volumes: [] }} events={{ onExpand: () => {} }} />
</Story>
