<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaComicHeroBanner from './acerola-comic-hero-banner.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Comic/AcerolaComicHeroBanner',
		component: AcerolaComicHeroBanner,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Banner do topo da página do comic com estatísticas flutuantes.'
				}
			}
		}
	});
</script>

<script lang="ts">
	const banner = 'https://placehold.co/1200x400/1e1e2e/a6accd?text=Banner';
</script>

<Story name="With Banner" asChild>
	<AcerolaComicHeroBanner data={{ banner }} />
</Story>

<Story name="Without Banner" asChild>
	<AcerolaComicHeroBanner data={{ banner: null }} />
</Story>
