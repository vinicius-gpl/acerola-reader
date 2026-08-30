<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaComicPreferences from './acerola-comic-preferences.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Comic/AcerolaComicPreferences',
		component: AcerolaComicPreferences,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Painel de preferências da página do comic.'
				}
			}
		}
	});
</script>

<script lang="ts">
	let displayMode = $state('Lista');
	let mediaType = $state('Manga');
	let volumeViewMode = $state<'cover' | 'banner'>('cover');
</script>

<Story name="Default" asChild>
	<AcerolaComicPreferences
		data={{ hasVolumeStructure: true }}
		state={{
			volumeViewMode,
			bookmarkId: null,
			externalSyncEnabled: true
		}}
		events={{
			onVolumeViewModeChange: (value) => (volumeViewMode = value),
			onBookmarkChange: () => {},
			onExternalSyncChange: () => {}
		}}
	/>
</Story>
