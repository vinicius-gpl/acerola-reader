import type { StorybookConfig } from '@storybook/sveltekit';
import path from 'path';
import { mergeConfig } from 'vite';

const config: StorybookConfig = {
	stories: ['../src/**/*.mdx', '../src/**/*.stories.@(js|ts|svelte)'],
	addons: [
		'@storybook/addon-svelte-csf',
		'@storybook/addon-vitest',
		'@storybook/addon-a11y',
		'@storybook/addon-docs'
	],
	framework: '@storybook/sveltekit',
	staticDirs: ['../static'],
	viteFinal(config) {
		return mergeConfig(config, {
			server: {
				fs: {
					allow: [path.resolve('src/theme')]
				}
			},
			optimizeDeps: {
				entries: ['../src/**/*.stories.@(js|ts|svelte)', '../src/**/*.svelte'],
				holdUntilResolved: true,
				include: [
					'svelte',
					'svelte/internal',
					'svelte/internal/client',
					'svelte/internal/disclose-version',
					'esm-env',
					'devalue',
					'@storybook/sveltekit/internal/mocks/app/state.svelte.js',
					'@storybook/sveltekit/internal/mocks/app/navigation',
					'@storybook/sveltekit/internal/mocks/app/stores',
					'tailwind-merge',
					'tailwind-variants',
					'bits-ui',
					'clsx',
					'@internationalized/date',
					'@lucide/svelte',
					'aria-query',
					'axobject-query',
					'lz-string',
					'dom-accessibility-api',
					'pretty-format',
					'picocolors'
				]
			}
		});
	}
};

export default config;
