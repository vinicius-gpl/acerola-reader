/// Espelha `acerola_p2p::api::peer::PeerAddr` — sem `rename_all` no Rust, então os campos
/// vêm em snake_case direto do backend (diferente dos nossos próprios payloads).
export type LocalPeerAddr = {
	id: { id: string; device_id: string | null };
	addrs: number[];
};

const CODE_PREFIX = 'acerola1:';

type ConnectionEnvelope = { i: string; d: string | null; a: string };

function bytesToBase64(bytes: number[]): string {
	let binary = '';
	for (const byte of bytes) binary += String.fromCharCode(byte);
	return btoa(binary);
}

function base64ToBytes(base64: string): number[] {
	const binary = atob(base64);
	// Stryker disable next-line ArrayDeclaration : `new Array()` sem tamanho fica byte-a-byte
	// idêntico depois que o loop abaixo preenche cada índice em sequência (0..length-1) sem
	// pular nenhum — não existe entrada observável pela API pública que distinga os dois.
	const bytes: number[] = new Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

/**
 * Empacota um `PeerAddr` num token opaco pra QR Code/copiar-colar. `addrs` é um `Vec<u8>`
 * no Rust — sem isso, `JSON.stringify` serializa cada byte como um número decimal separado
 * por vírgula (~4 caracteres por byte). Codificando os bytes em base64 primeiro, e só então
 * empacotando tudo num único blob base64, o código fica bem mais curto e sem caracteres
 * "assustadores" tipo colchetes/vírgulas — só uma string opaca, do jeito que um código de
 * convite deveria parecer.
 */
export function encodeConnectionCode(addr: LocalPeerAddr): string {
	const envelope: ConnectionEnvelope = {
		i: addr.id.id,
		d: addr.id.device_id,
		a: bytesToBase64(addr.addrs)
	};

	return CODE_PREFIX + btoa(JSON.stringify(envelope));
}

/** Erro específico pra distinguir "código mal formado" de outras falhas na UI. */
export class InvalidConnectionCodeError extends Error {
	constructor() {
		super('invalid_connection_code');
	}
}

export function decodeConnectionCode(code: string): LocalPeerAddr {
	const trimmed = code.trim();

	if (!trimmed.startsWith(CODE_PREFIX)) {
		throw new InvalidConnectionCodeError();
	}

	try {
		const envelope = JSON.parse(atob(trimmed.slice(CODE_PREFIX.length))) as ConnectionEnvelope;

		// Stryker disable next-line OptionalChaining : o catch-all logo abaixo normaliza QUALQUER
		// erro (inclusive o TypeError de acessar `.i` num envelope null/undefined) pra um
		// InvalidConnectionCodeError idêntico — trocar `?.` por `.` aqui não muda o que a API
		// pública observa.
		if (!envelope?.i || typeof envelope.a !== 'string') {
			throw new InvalidConnectionCodeError();
		}

		return {
			id: { id: envelope.i, device_id: envelope.d ?? null },
			addrs: base64ToBytes(envelope.a)
		};
	} catch {
		throw new InvalidConnectionCodeError();
	}
}

/** Trunca um id longo (peer id, device id) pra exibição: `909b713a…6511a8`. */
export function shortId(id: string, keepStart = 8, keepEnd = 6): string {
	if (id.length <= keepStart + keepEnd + 1) return id;
	return `${id.slice(0, keepStart)}…${id.slice(-keepEnd)}`;
}
