package br.acerola.comic.usecase.network

import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import javax.inject.Inject

/** Must match `HISTORY_SYNC_ALPN` in `SyncViewModel` — no native method exposes history sync
 *  directly (unlike [P2pUseCase.syncComic]/[P2pUseCase.browseLibrary]), so the ALPN has to be
 *  supplied by the Kotlin caller via the generic [P2pUseCase.connect]. */
private const val HISTORY_SYNC_ALPN = "acerola/sync-history/1"

/**
 * Resolves a paired peer's address by id and fires an `acerola/sync-history/1` session —
 * shared by the Sync screen's peer menu and the History screen's own sync entry point, since
 * both trigger the exact same protocol handshake.
 */
class SyncHistoryWithPeerUseCase
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
    ) {
        /** Returns `false` if `peerId` isn't currently paired (nothing was fired). */
        operator fun invoke(peerId: String): Boolean {
            val peerAddress = p2pUseCase.getPairedPeers().find { it.id == peerId }
            if (peerAddress == null) {
                AcerolaLogger.w("SyncHistoryWithPeerUseCase", "Peer not paired: $peerId", LogSource.NETWORK)
                return false
            }

            p2pUseCase.connect(peerAddress, HISTORY_SYNC_ALPN.toByteArray())
            return true
        }
    }
