import { describe, expect, it } from 'vitest';
import { extractErrorMessage } from './error.utils';
import type { ErrorPayload } from '$lib/contracts/shared/shared.payloads';

describe('extractErrorMessage', () => {
	it('returns a fallback message for null', () => {
		expect(extractErrorMessage(null)).toBe('Unknown error occurred');
	});

	it('returns a fallback message for undefined', () => {
		expect(extractErrorMessage(undefined)).toBe('Unknown error occurred');
	});

	it('returns a plain string unchanged', () => {
		expect(extractErrorMessage('plain error text')).toBe('plain error text');
	});

	it('extracts the message from an Error instance', () => {
		expect(extractErrorMessage(new Error('boom'))).toBe('boom');
	});

	it('extracts the message from an ErrorPayload-shaped object', () => {
		const payload: ErrorPayload = { errorType: 'IoError', message: 'disk full' };

		expect(extractErrorMessage(payload)).toBe('disk full');
	});

	it('falls back to the raw (blank) message field when it is only whitespace', () => {
		// NOTE: pre-existing behavior — the blank-message guard only skips the FIRST check;
		// the second check accepts any string type regardless of content, so a whitespace-only
		// `message` field is returned as-is instead of falling back to String(error).
		const payload = { errorType: 'IoError', message: '   ' };

		expect(extractErrorMessage(payload)).toBe('   ');
	});

	it('falls back to String(error) for a plain object without a message field', () => {
		const payload = { errorType: 'IoError' };

		expect(extractErrorMessage(payload)).toBe(String(payload));
	});

	it('falls back to String(error) when message is present but not a string', () => {
		// Distingue os dois guards de tipo (`typeof payload.message === 'string'` na 1ª
		// checagem e na de fallback) de uma versão que aceitasse qualquer valor: com
		// `message` sendo um número, nenhuma das duas deveria "vazar" o valor cru.
		const payload = { errorType: 'IoError', message: 42 };

		expect(extractErrorMessage(payload)).toBe(String(payload));
	});

	it('stringifies numbers and other primitives', () => {
		expect(extractErrorMessage(42)).toBe('42');
	});
});
