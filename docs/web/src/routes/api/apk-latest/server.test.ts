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

describe('GET /api/apk-latest', () => {
	it('redirects to the only APK when latest/ has a single object', async () => {
		await expect(
			GET(fakeEvent([{ key: 'android/latest/acerola-1.0.15.apk' }]))
		).rejects.toMatchObject({
			status: 302,
			location: 'https://binary.acerola-comic.com/android/latest/acerola-1.0.15.apk'
		});
	});

	it('picks the highest semantic version when more than one APK lingers in latest/', async () => {
		await expect(
			GET(
				fakeEvent([
					{ key: 'android/latest/acerola-1.0.9.apk' },
					{ key: 'android/latest/acerola-1.0.15.apk' },
					{ key: 'android/latest/acerola-1.2.0.apk' }
				])
			)
		).rejects.toMatchObject({
			status: 302,
			location: 'https://binary.acerola-comic.com/android/latest/acerola-1.2.0.apk'
		});
	});

	it('ignores keys that do not match the expected filename pattern', async () => {
		await expect(
			GET(
				fakeEvent([
					{ key: 'android/latest/README.txt' },
					{ key: 'android/latest/acerola-1.0.15.apk' }
				])
			)
		).rejects.toMatchObject({
			status: 302,
			location: 'https://binary.acerola-comic.com/android/latest/acerola-1.0.15.apk'
		});
	});

	it('returns 404 when the latest/ folder is empty', async () => {
		await expect(GET(fakeEvent([]))).rejects.toMatchObject({ status: 404 });
	});

	it('returns 500 when the R2 binding is not configured', async () => {
		await expect(GET(fakeEvent(undefined))).rejects.toMatchObject({ status: 500 });
	});
});
