import { error, redirect } from '@sveltejs/kit';
import { APK_BINARY_BASE_URL } from '$lib/constants/site';
import type { RequestHandler } from './$types';

// android-release.yml sobe cada release prod pra `android/releases/acerola-{version}.apk`
// (arquivo permanente) e também pra `android/latest/acerola-{version}.apk`, sem apagar a
// versão anterior dessa segunda pasta. `latest/` pode então ter mais de um APK por um
// tempo — resolve sempre a maior versão em vez de assumir uma chave fixa tipo `latest.apk`.
const LATEST_PREFIX = 'android/latest/';
const KEY_PATTERN = /^android\/latest\/acerola-(\d+\.\d+\.\d+)\.apk$/;

function compareVersions(a: string, b: string): number {
	const partsA = a.split('.').map(Number);
	const partsB = b.split('.').map(Number);

	for (let i = 0; i < 3; i++) {
		if (partsA[i] !== partsB[i]) return partsA[i] - partsB[i];
	}

	return 0;
}

export const GET: RequestHandler = async ({ platform }) => {
	const bucket = platform?.env?.APK_BUCKET;
	if (!bucket) error(500, 'APK bucket not configured');

	const listed = await bucket.list({ prefix: LATEST_PREFIX });
	const versions = listed.objects
		.map((object) => object.key.match(KEY_PATTERN)?.[1])
		.filter((version): version is string => version !== undefined);

	if (versions.length === 0) error(404, 'No APK available');

	const latest = versions.sort(compareVersions).at(-1) as string;
	redirect(302, `${APK_BINARY_BASE_URL}/${LATEST_PREFIX}acerola-${latest}.apk`);
};
