package br.acerola.comic.usecase.network

import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.service.SyncDirection
import javax.inject.Inject

/**
 * Resolves a paired peer's address by id and fires an `acerola/sync-comic/1` session for a
 * single comic, in the explicit [SyncDirection] the caller chose — shared by both the push entry
 * point (user picks a comic they already have) and the pull entry point (user picks a comic
 * discovered by browsing a peer's library).
 *
 * [chapters] (chapter labels, empty by default = whole comic) scopes the session to a subset of
 * chapters — used by the chapter-selection/per-chapter "Send" flows, which send the actual
 * file(s) instead of just reading progress.
 */
class SyncComicWithPeerUseCase
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
    ) {
        suspend operator fun invoke(
            peerId: String,
            comicName: String,
            direction: SyncDirection,
            chapters: List<String> = emptyList(),
        ): SyncWithPeerResult {
            val peerAddress = p2pUseCase.getPairedPeers().find { it.id == peerId }
            if (peerAddress == null) {
                AcerolaLogger.w("SyncComicWithPeerUseCase", "Peer not paired: $peerId", LogSource.NETWORK)
                return SyncWithPeerResult.NOT_PAIRED
            }

            val started = p2pUseCase.syncComic(peerAddress, comicName, direction, chapters)
            return if (started) SyncWithPeerResult.STARTED else SyncWithPeerResult.DECLINED_MOBILE_DATA
        }
    }
