// Só o método que os resolvers de /api/apk-latest e /api/apk-status usam — evita puxar
// @cloudflare/workers-types inteiro por causa de um binding só.
export interface ApkBucket {
	list(options: { prefix: string }): Promise<{ objects: { key: string }[] }>;
}

// android-release.yml sobe cada release prod pra `android/releases/acerola-{version}.apk`
// (arquivo permanente) e também pra `android/latest/acerola-{version}.apk`, sem apagar a
// versão anterior dessa segunda pasta. `latest/` pode então ter mais de um APK por um
// tempo — resolve sempre a maior versão em vez de assumir uma chave fixa tipo `latest.apk`.
export const LATEST_PREFIX = 'android/latest/';
const KEY_PATTERN = /^android\/latest\/acerola-(\d+\.\d+\.\d+)\.apk$/;

function compareVersions(a: string, b: string): number {
	const partsA = a.split('.').map(Number);
	const partsB = b.split('.').map(Number);

	for (let i = 0; i < 3; i++) {
		if (partsA[i] !== partsB[i]) return partsA[i] - partsB[i];
	}

	return 0;
}

// Retorna a chave do APK de maior versão em `android/latest/`, ou null se a pasta
// estiver vazia (ou só tiver arquivos que não casam com o padrão de nome esperado).
export async function resolveLatestApkKey(bucket: ApkBucket): Promise<string | null> {
	const listed = await bucket.list({ prefix: LATEST_PREFIX });
	const versions = listed.objects
		.map((object) => object.key.match(KEY_PATTERN)?.[1])
		.filter((version): version is string => version !== undefined);

	if (versions.length === 0) return null;

	const latest = versions.sort(compareVersions).at(-1) as string;
	return `${LATEST_PREFIX}acerola-${latest}.apk`;
}
