<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepFormats from './acerola-step-formats.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepFormats,
		title: 'Páginas/Onboarding/AcerolaStepFormats',
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
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepFormats onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
