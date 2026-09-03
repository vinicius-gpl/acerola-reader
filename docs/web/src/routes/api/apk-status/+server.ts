import { json } from '@sveltejs/kit';
import { APK_URL } from '$lib/constants/site';
import type { RequestHandler } from './$types';

// Checado no servidor, não no browser: o bucket R2 por trás desse domínio não manda
// header de CORS, então um fetch direto do client pra `binary.acerola-comic.com`
// falharia mesmo quando o objeto existe. Servidor-a-servidor não tem essa restrição.
export const GET: RequestHandler = async ({ fetch }) => {
	try {
		const response = await fetch(APK_URL, {
			method: 'HEAD',
			signal: AbortSignal.timeout(5000)
		});

		return json({ available: response.ok });
	} catch {
		// Timeout, DNS, conexão recusada etc. — trata como indisponível igual a um 404,
		// já que o efeito pro usuário é o mesmo: o download não vai funcionar.
		return json({ available: false });
	}
};
