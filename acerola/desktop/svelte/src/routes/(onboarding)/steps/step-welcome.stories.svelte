<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import StepWelcome from './step-welcome.svelte';

	const { Story } = defineMeta({
		component: StepWelcome,
		title: 'Páginas/Onboarding/StepWelcome',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Primeiro passo do onboarding: boas-vindas ao Acerola.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-96 bg-surface">
		<StepWelcome onNext={() => {}} />
	</div>
</Story>
