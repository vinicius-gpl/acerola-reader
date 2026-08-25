package br.acerola.comic.service.archive

import android.net.Uri
import android.provider.DocumentsContract
import br.acerola.comic.local.dao.archive.ChapterArchiveDao
import br.acerola.comic.local.dao.history.ReadingHistoryDao
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.util.file.FastFileMetadata
import br.acerola.comic.util.sort.SortNormalizer
import br.acerola.comic.util.sort.SortType
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ChapterSyncService
    @Inject
    constructor(
        private val chapterArchiveDao: ChapterArchiveDao,
        private val readingHistoryDao: ReadingHistoryDao,
        private val archiveValidator: ArchiveValidator,
        private val chapterIndexer: ChapterIndexer,
    ) {
        private val semaphore = Semaphore(permits = SEMAPHORE_PERMITS)

        suspend fun sync(
            comicId: Long,
            allChapterFiles: List<Pair<FastFileMetadata, Long?>>,
            chapterTemplates: List<String>,
            baseUri: Uri?,
            folderUri: Uri,
            onProgress: (Int) -> Unit,
        ) {
            val existingChapters = chapterArchiveDao.getChaptersListByDirectoryId(folderId = comicId)
            val existingChaptersMap = existingChapters.associateBy { it.path }
            val chaptersToInsert = mutableListOf<ChapterArchive>()
            val chaptersToDelete = existingChapters.toMutableList()
            val processedSorts = mutableSetOf<String>()

            allChapterFiles.forEachIndexed { index, (file, volumeId) ->
                val sortResult = SortNormalizer.normalize(file.name, SortType.CHAPTER, chapterTemplates)

                if (archiveValidator.isDuplicateSort(processedSorts, sortResult.normalizedSort)) {
                    AcerolaLogger.e(
                        TAG,
                        "Duplicate chapter detected: ${sortResult.normalizedSort} for comic $comicId. Skipping.",
                        LogSource.REPOSITORY,
                    )
                    return@forEachIndexed
                }
                processedSorts.add(sortResult.normalizedSort)

                val fileUri =
                    if (baseUri != null) {
                        DocumentsContract.buildDocumentUriUsingTree(baseUri, file.id).toString()
                    } else {
                        DocumentsContract.buildDocumentUriUsingTree(folderUri, file.id).toString()
                    }

                val existing = existingChaptersMap[fileUri]
                if (existing != null && existing.lastModified == file.lastModified && existing.volumeFk == volumeId) {
                    chaptersToDelete.remove(existing)
                    // Capítulo sem alteração, mas ainda sem checksum cacheado (biblioteca
                    // indexada antes do checksum passar a ser calculado no scan, ou uma linha
                    // corrompida por uma tentativa anterior) — preenche sem tratar como
                    // novo/alterado, pra não duplicar linha nem remapear histórico à toa.
                    if (existing.checksum == null) {
                        val checksum = chapterIndexer.computeChecksum(fileUri)
                        if (checksum != null) chapterArchiveDao.update(existing.copy(checksum = checksum))
                    }
                    return@forEachIndexed
                }

                semaphore.withPermit {
                    chaptersToInsert.add(
                        chapterIndexer.buildEntity(
                            file = file,
                            comicId = comicId,
                            fileUri = fileUri,
                            chapterSort = sortResult.normalizedSort,
                            volumeFk = volumeId,
                            isSpecial = sortResult.isSpecial,
                        ),
                    )
                }
                onProgress(30 + ((index + 1) * 60 / allChapterFiles.size))
            }

            if (chaptersToDelete.isNotEmpty()) {
                AcerolaLogger.d(TAG, "Deleting ${chaptersToDelete.size} stale chapters", LogSource.REPOSITORY)
                chaptersToDelete.forEach { chapterArchiveDao.delete(it) }
            }

            if (chaptersToInsert.isNotEmpty()) {
                AcerolaLogger.d(TAG, "Inserting ${chaptersToInsert.size} new chapters and updating history", LogSource.REPOSITORY)
                chaptersToInsert.forEach { chapter ->
                    val newId = chapterArchiveDao.insert(chapter)
                    readingHistoryDao.updateHistoryChapterIdBySort(comicId, chapter.chapterSort, newId)
                    readingHistoryDao.updateChapterReadIdBySort(comicId, chapter.chapterSort, newId)
                }
            }
        }

        companion object {
            private const val TAG = "ChapterSyncService"
            private const val SEMAPHORE_PERMITS = 3
        }
    }
