<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import StepFormats from './step-formats.svelte';

	const { Story } = defineMeta({
		component: StepFormats,
		title: 'Pages/Onboarding/StepFormats',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Passo de onboarding que apresenta os formatos de arquivo suportados.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-96 bg-surface">
		<StepFormats onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
