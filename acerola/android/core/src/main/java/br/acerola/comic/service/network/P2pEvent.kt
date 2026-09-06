package br.acerola.comic.service.network

import br.acerola.comic.error.message.SyncProtocolError

/**
 * Typed representation of the raw `(event, data)` events emitted by the Rust P2P node.
 * `data` arrives as a string (usually JSON) whose format depends on the event itself.
 */
sealed interface P2pEvent {
    data class PeerTrustedFirstTime(
        val peerId: String,
    ) : P2pEvent

    /**
     * The handshake (`acerola/handshake/1`) finished exchanging identity/DeviceInfo with this
     * peer — a deterministic signal that "pairing worked", emitted by the library itself
     * (`rpc:device_info_received` on the side that connected, `rpc:device_info_exchanged` on
     * the side that received). Far more reliable than polling the peer list with a timeout.
     */
    data class HandshakeCompleted(
        val peerId: String,
    ) : P2pEvent

    data class HistorySyncStarted(
        val peerId: String,
    ) : P2pEvent

    data class HistorySyncComplete(
        val peerId: String,
        val progressApplied: Int,
        val progressSkipped: Int,
        val chaptersReadApplied: Int,
        val chaptersReadSkipped: Int,
    ) : P2pEvent

    data class HistorySyncError(
        val peerId: String,
        val message: String,
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    data class FileSyncManifestExchanged(
        val peerId: String,
        val missingCount: Int,
        val offeringCount: Int,
    ) : P2pEvent

    data class FileSyncProgress(
        val peerId: String,
        val comicName: String,
        val chapter: String,
        val bytesTransferred: Long,
        val totalBytes: Long,
    ) : P2pEvent

    data class FileSyncChapterComplete(
        val peerId: String,
        val comicName: String,
        val chapter: String,
    ) : P2pEvent

    data class FileSyncChapterFailed(
        val peerId: String,
        val comicName: String,
        val chapter: String,
        // Cru/em inglês sempre — é o texto técnico original (às vezes de baixo nível do
        // QUIC/iroh, tipo "stream reset by peer"), usado só pro log de debug deste bus e como
        // fallback em `error` quando a causa não é reconhecida. Nunca deve ser mostrado direto
        // na UI sem passar por um template (ver `SyncScreen::describeEntry`, mesmo padrão já
        // usado por `HistorySyncError.message`).
        val reason: String,
        // Causa reconhecida (`SyncProtocolError.fromCode`, a partir do `code` que o Rust anexa —
        // ver `classify_sync_error` em `protocol/files/mod.rs`), já como valor tipado — não um
        // `code: String` cru pra quem consome ficar reclassificando com `when`. `null` quando a
        // causa não é reconhecida (usa `reason` cru como fallback). `UserMessage`/`UiText` não
        // precisa de `Context` pra ser construído, só na hora de renderizar (`SyncScreen`).
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    /** Contraparte de [FileSyncChapterComplete] pra itens extra (capa/banner/`ComicInfo.xml`) —
     *  `kind` é `"cover"`/`"banner"`/`"comic_info"` (ver `EXTRA_KIND_*` em
     *  `protocol/files/model.rs`), não um nome de capítulo. */
    data class FileSyncExtraComplete(
        val peerId: String,
        val comicName: String,
        val kind: String,
    ) : P2pEvent

    /** Contraparte de [FileSyncChapterFailed] pra itens extra — mesmo contrato de `reason`/
     *  `error` (ver doc lá: cru/em inglês só pro log, `error` já classificado por
     *  `SyncProtocolError.fromCode`). Não existe sentinela de falha de SESSÃO aqui — isso só
     *  acontece no canal de capítulo (`FileSyncChapterFailed` com `comicName`/`chapter` vazios). */
    data class FileSyncExtraFailed(
        val peerId: String,
        val comicName: String,
        val kind: String,
        val reason: String,
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    data class FileSyncComplete(
        val peerId: String,
        val receivedCount: Int,
        val sentCount: Int,
        val failedCount: Int,
    ) : P2pEvent

    data class LibraryBrowseResult(
        val peerId: String,
        val comics: List<ComicSummary>,
    ) : P2pEvent

    data class LibraryBrowseError(
        val peerId: String,
        val message: String,
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    /** Resultado de `acerola/browse-cover/1` — `status` é `"not_modified"`, `"changed"` (com
     *  `path` apontando pro cache local recém-gravado) ou `"unavailable"` (peer não tem essa
     *  capa). `coverVersion` vem junto pra atualizar o cache local de versão conhecida mesmo
     *  em `not_modified` (confirma que a versão já cacheada continua valendo). */
    data class CoverBrowseResult(
        val peerId: String,
        val comicName: String,
        val status: String,
        val coverVersion: Long?,
        val path: String?,
    ) : P2pEvent

    data class CoverBrowseError(
        val peerId: String,
        val comicName: String,
        val message: String,
    ) : P2pEvent

    data class Unknown(
        val event: String,
        val data: String,
    ) : P2pEvent
}

/** Um quadrinho da biblioteca de um peer remoto — resumido a nome + contagem de capítulos, não
 *  o manifesto completo (checksum/nome de arquivo por capítulo). Usado só pra escolher qual
 *  quadrinho pedir via `SyncAction.SyncComic` depois. `coverVersion` reaproveita
 *  `ComicDirectory.lastModified` do peer — usado pra decidir se `acerola/browse-cover/1` precisa
 *  buscar uma capa nova antes de disparar a busca. */
data class ComicSummary(
    val comicName: String,
    val chapterCount: Int,
    val coverVersion: Long,
)
