// Contador global que qualquer tela lendo `resolveCover`/`resolveBanner` (`artwork.utils.ts`)
// usa como cache-bust automático. Sem isso, cada tela precisava lembrar de setar seu próprio
// timestamp local toda vez que uma operação pudesse ter baixado uma capa nova (sync MangaDex/
// AniList, sync em lote, regenerar capa, sync P2P) — na prática várias esqueciam (ex.: sync
// individual via MangaDex/AniList nunca bustava, a Home nunca bustava em lugar nenhum), e a
// imagem antiga continuava em cache do WebView mesmo com o arquivo já sobrescrito no disco.
let version = $state(0);

export function bumpArtworkVersion(): void {
	version = Date.now();
}

export function getArtworkVersion(): number {
	return version;
}
