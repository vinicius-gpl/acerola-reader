export const NETWORK_EVENTS = {
	status: 'network:status',
	/** Emitidos pela lib `acerola_p2p` (não pelo Acerola) no exato momento em que um
	 *  handshake de conexão termina — `deviceInfoReceived` do lado que conectou,
	 *  `deviceInfoExchanged` do lado que aceitou. `network:status` nunca é emitido sozinho
	 *  quando isso acontece (só quando o comando `get_network_status` é chamado
	 *  explicitamente), então sem escutar esses dois a UI fica parada até um refresh manual
	 *  (F5) mesmo com um peer já conectado. */
	deviceInfoReceived: 'rpc:device_info_received',
	deviceInfoExchanged: 'rpc:device_info_exchanged',
	historyStarted: 'sync:history:started',
	historyComplete: 'sync:history:complete',
	historyError: 'sync:history:error',
	filesStarted: 'sync:files:started',
	filesProgress: 'sync:files:progress',
	filesComplete: 'sync:files:complete',
	filesError: 'sync:files:error',
	historyEntryStarted: 'sync:history-entry:started',
	historyEntryComplete: 'sync:history-entry:complete',
	historyEntryError: 'sync:history-entry:error',
	comicStarted: 'sync:comic:started',
	comicProgress: 'sync:comic:progress',
	comicComplete: 'sync:comic:complete',
	comicError: 'sync:comic:error',
	/** Resultado de `queryRemoteLibrary` — carrega `{ peerId, comics }` (ver
	 *  `LibraryQueryResultPayload`), emitido pelo `LibraryBrowseOutbound` no backend
	 *  (`infra::sync::protocol::library_browse_handler`, ALPN `acerola/browse-library/1`,
	 *  mesmo ALPN usado pelo Android). */
	libraryQueryResult: 'library:query:result',
	libraryQueryError: 'library:query:error',
	/** Resultado de `queryRemoteCover` (`{ peerId, comicName, status, coverVersion, path }`,
	 *  ver `CoverQueryResultPayload`) — `status` é `"not_modified"`, `"changed"` (com `path`
	 *  apontando pro cache local recém-gravado, resolvido via `convertFileSrc`) ou
	 *  `"unavailable"`. Emitido pelo `CoverBrowseOutbound` (`cover_browse_handler.rs`, ALPN
	 *  `acerola/browse-cover/1`). */
	coverQueryResult: 'browse:cover:result',
	coverQueryError: 'browse:cover:error',
	/** Emitido uma vez na inicialização se o keyring do SO não estiver disponível (ver
	 *  `infra::security::MasterKeySource::FallbackFile` no backend) — a chave mestra caiu
	 *  pra um arquivo local sem a proteção extra do SO. Não é um erro fatal (tudo continua
	 *  funcionando, ainda criptografado com AES-GCM), mas o usuário precisa saber. */
	keyringUnavailable: 'security:keyring_unavailable'
} as const;
