package br.acerola.comic.module.main.history.state
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.history.ReadingHistoryWithChapterDto

sealed interface HistoryAction {
    data class ClickManga(
        val comic: ComicDto,
    ) : HistoryAction

    data class ClickContinue(
        val comic: ComicDto,
        val history: ReadingHistoryWithChapterDto,
    ) : HistoryAction

    data object LoadPairedPeersForSync : HistoryAction

    data class SyncHistoryWithPeer(
        val peerId: String,
    ) : HistoryAction
}
