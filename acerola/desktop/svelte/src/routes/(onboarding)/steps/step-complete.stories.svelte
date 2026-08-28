<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import StepComplete from './step-complete.svelte';

	const { Story } = defineMeta({
		component: StepComplete,
		title: 'Pages/Onboarding/StepComplete',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Último passo do onboarding: confirmação de conclusão.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-96 bg-surface">
		<StepComplete onPrev={() => {}} onComplete={() => {}} />
	</div>
</Story>
