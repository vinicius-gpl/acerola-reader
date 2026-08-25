package br.acerola.comic.usecase.network

import br.acerola.comic.local.dao.archive.ChapterArchiveDao
import br.acerola.comic.local.dao.archive.ComicDirectoryDao
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.entity.archive.ComicDirectory
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Registers a chapter received via `acerola/sync-files/1` in Room, creating the
 * [ComicDirectory] and/or [ChapterArchive] by natural key if they don't exist locally yet.
 * Deliberately doesn't reuse `DirectoryScanner` (whole-tree scan) — here we already know
 * exactly which file arrived and where.
 */
@Singleton
class RegisterSyncedChapterUseCase
    @Inject
    constructor(
        private val comicDirectoryDao: ComicDirectoryDao,
        private val chapterArchiveDao: ChapterArchiveDao,
    ) {
        suspend operator fun invoke(
            comicName: String,
            chapter: String,
            checksum: String,
            fileUri: String,
            comicFolderUri: String,
        ): Long {
            val comic =
                comicDirectoryDao.getDirectoryByName(comicName) ?: run {
                    val newComic =
                        ComicDirectory(
                            name = comicName,
                            path = comicFolderUri,
                            cover = null,
                            banner = null,
                            lastModified = System.currentTimeMillis(),
                            archiveTemplateFk = null,
                        )
                    newComic.copy(id = comicDirectoryDao.insert(newComic))
                }

            val existingChapter = chapterArchiveDao.getChapterByDirectoryAndChapter(comic.id, chapter)
            val chapterEntity =
                ChapterArchive(
                    id = existingChapter?.id ?: 0,
                    chapter = chapter,
                    path = fileUri,
                    chapterSort = existingChapter?.chapterSort ?: chapter,
                    isSpecial = existingChapter?.isSpecial ?: false,
                    checksum = checksum,
                    folderPathFk = comic.id,
                    volumeFk = existingChapter?.volumeFk,
                    lastModified = System.currentTimeMillis(),
                )

            return if (existingChapter != null) {
                chapterArchiveDao.update(chapterEntity)
                existingChapter.id
            } else {
                chapterArchiveDao.insert(chapterEntity)
            }
        }
    }
