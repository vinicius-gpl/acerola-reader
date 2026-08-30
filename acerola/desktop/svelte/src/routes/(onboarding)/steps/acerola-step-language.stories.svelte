<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepLanguage from './acerola-step-language.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepLanguage,
		title: 'Páginas/Onboarding/AcerolaStepLanguage',
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
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepLanguage onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
