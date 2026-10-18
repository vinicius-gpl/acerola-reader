/// <reference types="vite/client" />
import type { Preview } from '@storybook/sveltekit';
import './storybook.css';

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
