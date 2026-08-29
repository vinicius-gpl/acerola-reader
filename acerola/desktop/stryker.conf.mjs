// @ts-check
/** @type {import('@stryker-mutator/api/core').PartialStrykerOptions} */
export default {
	packageManager: 'npm',
	testRunner: 'vitest',
	vitest: {
		configFile: 'vitest.stryker.config.ts'
	},
	coverageAnalysis: 'perTest',
	buildCommand: 'npx svelte-kit sync',
	ignorePatterns: [
		'src-tauri',
		'tests/wdio',
		'svelte/src/lib/paraglide',
		'coverage',
		'test-results',
		'playwright-report',
		'storybook-static',
		'mutants.out',
		'mutants.out.old'
	],
	mutate: [
		'svelte/src/lib/hooks/**/*.svelte.ts',
		'svelte/src/lib/state/**/*.svelte.ts',
		'svelte/src/lib/services/**/*.ts',
		'svelte/src/lib/utils/**/*.ts',
		'!svelte/src/**/*.test.ts',
		'!svelte/src/**/*.browser.test.ts',
		'!svelte/src/**/*.stories.svelte',
		'!svelte/src/lib/paraglide/**',
		'!svelte/src/lib/components/ui/**'
	],
	reporters: ['html', 'clear-text', 'progress'],
	htmlReporter: {
		fileName: 'mutants.out/mutation-report.html'
	},
	thresholds: {
		// Trocado de placeholder pro baseline real medido (49.03% na primeira rodada
		// completa) — high/low ajustados pra refletir a distribuição real por arquivo em
		// vez de metas arbitrárias. Atualização: depois de matar os 168 survivors reais
		// mapeados numa rodada de trabalho dedicada, o score subiu pra 71.06%, com 0
		// survivors nos arquivos medidos (hooks/state/services/utils com cobertura) — os
		// poucos mutantes que sobraram são equivalentes de verdade, documentados com
		// `// Stryker disable next-line <Mutator>: <razão>` direto no source de cada
		// arquivo (não threshold nem supressão cega). O que resta pra chegar perto do rigor
		// do lado Rust é só o bug de medição "no coverage" documentado abaixo.
		//
		// TODO (não corrigido, só documentado): os arquivos abaixo apareceram majoritariamente
		// como "no coverage" no relatório (mutants.out/mutation-report.html) MESMO TENDO
		// teste escrito — ou seja, o `coverageAnalysis: perTest` do Stryker não está
		// conseguindo linkar nenhum teste aos mutantes desses arquivos, então o mutation
		// score real deles é desconhecido, não necessariamente zero. Suspeita: hooks com
		// estado em nível de módulo (`$state` fora de função) ou chamados fora de um
		// componente montado confundem o rastreio per-test. Precisa de investigação
		// dedicada, não é um problema de configuração do Stryker em si:
		// - use-comic-summary.svelte.ts (100 mutantes, 0 cobertura)
		// - use-comic-scanner.svelte.ts (56 mutantes, 0 cobertura)
		// - use-comic-chapters.svelte.ts (76 mutantes, 0 cobertura)
		// - use-select-folder.svelte.ts (33 mutantes, 0 cobertura)
		// - use-history.svelte.ts (57 mutantes, 0 cobertura)
		// - use-theme.svelte.ts (parcial: 8 de 43 mutantes sem cobertura)
		//
		// Resolvido: use-metadata-language.svelte.ts, use-reader-mode.svelte.ts e
		// use-relay-settings.svelte.ts chegaram a 100% de mutation score real (43/43
		// mortos, 0 no-coverage) depois de ganharem teste — reforça a suspeita acima: os
		// três têm `$state` só dentro da função exportada (instância nova por chamada,
		// sem singleton em nível de módulo), diferente dos arquivos que ainda aparecem
		// como "no coverage" nesta lista.
		high: 70,
		low: 50,
		break: 45
	}
};
