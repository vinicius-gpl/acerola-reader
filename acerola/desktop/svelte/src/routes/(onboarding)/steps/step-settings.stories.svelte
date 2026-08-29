<script module lang="ts">
	import { vi } from 'vitest';

	// use-select-folder.svelte.ts imports `load` from '@tauri-apps/plugin-store', but the
	// project-wide browser test mock (svelte/tests/setup.browser.ts) only stubs `LazyStore`
	// on that module — importing this story would otherwise fail with "does not provide an
	// export named 'load'". Redeclaring the mock here (with both exports) overrides it for
	// this file only, without touching the shared test setup.
	vi.mock('@tauri-apps/plugin-store', () => ({
		LazyStore: vi.fn().mockImplementation(function () {
			return {
				get: vi.fn().mockResolvedValue(null),
				set: vi.fn().mockResolvedValue(undefined)
			};
		}),
		load: vi.fn().mockResolvedValue({
			reload: vi.fn().mockResolvedValue(undefined),
			get: vi.fn().mockResolvedValue(null),
			set: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		})
	}));

	import { defineMeta } from '@storybook/addon-svelte-csf';
	import StepSettings from './step-settings.svelte';

	const { Story } = defineMeta({
		component: StepSettings,
		title: 'Páginas/Onboarding/StepSettings',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Passo de onboarding para escolher o tema e a pasta da biblioteca. Usa os hooks de tema e seleção de pasta internamente (dependências do Tauri são mockadas no ambiente de testes/Storybook).'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-96 bg-surface">
		<StepSettings onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
