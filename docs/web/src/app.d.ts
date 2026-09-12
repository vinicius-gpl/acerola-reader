import '@poppanator/sveltekit-svg/dist/svg.d.ts';
import type { ApkBucket } from '$lib/server/apk-bucket';

declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		interface Platform {
			env?: {
				APK_BUCKET?: ApkBucket;
			};
		}
	}
}

export {};
