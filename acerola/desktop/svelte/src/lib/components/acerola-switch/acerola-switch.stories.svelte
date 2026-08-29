<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSwitch from './acerola-switch.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaSwitch',
		component: AcerolaSwitch,
		tags: ['autodocs'],
		parameters: {
			controls: {
				include: ['state', 'ui']
			}
		},
		argTypes: {
			state: { description: 'Estado do switch', control: 'object' },
			ui: { description: 'Configuração visual do switch', control: 'object' }
		}
	});
</script>

<Story name="Default" asChild>
	<AcerolaSwitch />
</Story>

<Story name="Checked" asChild>
	<AcerolaSwitch state={{ checked: true }} />
</Story>

<Story name="Small" asChild>
	<AcerolaSwitch ui={{ size: 'sm' }} />
</Story>

<Story name="Disabled" asChild>
	<AcerolaSwitch ui={{ disabled: true }} />
</Story>
