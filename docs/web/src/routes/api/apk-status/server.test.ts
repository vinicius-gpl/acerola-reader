import { describe, expect, it } from 'vitest';
import { GET } from './+server';

function fakeEvent(objects: { key: string }[] | undefined) {
	return {
		platform:
			objects === undefined
				? undefined
				: { env: { APK_BUCKET: { list: async () => ({ objects }) } } }
	} as Parameters<typeof GET>[0];
}

describe('GET /api/apk-status', () => {
	it('reports available when the latest/ folder has a matching APK', async () => {
		const response = await GET(fakeEvent([{ key: 'android/latest/acerola-1.0.15.apk' }]));

		expect(await response.json()).toEqual({ available: true });
	});

	it('reports unavailable when the latest/ folder is empty', async () => {
		const response = await GET(fakeEvent([]));

		expect(await response.json()).toEqual({ available: false });
	});

	it('reports unavailable when the latest/ folder has no key matching the expected filename pattern', async () => {
		const response = await GET(fakeEvent([{ key: 'android/latest/README.txt' }]));

		expect(await response.json()).toEqual({ available: false });
	});

	it('reports unavailable when the R2 binding is not configured', async () => {
		const response = await GET(fakeEvent(undefined));

		expect(await response.json()).toEqual({ available: false });
	});
});
