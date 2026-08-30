import adapter from '@sveltejs/adapter-cloudflare';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';
import mdsvexConfig from './mdsvex.config.js';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', ...mdsvexConfig.extensions],
	preprocess: [vitePreprocess(), mdsvex(mdsvexConfig)],
	compilerOptions: {
		// rehype-pretty-code puts `tabindex="0"` on <pre> so long code blocks are
		// keyboard-scrollable — a deliberate a11y pattern that Svelte's static
		// checker can't tell apart from a genuine misuse, so it warns anyway.
		warningFilter: (warning) => warning.code !== 'a11y_no_noninteractive_tabindex'
	},
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
			entries: [
				'*',
				'/en',
				'/en/docs/getting-started',
				'/en/docs/architecture',
				'/en/docs/storybook',
				'/en/docs/contributing-overview',
				'/en/docs/contributing-desktop',
				'/en/docs/contributing-android',
				'/en/docs/contributing-p2p',
				'/en/docs/privacy-policy',
				'/en/docs/libraries'
			]
		}
	}
};

export default config;
