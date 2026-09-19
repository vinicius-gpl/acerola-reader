// Estado global espelhando se um sync em lote de metadados (MangaDex/AniList, tela de Config)
// está rodando agora — permite que `useMetadataSync()` (usado na tela do quadrinho) desabilite
// o botão de sync individual enquanto o lote roda, em vez do usuário clicar e só descobrir via
// toast de `SyncInProgress` que o comando foi recusado (o lock em si já existe no backend,
// isso é só a UI antecipar o estado).
let syncingAll = $state(false);

export function setSyncingAll(value: boolean): void {
	syncingAll = value;
}

export function isSyncingAll(): boolean {
	return syncingAll;
}
