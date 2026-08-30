<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepWelcome from './acerola-step-welcome.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepWelcome,
		title: 'Páginas/Onboarding/AcerolaStepWelcome',
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
		<AcerolaStepWelcome onNext={() => {}} />
	</div>
</Story>
