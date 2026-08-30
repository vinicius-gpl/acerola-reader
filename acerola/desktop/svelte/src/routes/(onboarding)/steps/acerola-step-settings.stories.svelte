<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepSettings from './acerola-step-settings.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepSettings,
		title: 'Páginas/Onboarding/AcerolaStepSettings',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Passo de onboarding para escolher o tema e a pasta da biblioteca. Usa os hooks de tema e seleção de pasta internamente (chamadas ao Tauri falham silenciosamente fora do app real).'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepSettings onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
