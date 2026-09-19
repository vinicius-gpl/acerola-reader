package br.acerola.comic.usecase.network

import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import javax.inject.Inject

/** Must match `FILE_SYNC_ALPN` in `SyncViewModel` — no native method exposes a whole-library
 *  file sync directly (unlike [P2pUseCase.syncComic], scoped to one comic), so the ALPN has to
 *  be supplied by the Kotlin caller via the generic [P2pUseCase.connect]. */
private const val FILE_SYNC_ALPN = "acerola/sync-files/1"

/**
 * Resolves a paired peer's address by id and fires an `acerola/sync-files/1` session for the
 * WHOLE library — shared by the Sync screen's "sync all" action and the Home screen's own
 * "sync everything this peer has" entry point, since both trigger the exact same protocol
 * handshake.
 */
class SyncFilesWithPeerUseCase
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
    ) {
        suspend operator fun invoke(peerId: String): SyncWithPeerResult {
            val peerAddress = p2pUseCase.getPairedPeers().find { it.id == peerId }
            if (peerAddress == null) {
                AcerolaLogger.w("SyncFilesWithPeerUseCase", "Peer not paired: $peerId", LogSource.NETWORK)
                return SyncWithPeerResult.NOT_PAIRED
            }

            val started = p2pUseCase.connect(peerAddress, FILE_SYNC_ALPN.toByteArray())
            return if (started) SyncWithPeerResult.STARTED else SyncWithPeerResult.DECLINED_MOBILE_DATA
        }
    }
