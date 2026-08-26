import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { playwright } from '@vitest/browser-playwright';
import { fileURLToPath } from 'node:url';
import path from 'path';
import { defineConfig } from 'vitest/config';

const dirname =
	typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
	plugins: [sveltekit()],
	test: {
		projects: [
			{
				extends: true,
				plugins: [svelteTesting()],
				test: {
					name: 'unit',
					globals: true,
					environment: 'jsdom',
					include: ['src/**/*.test.ts'],
					setupFiles: ['tests/setup.ts']
				}
			},
			{
				extends: true,
				plugins: [
					storybookTest({
						configDir: path.join(dirname, '.storybook')
					})
				],
				test: {
					name: 'storybook',
					setupFiles: ['tests/setup.browser.ts'],
					browser: {
						enabled: true,
						headless: true,
						provider: playwright({}),
						instances: [{ browser: 'chromium' }]
					}
				}
			}
		],
		coverage: {
			provider: 'v8',
			reporter: ['text', 'lcov', 'html'],
			reportsDirectory: './coverage',
			include: ['src/**/*.{ts,svelte}'],
			exclude: [
				'src/**/*.test.ts',
				'src/**/*.stories.svelte',
				'src/lib/paraglide/**',
				'src/lib/components/ui/**',
				'src/**/*.d.ts'
			],
			thresholds: {
				// NOTE: placeholder — projeto começou do zero, ainda sem testes. A fase que
				// mede a cobertura real depois de escrever os testes trava o valor definitivo.
				lines: 0,
				statements: 0,
				functions: 0,
				branches: 0
			}
		}
	}
});
