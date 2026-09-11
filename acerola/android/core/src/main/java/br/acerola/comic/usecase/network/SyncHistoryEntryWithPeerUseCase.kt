package br.acerola.comic.usecase.network

import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import javax.inject.Inject

/**
 * Resolves a paired peer's address by id and fires an `acerola/sync-history-entry/1` push for
 * the given comic, restricted to `chapterSorts` — used by the chapter selection "send to peer"
 * action, which can cover a single chapter or several selected at once.
 */
class SyncHistoryEntryWithPeerUseCase
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
    ) {
        /** Returns `false` if `peerId` isn't currently paired (nothing was fired). */
        operator fun invoke(
            peerId: String,
            comicName: String,
            chapterSorts: List<String>,
        ): Boolean {
            val peerAddress = p2pUseCase.getPairedPeers().find { it.id == peerId }
            if (peerAddress == null) {
                AcerolaLogger.w("SyncHistoryEntryWithPeerUseCase", "Peer not paired: $peerId", LogSource.NETWORK)
                return false
            }

            p2pUseCase.syncHistoryEntry(peerAddress, comicName, chapterSorts)
            return true
        }
    }
