import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

// Config enxuta usada só pelo Stryker (stryker.conf.json) — mutation testing só faz
// sentido pro projeto `unit` (lógica pura em .ts/.svelte.ts). O projeto `storybook` do
// vitest.config.ts principal usa browser real via Playwright, o que só deixaria cada
// mutante mais lento pra rodar sem agregar sinal de mutação relevante.
export default defineConfig({
	plugins: [sveltekit(), svelteTesting()],
	test: {
		globals: true,
		environment: 'jsdom',
		include: ['src/**/*.test.ts'],
		setupFiles: ['tests/setup.ts']
	}
});
