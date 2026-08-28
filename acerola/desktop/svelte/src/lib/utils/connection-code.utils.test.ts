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
