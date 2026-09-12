import '@poppanator/sveltekit-svg/dist/svg.d.ts';

declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		interface Platform {
			env?: {
				// Só o método que o resolver de /api/apk-latest usa — evita puxar
				// @cloudflare/workers-types inteiro por causa de um binding só.
				APK_BUCKET?: {
					list(options: { prefix: string }): Promise<{ objects: { key: string }[] }>;
				};
			};
		}
	}
}

export {};
