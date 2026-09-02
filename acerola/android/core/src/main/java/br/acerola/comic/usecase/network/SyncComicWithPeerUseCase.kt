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
 */
class SyncComicWithPeerUseCase
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
    ) {
        /** Returns `false` if `peerId` isn't currently paired (nothing was fired). */
        operator fun invoke(
            peerId: String,
            comicName: String,
            direction: SyncDirection,
        ): Boolean {
            val peerAddress = p2pUseCase.getPairedPeers().find { it.id == peerId }
            if (peerAddress == null) {
                AcerolaLogger.w("SyncComicWithPeerUseCase", "Peer not paired: $peerId", LogSource.NETWORK)
                return false
            }

            p2pUseCase.syncComic(peerAddress, comicName, direction)
            return true
        }
    }
