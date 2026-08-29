<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import StepLanguage from './step-language.svelte';

	const { Story } = defineMeta({
		component: StepLanguage,
		title: 'Páginas/Onboarding/StepLanguage',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Passo de onboarding para selecionar o idioma preferido da aplicação.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-96 bg-surface">
		<StepLanguage onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
