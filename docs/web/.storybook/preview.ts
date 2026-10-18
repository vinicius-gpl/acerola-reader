/// <reference types="vite/client" />
import type { Preview } from '@storybook/sveltekit';
import { overwriteSetLocale } from '$lib/paraglide/runtime';
import './storybook.css';

// setLocale() real navega/recarrega a página (ver src/lib/paraglide/runtime.js) — dentro do
// preview do Storybook isso navega o iframe pra uma URL que não existe no build estático,
// quebrando a página (AcerolaNavControls/AcerolaTopNav). overwriteSetLocale é o hook oficial
// do paraglide pra isso, então nenhum componente precisa saber que está rodando no Storybook.
overwriteSetLocale(() => {});

export const globalTypes = {
	theme: {
		name: 'Theme',
		defaultValue: 'dark',
		toolbar: {
			icon: 'paintbrush',
			items: [
				{ value: 'dark', title: 'Dark' },
				{ value: 'light', title: 'Light' }
			],
			dynamicTitle: true
		}
	}
};

const preview: Preview = {
	decorators: [
		(Story, context) => {
			const theme = (context.globals.theme as string) ?? 'dark';

			document.documentElement.setAttribute('data-theme', theme);
			document.documentElement.classList.toggle('dark', theme === 'dark');

			return Story();
		}
	],
	parameters: {
		docs: {
			source: {
				type: 'dynamic',
				language: 'svelte'
			}
		},
		controls: {
			matchers: {
				color: /(background|color)$/i,
				date: /Date$/i
			}
		},
		a11y: {
			test: 'todo'
		}
	}
};

export default preview;
