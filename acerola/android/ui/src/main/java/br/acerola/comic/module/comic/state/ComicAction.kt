package br.acerola.comic.module.comic.state

import br.acerola.comic.config.preference.types.ChapterPageSizeType
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.service.SyncDirection

sealed interface ComicAction {
    data object NavigateBack : ComicAction

    data class SelectTab(
        val tab: MainTab,
    ) : ComicAction

    data class UpdatePageSize(
        val size: ChapterPageSizeType,
    ) : ComicAction

    data class UpdateCategory(
        val categoryId: Long?,
    ) : ComicAction

    data class ToggleExternalSync(
        val enabled: Boolean,
    ) : ComicAction

    data class UpdateVolumeView(
        val mode: VolumeViewType,
    ) : ComicAction

    data object ClearMetadata : ComicAction
}

sealed interface ComicChapterAction {
    data class ChangePage(
        val page: Int,
    ) : ComicChapterAction

    data class ClickChapter(
        val chapter: ChapterFileDto,
        val initialPage: Int,
    ) : ComicChapterAction

    data class ClickContinue(
        val chapterId: Long?,
        val lastPage: Int,
    ) : ComicChapterAction

    data class ToggleReadStatus(
        val chapterSort: String,
    ) : ComicChapterAction
}

sealed interface ComicSyncAction {
    data object SyncChaptersLocal : ComicSyncAction

    data object RescanComic : ComicSyncAction

    data object SyncMangadexInfo : ComicSyncAction

    data object SyncComicInfo : ComicSyncAction

    data object SyncAnilistInfo : ComicSyncAction

    data object ExtractFirstPageAsCover : ComicSyncAction

    data object ExtractVolumeCovers : ComicSyncAction

    data class SyncWithPeer(
        val peerId: String,
        val direction: SyncDirection,
    ) : ComicSyncAction
}
