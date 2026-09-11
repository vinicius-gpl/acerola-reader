package br.acerola.comic.service.network

import br.acerola.comic.local.dao.archive.ComicDirectoryDao
import br.acerola.comic.local.dao.history.ReadingHistoryDao
import br.acerola.comic.local.entity.history.ChapterRead
import br.acerola.comic.local.entity.history.ReadingHistory
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import p2p.FfiChapterReadEntry
import p2p.FfiReadingProgressEntry
import p2p.HistorySyncProvider
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Kotlin implementation of [HistorySyncProvider] (foreign `uniffi` trait) used by the Rust
 * protocol `acerola/sync-history/1`. Only reads/writes Room by natural key — all
 * manifest/diff/LWW/idempotent-union logic stays on the Rust side.
 */
@Singleton
class HistorySyncProviderImpl
    @Inject
    constructor(
        private val comicDirectoryDao: ComicDirectoryDao,
        private val readingHistoryDao: ReadingHistoryDao,
    ) : HistorySyncProvider {
        override fun getReadingProgress(): List<FfiReadingProgressEntry> =
            runBlocking {
                val comicNamesById = comicDirectoryDao.getAllDirectories().first().associate { it.id to it.name }
                readingHistoryDao.observeAllRecentHistories().first().mapNotNull { history ->
                    val comicName = comicNamesById[history.comicDirectoryFk] ?: return@mapNotNull null
                    FfiReadingProgressEntry(
                        comicName = comicName,
                        chapterSort = history.chapterSort,
                        lastPage = history.lastPage,
                        isCompleted = history.isCompleted,
                        updatedAt = history.updatedAt,
                    )
                }
            }

        override fun getChaptersRead(): List<FfiChapterReadEntry> =
            runBlocking {
                val comicNamesById = comicDirectoryDao.getAllDirectories().first().associate { it.id to it.name }
                readingHistoryDao.observeAllChapterReads().first().mapNotNull { chapterRead ->
                    val comicName = comicNamesById[chapterRead.comicDirectoryFk] ?: return@mapNotNull null
                    FfiChapterReadEntry(
                        comicName = comicName,
                        chapterSort = chapterRead.chapterSort,
                        createdAt = chapterRead.createdAt,
                    )
                }
            }

        override fun getReadingProgressForComic(comicName: String): FfiReadingProgressEntry? =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(comicName) ?: return@runBlocking null
                val history = readingHistoryDao.observeHistoryByDirectoryId(comic.id).first() ?: return@runBlocking null
                FfiReadingProgressEntry(
                    comicName = comicName,
                    chapterSort = history.chapterSort,
                    lastPage = history.lastPage,
                    isCompleted = history.isCompleted,
                    updatedAt = history.updatedAt,
                )
            }

        override fun getReadingProgressForChapters(
            comicName: String,
            chapterSorts: List<String>,
        ): List<FfiReadingProgressEntry> =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(comicName) ?: return@runBlocking emptyList()
                val history = readingHistoryDao.observeHistoryByDirectoryId(comic.id).first() ?: return@runBlocking emptyList()
                if (history.chapterSort !in chapterSorts) return@runBlocking emptyList()

                listOf(
                    FfiReadingProgressEntry(
                        comicName = comicName,
                        chapterSort = history.chapterSort,
                        lastPage = history.lastPage,
                        isCompleted = history.isCompleted,
                        updatedAt = history.updatedAt,
                    ),
                )
            }

        override fun getChaptersReadForChapters(
            comicName: String,
            chapterSorts: List<String>,
        ): List<FfiChapterReadEntry> =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(comicName) ?: return@runBlocking emptyList()
                readingHistoryDao.getChapterReadsByDirectoryIdAndSorts(comic.id, chapterSorts).map { chapterRead ->
                    FfiChapterReadEntry(
                        comicName = comicName,
                        chapterSort = chapterRead.chapterSort,
                        createdAt = chapterRead.createdAt,
                    )
                }
            }

        override fun comicExists(comicName: String): Boolean =
            runBlocking {
                comicDirectoryDao.getDirectoryByName(comicName) != null
            }

        override fun applyReadingProgress(entry: FfiReadingProgressEntry): Boolean =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(entry.comicName) ?: return@runBlocking false
                readingHistoryDao.upsertHistory(
                    ReadingHistory(
                        comicDirectoryFk = comic.id,
                        chapterSort = entry.chapterSort,
                        chapterArchiveFk = null,
                        lastPage = entry.lastPage,
                        isCompleted = entry.isCompleted,
                        updatedAt = entry.updatedAt,
                    ),
                )
                true
            }

        override fun applyChapterRead(entry: FfiChapterReadEntry): Boolean =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(entry.comicName) ?: return@runBlocking false
                readingHistoryDao.upsertChapterRead(
                    ChapterRead(
                        comicDirectoryFk = comic.id,
                        chapterSort = entry.chapterSort,
                        chapterArchiveFk = null,
                        createdAt = entry.createdAt,
                    ),
                )
                true
            }
    }
