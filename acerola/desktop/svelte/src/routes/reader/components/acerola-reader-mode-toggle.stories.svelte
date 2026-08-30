<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ReaderModeToggle from './acerola-reader-mode-toggle.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/ReaderModeToggle',
		component: ReaderModeToggle,
		tags: ['autodocs'],
		argTypes: {
			state: { control: 'object' },
			events: { control: 'object' },
			ui: { control: 'object' }
		}
	});
</script>

<script lang="ts">
	let value = $state<'vertical' | 'horizontal' | 'webtoon'>('vertical');

	const events = {
		onValueChange: (nextValue: typeof value) => (value = nextValue)
	};
</script>

<Story name="Desktop" asChild>
	<ReaderModeToggle state={{ value }} {events} />
</Story>

<Story name="Mobile" asChild>
	<ReaderModeToggle state={{ value }} {events} ui={{ variant: 'mobile' }} />
</Story>

<Story name="Webtoon Active" asChild>
	<ReaderModeToggle state={{ value: 'webtoon' }} {events} />
</Story>
