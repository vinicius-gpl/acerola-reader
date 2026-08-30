<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepComplete from './acerola-step-complete.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepComplete,
		title: 'Páginas/Onboarding/AcerolaStepComplete',
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
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepComplete onPrev={() => {}} onComplete={() => {}} />
	</div>
</Story>
