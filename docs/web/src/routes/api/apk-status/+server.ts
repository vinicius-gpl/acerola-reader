import { json } from '@sveltejs/kit';
import { resolveLatestApkKey } from '$lib/server/apk-bucket';
import type { RequestHandler } from './$types';

// Cacheado na borda da Cloudflare por alguns minutos: a checagem ao vivo passa a rodar de
// fato só uma vez por janela, por PoP — o resto das visitas nessa janela reaproveita o
// resultado sem nenhum `list` novo no bucket R2.
const CACHE_TTL_SECONDS = 300;

// Checado direto pelo binding R2 (`APK_BUCKET`), não por HTTP: um `fetch` do worker pra uma
// rota dele mesmo (self-fetch) se mostrou pouco confiável no runtime do Cloudflare — o
// binding elimina esse round-trip de rede e a lógica de retry/timeout que ele exigia.
export const GET: RequestHandler = async ({ platform }) => {
	// `caches.default` (Cache API da Cloudflare) só existe em runtime de Worker de verdade —
	// em testes/dev local cai direto pra checagem ao vivo sem cache, sem quebrar nada. `lib
	// dom` padrão do TS não conhece essa extensão específica da Cloudflare, daí o cast local
	// em vez de puxar a dependência inteira de @cloudflare/workers-types só por isso.
	const edgeCache =
		typeof caches !== 'undefined'
			? (caches as CacheStorage & { default: Cache }).default
			: undefined;
	// Chave fixa (não depende do request de quem chamou): só existe UM resultado possível
	// pra essa rota — não há nada por-usuário/por-query-string pra variar o cache.
	const cacheKey = new Request('https://internal-cache-key.acerola-comic.com/api/apk-status');

	if (edgeCache) {
		const cached = await edgeCache.match(cacheKey);
		if (cached) return cached;
	}

	const bucket = platform?.env?.APK_BUCKET;
	const available = bucket !== undefined && (await resolveLatestApkKey(bucket)) !== null;

	const response = json(
		{ available },
		{ headers: { 'Cache-Control': `public, max-age=${CACHE_TTL_SECONDS}` } }
	);

	if (edgeCache) {
		await edgeCache.put(cacheKey, response.clone());
	}

	return response;
};
