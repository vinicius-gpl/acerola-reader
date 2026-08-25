package br.acerola.comic.local.translator.ui

import androidx.core.net.toUri
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.dto.ChapterDto
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.dto.archive.ChapterPageDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.archive.VolumeArchiveDto
import br.acerola.comic.dto.archive.VolumeChapterGroupDto
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.entity.archive.ComicDirectory
import br.acerola.comic.local.entity.archive.VolumeArchive
import br.acerola.comic.local.entity.relation.ChapterVolumeJoin
import br.acerola.comic.local.entity.relation.VolumeChapterCount
import kotlin.math.ceil
import kotlin.math.max

fun ComicDirectory.toViewDto(): ComicDirectoryDto =
    ComicDirectoryDto(
        id = id,
        name = name,
        path = path,
        coverUri = cover?.toUri(),
        bannerUri = banner?.toUri(),
        lastModified = lastModified,
        archiveTemplateFk = archiveTemplateFk,
        externalSyncEnabled = externalSyncEnabled,
        hidden = hidden,
    )

fun VolumeArchive.toViewDto(): VolumeArchiveDto =
    VolumeArchiveDto(
        id = id,
        name = name,
        volumeSort = volumeSort,
        isSpecial = isSpecial,
        coverUri = cover,
        bannerUri = banner,
        lastModified = lastModified,
    )

fun ChapterArchive.toViewDto(volumeName: String? = null): ChapterFileDto =
    ChapterFileDto(
        id = id,
        name = chapter,
        path = path,
        chapterSort = chapterSort,
        volumeId = volumeFk,
        volumeName = volumeName,
        isSpecial = isSpecial,
        lastModified = lastModified,
    )

fun ChapterVolumeJoin.toViewDto(): ChapterFileDto = chapter.toViewDto(volumeName = volume?.name)

fun VolumeChapterCount.toViewDto(): VolumeArchiveDto =
    VolumeArchiveDto(
        id = id,
        name = name,
        volumeSort = volumeSort,
        isSpecial = isSpecial,
        coverUri = cover,
        bannerUri = banner,
        lastModified = lastModified,
    )

fun VolumeChapterCount.toVolumeGroupDto(items: List<ChapterFileDto>): VolumeChapterGroupDto =
    VolumeChapterGroupDto(
        volume = this.toViewDto(),
        items = items,
        totalChapters = this.chapterCount,
        loadedCount = items.size,
        hasMore = this.chapterCount > items.size,
    )

fun List<ChapterVolumeJoin>.toChapterPageDto(
    pageSize: Int = this.size,
    total: Int = this.size,
    page: Int = 0,
): ChapterPageDto =
    ChapterPageDto(
        items = this.map { it.toViewDto() },
        volumes = this.mapNotNull { it.volume }.distinctBy { it.id }.map { it.toViewDto() },
        pageSize = pageSize,
        total = total,
        page = page,
    )

fun List<ChapterVolumeJoin>.toVolumeGroupedDto(
    volumeViewType: VolumeViewType,
    pageSize: Int,
): ChapterDto {
    if (volumeViewType == VolumeViewType.CHAPTER || isEmpty()) {
        return ChapterDto(
            archive =
                ChapterPageDto(
                    items = map { it.toViewDto() },
                    volumeSections = emptyList(),
                    pageSize = pageSize,
                    page = 0,
                    total = size,
                ),
        )
    }

    val volumeGroups = mutableListOf<VolumeChapterGroupDto>()
    val currentVolumes = mutableMapOf<Long, VolumeChapterGroupDto>()

    forEach { join ->
        val volume = join.volume ?: return@forEach
        val chapterDto = join.toViewDto()

        val group =
            currentVolumes.getOrPut(volume.id) {
                val newGroup =
                    VolumeChapterGroupDto(
                        volume = volume.toViewDto(),
                        items = mutableListOf(),
                        totalChapters = 0,
                        loadedCount = 0,
                        currentPage = 0,
                        hasMore = false,
                    )
                volumeGroups.add(newGroup)
                newGroup
            }

        val items = group.items as MutableList<ChapterFileDto>
        items.add(chapterDto)

        // Update counts
        currentVolumes[volume.id] =
            group.copy(
                items = items,
                loadedCount = items.size,
                totalChapters = items.size, // In this grouping mode, we only have what's loaded
            )
    }

    return ChapterDto(
        archive =
            ChapterPageDto(
                items = emptyList(),
                volumeSections = volumeGroups,
                pageSize = pageSize,
                page = 0,
                total = size,
            ),
    )
}

fun List<VolumeChapterCount>.toVolumeSectionsDto(): ChapterDto {
    val volumeGroups =
        map { count ->
            VolumeChapterGroupDto(
                volume = count.toViewDto(),
                items = emptyList(),
                totalChapters = count.chapterCount,
                loadedCount = 0,
                currentPage = 0,
                hasMore = count.chapterCount > 0,
            )
        }

    return ChapterDto(
        archive =
            ChapterPageDto(
                items = emptyList(),
                volumeSections = volumeGroups,
                pageSize = volumeGroups.size,
                page = 0,
                total = volumeGroups.size,
            ),
    )
}

fun List<VolumeChapterGroupDto>.toCombinedVolumeDto(
    volumeOverrides: Map<Long, VolumeChapterGroupDto>,
    pageSize: Int,
    effectiveViewMode: VolumeViewType,
): ChapterDto {
    val mergedSections =
        this.map { section ->
            val sectionTotalPages =
                if (section.totalChapters == 0) {
                    1
                } else {
                    ceil(section.totalChapters.toDouble() / pageSize).toInt()
                }

            volumeOverrides[section.volume.id]?.let { override ->
                section.copy(
                    items = override.items,
                    loadedCount = override.items.size,
                    hasMore = override.items.size < section.totalChapters,
                    currentPage = override.currentPage,
                    totalPages = sectionTotalPages,
                )
            } ?: section.copy(totalPages = sectionTotalPages)
        }

    val visibleItems = mergedSections.flatMap { it.items }

    return ChapterDto(
        archive =
            ChapterPageDto(
                items = visibleItems,
                volumes = mergedSections.map { it.volume },
                volumeSections = mergedSections,
                pageSize = pageSize,
                total = mergedSections.sumOf { it.totalChapters },
                page = 0,
            ),
        showVolumeHeaders = mergedSections.size > 1,
        hasVolumeStructure = true,
        effectiveViewMode = effectiveViewMode,
    )
}

fun ChapterPageDto.toCombinedRegularDto(
    page: Int,
    pageSize: Int,
    hasVolumeStructure: Boolean,
    effectiveViewMode: VolumeViewType,
): ChapterDto {
    val items = this.items
    if (items.isEmpty()) {
        return ChapterDto(
            archive = this,
            hasVolumeStructure = hasVolumeStructure,
            effectiveViewMode = effectiveViewMode,
        )
    }

    val total = items.size
    val totalPages = ceil(total.toDouble() / pageSize).toInt()
    val safePage = page.coerceIn(0, max(0, totalPages - 1))

    // Accumulative end index for infinite scroll
    val end = ((safePage + 1) * pageSize).coerceIn(0, total)

    val pagedLocalItems = items.subList(0, end)

    return ChapterDto(
        archive =
            this.copy(
                items = pagedLocalItems,
                pageSize = pageSize,
                total = total,
                page = safePage,
            ),
        showVolumeHeaders = false,
        hasVolumeStructure = hasVolumeStructure,
        effectiveViewMode = effectiveViewMode,
    )
}
