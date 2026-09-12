import { error, redirect } from '@sveltejs/kit';
import { APK_BINARY_BASE_URL } from '$lib/constants/site';
import { resolveLatestApkKey } from '$lib/server/apk-bucket';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ platform }) => {
	const bucket = platform?.env?.APK_BUCKET;
	if (!bucket) error(500, 'APK bucket not configured');

	const key = await resolveLatestApkKey(bucket);
	if (key === null) error(404, 'No APK available');

	redirect(302, `${APK_BINARY_BASE_URL}/${key}`);
};
