import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';
import path from 'path';
import { defineConfig } from 'vitest/config';
import svg from '@poppanator/sveltekit-svg';

// Config enxuta usada só pelo Stryker (stryker.conf.json) — mutation testing só faz
// sentido pro projeto `unit` (lógica pura em .ts/.svelte.ts). Os projetos `integration`/
// `storybook` do vitest.config.ts principal usam browser real via Playwright, o que só
// deixaria cada mutante mais lento pra rodar sem agregar sinal de mutação relevante.
export default defineConfig({
	plugins: [
		svelte({
			hot: false,
			compilerOptions: {
				dev: true
			}
		}),
		svg({
			includePaths: ['./svelte/src/lib/assets/'],
			svgoOptions: {
				multipass: true,
				plugins: ['preset-default']
			}
		}),
		svelteTesting()
	],
	optimizeDeps: {
		include: [
			'@tauri-apps/api/window',
			'aria-query',
			'axobject-query',
			'lz-string',
			'dom-accessibility-api',
			'pretty-format',
			'picocolors'
		]
	},
	resolve: {
		alias: {
			$lib: path.resolve('./svelte/src/lib'),
			$theme: path.resolve('./svelte/src/theme'),
			$services: path.resolve('./svelte/src/services'),
			'$app/state': path.resolve('./svelte/tests/mocks/app-state.ts'),
			'$app/environment': path.resolve('./svelte/tests/mocks/app-environment.ts'),
			'$app/navigation': path.resolve('./svelte/tests/mocks/app-navigation.ts')
		},
		conditions: ['browser', 'svelte', 'development'],
		// Stryker roda a partir de um sandbox copiado com node_modules SIMLINKADO de volta
		// pro projeto real (.stryker-tmp/sandbox*) — sem isso o Vite/Rolldown resolve os
		// symlinks pro caminho real fora do sandbox e quebra a busca de tsconfig durante a
		// otimização de dependências ("Tsconfig not found" ao tentar resolver 'node:module').
		preserveSymlinks: true
	},
	test: {
		globals: true,
		environment: 'jsdom',
		include: ['svelte/src/**/*.test.ts'],
		exclude: ['svelte/src/**/*.browser.test.ts'],
		setupFiles: ['svelte/tests/setup.ts']
	}
});
