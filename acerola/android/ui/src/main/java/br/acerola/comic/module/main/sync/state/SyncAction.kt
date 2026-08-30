package br.acerola.comic.module.main.sync.state

sealed interface SyncAction {
    /** Sets a custom local device alias (LocalSend-style) — applies immediately on the P2P
     *  node (next handshake already uses it, no restart needed) and persists it for future
     *  launches. */
    data class RenameDevice(
        val name: String,
    ) : SyncAction

    /** Decodes a code (pasted or scanned) and proposes pairing for confirmation. */
    data class ProposeConnect(
        val code: String,
    ) : SyncAction

    data object ConfirmConnect : SyncAction

    data object CancelConnect : SyncAction

    data class SyncHistory(
        val peerId: String,
    ) : SyncAction

    data class SyncFiles(
        val peerId: String,
    ) : SyncAction

    data class SyncAll(
        val peerId: String,
    ) : SyncAction

    data object DismissTrustDialog : SyncAction

    data object DismissConnectError : SyncAction

    data class RemovePeer(
        val peerId: String,
    ) : SyncAction

    /** Pede a lista de quadrinhos do peer (nome + contagem de capítulos), sem sincronizar
     *  nada — abre o [br.acerola.comic.module.main.sync.RemoteLibrarySheet]. */
    data class BrowseLibrary(
        val peerId: String,
    ) : SyncAction

    data object DismissLibraryBrowse : SyncAction

    /** Sincroniza um único quadrinho (descoberto via [BrowseLibrary]) com o peer. */
    data class SyncComic(
        val peerId: String,
        val comicName: String,
    ) : SyncAction
}
