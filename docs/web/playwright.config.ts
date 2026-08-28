import { defineConfig, devices } from '@playwright/test';

// IMPORTANT: the webServer below runs `wrangler dev`, NOT `npm run dev` or
// `npm run preview`. This is deliberate:
//   - `search-dialog.svelte` shows an "unavailable" placeholder whenever
//     `import.meta.env.DEV` is true, skipping Pagefind entirely in dev mode.
//   - The Pagefind index (`pagefind.js` + data files) is built directly into
//     `.svelte-kit/cloudflare/pagefind` (`pagefind --site .svelte-kit/cloudflare
//     --output-subdir pagefind`, part of the `build` script) — that's also the exact
//     directory `wrangler.toml`'s `[assets] directory` serves in production.
//   - `vite preview` does NOT serve from `.svelte-kit/cloudflare` (it serves the
//     adapter-agnostic `.svelte-kit/output/client`), so it 404s on `/pagefind/*` even
//     after a real build — only `wrangler dev` reproduces the actual Cloudflare Worker
//     asset serving locally, pagefind included.
// So a real `npm run build` must happen before `npx playwright test` for the search
// spec to have anything real to hit.
export default defineConfig({
	testDir: './tests/e2e',
	// A home roda um fundo WebGL (`faulty-terminal.svelte`) e o search-dialog
	// carrega o WASM/índice do Pagefind sob demanda na primeira consulta — sob o
	// GPU virtual do Chromium headless no CI isso passa fácil de 10s. O timeout
	// por asserção continua curto (5s); só o teto do teste inteiro é folgado.
	timeout: 60_000,
	workers: 1,
	expect: {
		timeout: 5_000
	},
	retries: process.env.CI ? 2 : 0,
	use: {
		baseURL: 'http://127.0.0.1:5173',
		trace: 'retain-on-failure',
		// A home usa um efeito de fundo em WebGL (`faulty-terminal.svelte`, via `ogl`) que
		// gera "GPU stall"/driver warnings sob o GPU virtual do Chromium headless e pode
		// derrubar a página de vez em quando (mais visível em `page.reload()`) — forçar
		// SwiftShader (renderização por software) dá um contexto WebGL estável em vez de
		// depender do driver de GPU do host, que headless não expõe direito.
		launchOptions: {
			args: ['--use-gl=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist']
		}
	},
	webServer: {
		command: 'npx wrangler dev --port 5173',
		url: 'http://127.0.0.1:5173',
		reuseExistingServer: !process.env.CI,
		timeout: 120_000
	},
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		}
	]
});
