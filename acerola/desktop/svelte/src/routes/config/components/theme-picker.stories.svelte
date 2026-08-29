<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import ThemePicker from './theme-picker.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Config/ThemePicker',
		component: ThemePicker,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Seletor de paleta de cores da aplicação. Componente de página.'
				}
			}
		},
		argTypes: {
			data: { description: 'Tema e modo atual', control: 'object' },
			events: {
				description: 'Callback quando um tema é selecionado',
				control: 'object'
			}
		}
	});
</script>

<script lang="ts">
	import { THEMES } from '$lib/constants/themes';
	import type { ThemeColor, ThemeMode } from '$lib/hooks/theme/use-theme.svelte';

	let theme = $state<ThemeColor>('catppuccin');

	const mode = $derived<ThemeMode>(
		document.documentElement.classList.contains('dark') ? 'dark' : 'light'
	);

	function onSelect(name: ThemeColor) {
		theme = name;
		document.documentElement.setAttribute('data-theme', THEMES[name][mode]);
		document.documentElement.classList.toggle('dark', mode === 'dark');
	}
</script>

<Story name="Default" asChild>
	<ThemePicker data={{ theme, mode }} events={{ onSelect }} />
</Story>
