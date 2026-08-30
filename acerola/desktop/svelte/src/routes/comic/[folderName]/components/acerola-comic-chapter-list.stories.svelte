<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaComicChapterList, { type Chapter } from './acerola-comic-chapter-list.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Comic/AcerolaComicChapterList',
		component: AcerolaComicChapterList,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Lista de capítulos do comic.'
				}
			}
		}
	});
</script>

<script lang="ts">
	const pagesData = [
		{
			page: 0,
			items: [
				{
					id: 'c1',
					title: 'Capítulo 1: O Início',
					chapterSort: '1',
					fileName: '001.cbz',
					isRead: true
				},
				{
					id: 'c2',
					title: 'Capítulo 2: A Descoberta',
					chapterSort: '2',
					fileName: '002.cbz',
					isRead: false
				}
			]
		}
	];
</script>

<Story name="With Chapters" asChild>
	<AcerolaComicChapterList data={{ pagesData, totalChapters: 2, pageSize: 25 }} />
</Story>

<Story name="Empty" asChild>
	<AcerolaComicChapterList data={{ pagesData: [], totalChapters: 0, pageSize: 25 }} />
</Story>

<Story name="With Conflict" asChild>
	<AcerolaComicChapterList
		data={{
			pagesData: [
				{
					page: 0,
					items: [
						{
							id: 'c1',
							title: 'Capítulo 1: O Início',
							chapterSort: '1',
							fileName: '001.cbz',
							isRead: true
						},
						{
							id: 'c5',
							title: 'Capítulo 5 (conflito-peer1)',
							chapterSort: '5',
							fileName: '005.cbz',
							isRead: false,
							hasConflict: true
						}
					]
				}
			],
			totalChapters: 2,
			pageSize: 25
		}}
	/>
</Story>
