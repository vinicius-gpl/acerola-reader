export type DeviceInfoPayload = {
	name: string;
	os: string;
	version: string;
};

export type ConnectedPeerPayload = {
	peerId: string;
	alpn: string;
	device: DeviceInfoPayload | null;
};

export type NetworkStatusPayload = {
	mode: 'local' | 'relay';
	peers: ConnectedPeerPayload[];
};

/** Peer já pareado alguma vez, com o último endereço conhecido — sobrevive a restart e
 *  independe de estar conectado agora (ver `NetworkServiceApi::paired_peers` no backend).
 *  `deviceName` vem de `known_peers()` (persistente) — `null` só quando esse peer nunca
 *  respondeu a entrevista de identidade (DeviceInfo). */
export type PairedPeerPayload = {
	peerId: string;
	addrs: number[];
	deviceName: string | null;
};

/** Configuração de relay combinável, lida de `settings.json` (ver `RelaySettings` no
 *  backend). `acerolaRelayUrl` é fixo, só pra exibir mesmo quando `useAcerolaRelay` é
 *  `false`. Trocar qualquer campo só tem efeito no próximo início do app. */
export type RelayInfo = {
	acerolaRelayUrl: string;
	useAcerolaRelay: boolean;
	useIrohPublicNetwork: boolean;
	customRelayUrls: string[];
	/** Só indica SE um ticket da conta do usuário em `services.iroh.computer` já foi colado e
	 *  salvo — o valor em si nunca é devolvido pro frontend (é uma credencial real, guardada
	 *  no cofre criptografado, não em `settings.json`). */
	hasIrohServicesTicket: boolean;
};

/** Resumo de um quadrinho da biblioteca remota (ver `library_browse_handler.rs`) — só título +
 *  contagem de capítulos, sem transferir nada ainda. `coverVersion` reaproveita
 *  `comic_directory.last_modified` do peer — usado pra decidir se `queryRemoteCover` precisa
 *  buscar uma capa nova antes de disparar a busca. */
export type ComicSummary = {
	comicName: string;
	chapterCount: number;
	coverVersion: number;
};

/** Payload do evento `library:query:result`. */
export type LibraryQueryResultPayload = {
	peerId: string;
	comics: ComicSummary[];
};

/** Payload do evento `browse:cover:result` (`cover_browse_handler.rs`, ALPN
 *  `acerola/browse-cover/1`). `path` só vem preenchido quando `status === 'changed'` — caminho
 *  local (`<app_data_dir>/remote_covers/...`) já resolvido via `convertFileSrc` do lado do
 *  chamador (ver `resolveArtworkPath`, `artwork.utils.ts`). */
export type CoverQueryResultPayload = {
	peerId: string;
	comicName: string;
	status: 'not_modified' | 'changed' | 'unavailable';
	coverVersion: number | null;
	path: string | null;
};

export type CoverQueryErrorPayload = {
	peerId: string;
	comicName?: string;
	message: string;
};
