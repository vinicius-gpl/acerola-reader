package br.acerola.comic.module.comic.template

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.LaunchedEffect
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.dto.ChapterDto
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.module.comic.component.ChapterItem
import br.acerola.comic.module.comic.component.CoverVolumeSectionHeader
import br.acerola.comic.module.comic.component.VolumeSectionHeader

fun Comic.Template.chapterSection(
    scope: LazyListScope,
    chapters: ChapterDto,
    currentPage: Int,
    totalPages: Int,
    readChapters: List<String> = emptyList(),
    volumeViewMode: VolumeViewType = VolumeViewType.CHAPTER,
    activeVolumeId: Long? = null,
    selectedChapterSorts: Set<String> = emptySet(),
    isSelectionMode: Boolean = false,
    sendingChapterSorts: Set<String> = emptySet(),
    sendChaptersVisualState: SyncActionVisualState = SyncActionVisualState.IDLE,
    onChapterClick: (ChapterFileDto) -> Unit,
    onToggleRead: (String) -> Unit,
    onToggleSelection: (String) -> Unit = {},
    onLongPressChapter: (String) -> Unit = {},
    onSendToPeer: (String) -> Unit = {},
    onPageChange: (Int) -> Unit,
    onSetActiveVolume: (Long?) -> Unit = {},
    onLoadVolumeChaptersPage: (Long, Int) -> Unit = { _, _ -> },
    onExtractVolumeCover: (Long) -> Unit = {},
) {
    val useVolumeSections =
        (volumeViewMode == VolumeViewType.VOLUME || volumeViewMode == VolumeViewType.COVER_VOLUME) &&
            chapters.archive.volumeSections.isNotEmpty()

    if (useVolumeSections) {
        val sortedSections = chapters.archive.volumeSections

        sortedSections.forEach { group ->
            val isExpanded = activeVolumeId == group.volume.id
            val onToggleExpanded = { onSetActiveVolume(if (isExpanded) null else group.volume.id) }

            scope.item(
                key = "vol_${group.volume.id}",
                contentType = "volume_card",
            ) {
                if (volumeViewMode == VolumeViewType.COVER_VOLUME) {
                    Comic.Component.CoverVolumeSectionHeader(
                        group = group,
                        expanded = isExpanded,
                        onToggleExpanded = onToggleExpanded,
                        onExtractCover = { onExtractVolumeCover(group.volume.id) },
                    )
                } else {
                    Comic.Component.VolumeSectionHeader(
                        group = group,
                        expanded = isExpanded,
                        onToggleExpanded = onToggleExpanded,
                    )
                }
            }

            if (isExpanded) {
                group.items.forEachIndexed { index, chapter ->
                    scope.item(
                        key = "vol_${group.volume.id}_ch_${chapter.id}_$index",
                        contentType = "chapter_item",
                    ) {
                        Comic.Component.ChapterItem(
                            chapterFileDto = chapter,
                            isRead = readChapters.contains(chapter.chapterSort),
                            isSelected = selectedChapterSorts.contains(chapter.chapterSort),
                            isSelectionMode = isSelectionMode,
                            onClick = {
                                if (isSelectionMode) onToggleSelection(chapter.chapterSort) else onChapterClick(chapter)
                            },
                            onLongClick = { onLongPressChapter(chapter.chapterSort) },
                            onToggleRead = { onToggleRead(chapter.chapterSort) },
                            onSendToPeer = { onSendToPeer(chapter.chapterSort) },
                            sendState =
                                if (chapter.chapterSort in sendingChapterSorts) {
                                    sendChaptersVisualState
                                } else {
                                    SyncActionVisualState.IDLE
                                },
                        )
                    }
                }

                if (group.hasMore) {
                    scope.item(key = "vol_load_more_${group.volume.id}") {
                        LaunchedEffect(group.currentPage) {
                            onLoadVolumeChaptersPage(group.volume.id, group.currentPage + 1)
                        }
                    }
                }
            }
        }
    } else {
        chapters.archive.items.forEachIndexed { index, chapter ->
            scope.item(
                key = "ch_${chapter.id}_$index",
                contentType = "chapter_item",
            ) {
                Comic.Component.ChapterItem(
                    chapterFileDto = chapter,
                    isRead = readChapters.contains(chapter.chapterSort),
                    isSelected = selectedChapterSorts.contains(chapter.chapterSort),
                    isSelectionMode = isSelectionMode,
                    onClick = {
                        if (isSelectionMode) onToggleSelection(chapter.chapterSort) else onChapterClick(chapter)
                    },
                    onLongClick = { onLongPressChapter(chapter.chapterSort) },
                    onToggleRead = { onToggleRead(chapter.chapterSort) },
                    onSendToPeer = { onSendToPeer(chapter.chapterSort) },
                )
            }
        }

        if (currentPage + 1 < totalPages) {
            scope.item(key = "flat_load_more") {
                LaunchedEffect(currentPage) {
                    onPageChange(currentPage + 1)
                }
            }
        }
    }
}
