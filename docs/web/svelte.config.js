import adapter from '@sveltejs/adapter-cloudflare';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';
import mdsvexConfig from './mdsvex.config.js';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', ...mdsvexConfig.extensions],
	preprocess: [vitePreprocess(), mdsvex(mdsvexConfig)],
	kit: {
		adapter: adapter(),
		alias: {
			$theme: 'src/theme',
			$lib: 'src/lib',
			$routes: 'src/routes'
		},
		prerender: {
			// The locale toggle switches via `setLocale()`, not an `<a href>`, so the
			// crawler never finds these on its own — list them explicitly.
			entries: ['*', '/en', '/en/docs/getting-started', '/en/docs/architecture']
		}
	}
};

export default config;
