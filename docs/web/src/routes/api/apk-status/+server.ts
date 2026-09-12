import { json } from '@sveltejs/kit';
import { APK_URL } from '$lib/constants/site';
import type { RequestHandler } from './$types';

// Cacheado na borda da Cloudflare por alguns minutos: a checagem ao vivo passa a rodar de
// fato só uma vez por janela, por PoP — o resto das visitas nessa janela reaproveita o
// resultado sem nenhuma subrequest nova pro R2. Isso reduz o quanto uma conexão fria
// (Worker do docs -> domínio do R2) chega a afetar o usuário, em vez de depender de
// adivinhar um timeout "grande o suficiente" pra essa subrequest.
const CACHE_TTL_SECONDS = 300;

// Timeout por tentativa: rede de segurança pra nunca deixar a requisição pendurada pra
// sempre — não é o mecanismo principal de resiliência (isso é o retry abaixo + o cache
// acima), só o limite de última instância.
const PER_ATTEMPT_TIMEOUT_MS = 4000;

// Duas tentativas em vez de uma espera única mais longa: o padrão observado ao vivo foi
// "a primeira subrequest fria falha, a seguinte (DNS/TLS já resolvidos) funciona muito
// rápido" — repetir ataca exatamente esse padrão, sem inflar o timeout de uma tentativa só
// (que piora o pior caso quando o arquivo está mesmo indisponível). Só tenta de novo em
// falha de REDE (catch) — uma resposta HTTP de verdade (mesmo um 404) já é definitiva,
// tentar de novo não muda nada.
async function checkApkAvailable(fetchImpl: typeof fetch): Promise<boolean> {
	for (let attempt = 0; attempt < 2; attempt++) {
		try {
			const response = await fetchImpl(APK_URL, {
				method: 'HEAD',
				signal: AbortSignal.timeout(PER_ATTEMPT_TIMEOUT_MS)
			});
			return response.ok;
		} catch {
			// Timeout, DNS, conexão recusada etc. — só nesse caso vale tentar de novo.
			if (attempt === 1) return false;
		}
	}
	return false;
}

// Checado no servidor, não no browser: o bucket R2 por trás desse domínio não manda
// header de CORS, então um fetch direto do client pra `binary.acerola-comic.com`
// falharia mesmo quando o objeto existe. Servidor-a-servidor não tem essa restrição.
export const GET: RequestHandler = async ({ fetch }) => {
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

	const available = await checkApkAvailable(fetch);
	const response = json(
		{ available },
		{ headers: { 'Cache-Control': `public, max-age=${CACHE_TTL_SECONDS}` } }
	);

	if (edgeCache) {
		await edgeCache.put(cacheKey, response.clone());
	}

	return response;
};
