import { describe, expect, it } from 'vitest';
import {
	decodeConnectionCode,
	encodeConnectionCode,
	InvalidConnectionCodeError,
	shortId,
	type LocalPeerAddr
} from './connection-code.utils';

describe('encodeConnectionCode / decodeConnectionCode', () => {
	it('round-trips a local peer addr through encode/decode', () => {
		const addr: LocalPeerAddr = {
			id: { id: 'peer-123', device_id: 'device-abc' },
			addrs: [1, 2, 3, 255, 0]
		};

		const code = encodeConnectionCode(addr);
		const decoded = decodeConnectionCode(code);

		expect(decoded).toEqual(addr);
		// Checagem precisa e independente do toEqual sobre o array de bytes decodificado:
		// tamanho exato e conteúdo exato por índice pra uma entrada conhecida, incluindo
		// valores de fronteira 0 e 255.
		expect(decoded.addrs).toHaveLength(5);
		expect(decoded.addrs).toEqual([1, 2, 3, 255, 0]);
		expect(Array.isArray(decoded.addrs)).toBe(true);
	});

	it('round-trips when device_id is null', () => {
		const addr: LocalPeerAddr = {
			id: { id: 'peer-456', device_id: null },
			addrs: [42]
		};

		const code = encodeConnectionCode(addr);
		const decoded = decodeConnectionCode(code);

		expect(decoded).toEqual(addr);
	});

	it('produces a code prefixed with acerola1:', () => {
		const code = encodeConnectionCode({ id: { id: 'x', device_id: null }, addrs: [] });

		expect(code.startsWith('acerola1:')).toBe(true);
	});

	it('trims surrounding whitespace before decoding', () => {
		const addr: LocalPeerAddr = { id: { id: 'peer-789', device_id: null }, addrs: [9, 8] };
		const code = `  ${encodeConnectionCode(addr)}  `;

		expect(decodeConnectionCode(code)).toEqual(addr);
	});

	it('throws InvalidConnectionCodeError for a malformed prefix', () => {
		expect(() => decodeConnectionCode('not-a-valid-prefix:abcd')).toThrow(
			InvalidConnectionCodeError
		);
	});

	it('rejects a wrong prefix even when the remaining payload would otherwise decode fine', () => {
		// Um payload base64/JSON malformado acaba normalizado em InvalidConnectionCodeError
		// pelo catch-all mais abaixo independente do check de prefixo, então um teste como o
		// de cima (prefixo errado + payload lixo) não distingue "o guard do prefixo lançou" de
		// "o guard foi pulado e o JSON.parse/atob que explodiu". Pra provar de verdade que é o
		// guard do prefixo que rejeita o código, o payload depois do prefixo falso (do mesmo
		// tamanho) precisa ser um envelope *válido* — um que decodificaria com sucesso se o
		// guard fosse pulado. Se o guard for removido/curto-circuitado, isso retorna um
		// LocalPeerAddr válido em vez de lançar.
		const addr: LocalPeerAddr = { id: { id: 'peer-x', device_id: null }, addrs: [1, 2, 3] };
		const validCode = encodeConnectionCode(addr);
		const realPrefixLength = 'acerola1:'.length;
		const payload = validCode.slice(realPrefixLength);
		const wrongPrefixCode = `${'x'.repeat(realPrefixLength)}${payload}`;

		expect(wrongPrefixCode.startsWith('acerola1:')).toBe(false);
		expect(() => decodeConnectionCode(wrongPrefixCode)).toThrow(InvalidConnectionCodeError);
	});

	it('throws InvalidConnectionCodeError for malformed base64 payload', () => {
		expect(() => decodeConnectionCode('acerola1:###not-base64###')).toThrow(
			InvalidConnectionCodeError
		);
	});

	it('throws InvalidConnectionCodeError when envelope is missing required fields', () => {
		const badEnvelope = btoa(JSON.stringify({ d: null }));

		expect(() => decodeConnectionCode(`acerola1:${badEnvelope}`)).toThrow(
			InvalidConnectionCodeError
		);
	});

	it('throws InvalidConnectionCodeError when i is missing even if a is a valid string', () => {
		const badEnvelope = btoa(JSON.stringify({ d: null, a: 'validbase64' }));

		expect(() => decodeConnectionCode(`acerola1:${badEnvelope}`)).toThrow(
			InvalidConnectionCodeError
		);
	});

	it('throws InvalidConnectionCodeError when i is present but a is not a string', () => {
		const badEnvelope = btoa(JSON.stringify({ i: 'peer-1', d: null, a: 123 }));

		expect(() => decodeConnectionCode(`acerola1:${badEnvelope}`)).toThrow(
			InvalidConnectionCodeError
		);
	});

	it('throws InvalidConnectionCodeError (not a raw TypeError) when the envelope itself is null', () => {
		const nullEnvelope = btoa(JSON.stringify(null));

		expect(() => decodeConnectionCode(`acerola1:${nullEnvelope}`)).toThrow(
			InvalidConnectionCodeError
		);
	});

	it('the thrown error carries the invalid_connection_code message', () => {
		expect(() => decodeConnectionCode('')).toThrow('invalid_connection_code');
	});

	it('throws InvalidConnectionCodeError for an empty string', () => {
		expect(() => decodeConnectionCode('')).toThrow(InvalidConnectionCodeError);
	});
});

describe('shortId', () => {
	it('truncates a long id keeping start and end segments', () => {
		const id = '909b713a1234567890abcdef6511a8';

		expect(shortId(id)).toBe(`${id.slice(0, 8)}…${id.slice(-6)}`);
	});

	it('returns the id unchanged when shorter than keepStart + keepEnd + 1', () => {
		expect(shortId('short')).toBe('short');
	});

	it('returns the id unchanged at the exact boundary length', () => {
		const id = 'a'.repeat(8 + 6 + 1);

		expect(shortId(id)).toBe(id);
	});

	it('truncates when just one character over the boundary', () => {
		const id = 'a'.repeat(8 + 6 + 2);

		expect(shortId(id)).toBe(`${id.slice(0, 8)}…${id.slice(-6)}`);
	});

	it('honors custom keepStart/keepEnd values', () => {
		const id = 'abcdefghijklmnop';

		expect(shortId(id, 2, 2)).toBe('ab…op');
	});

	it('handles empty string without throwing', () => {
		expect(shortId('')).toBe('');
	});
});
