<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import AcerolaButtonIcon from './acerola-button-icon.svelte';

	const { Story } = defineMeta({
		title: 'Primitivos/AcerolaButtonIcon',
		component: AcerolaButtonIcon,
		tags: ['autodocs'],
		argTypes: {
			ui: { description: 'Configuração visual do botão', control: 'object' }
		}
	});
</script>

<Story name="Moon">
	<AcerolaButtonIcon>
		<MoonIcon size={16} />
	</AcerolaButtonIcon>
</Story>

<Story name="Sun">
	<AcerolaButtonIcon>
		<SunIcon size={16} />
	</AcerolaButtonIcon>
</Story>

<Story name="Settings">
	<AcerolaButtonIcon>
		<SettingsIcon size={16} />
	</AcerolaButtonIcon>
</Story>

<Story name="Disabled">
	<AcerolaButtonIcon ui={{ disabled: true }}>
		<MoonIcon size={16} />
	</AcerolaButtonIcon>
</Story>
