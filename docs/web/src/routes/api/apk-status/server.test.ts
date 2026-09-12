import { describe, expect, it, vi } from 'vitest';
import { GET } from './+server';

function fakeEvent(fetchImpl: typeof fetch) {
	return { fetch: fetchImpl } as Parameters<typeof GET>[0];
}

describe('GET /api/apk-status', () => {
	it('reports available when the HEAD request succeeds', async () => {
		const fetchMock = vi.fn().mockResolvedValue({ ok: true });
		const response = await GET(fakeEvent(fetchMock as unknown as typeof fetch));

		expect(await response.json()).toEqual({ available: true });
		expect(fetchMock).toHaveBeenCalledWith(
			'/api/apk-latest',
			expect.objectContaining({ method: 'HEAD' })
		);
	});

	it('reports unavailable when the HEAD request returns a non-2xx status (e.g. 404)', async () => {
		const fetchMock = vi.fn().mockResolvedValue({ ok: false, status: 404 });
		const response = await GET(fakeEvent(fetchMock as unknown as typeof fetch));

		expect(await response.json()).toEqual({ available: false });
	});

	it('reports unavailable when the request itself throws (timeout, DNS, connection refused)', async () => {
		const fetchMock = vi.fn().mockRejectedValue(new Error('network error'));
		const response = await GET(fakeEvent(fetchMock as unknown as typeof fetch));

		expect(await response.json()).toEqual({ available: false });
		// Duas tentativas antes de desistir — não uma só.
		expect(fetchMock).toHaveBeenCalledTimes(2);
	});

	it('retries once and reports available when only the first attempt throws (cold-start pattern)', async () => {
		const fetchMock = vi
			.fn()
			.mockRejectedValueOnce(new Error('network error'))
			.mockResolvedValueOnce({ ok: true });
		const response = await GET(fakeEvent(fetchMock as unknown as typeof fetch));

		expect(await response.json()).toEqual({ available: true });
		expect(fetchMock).toHaveBeenCalledTimes(2);
	});

	it('does not retry a definitive non-2xx HTTP response (e.g. 404) — only network-level failures', async () => {
		const fetchMock = vi.fn().mockResolvedValue({ ok: false, status: 404 });
		const response = await GET(fakeEvent(fetchMock as unknown as typeof fetch));

		expect(await response.json()).toEqual({ available: false });
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});
});
