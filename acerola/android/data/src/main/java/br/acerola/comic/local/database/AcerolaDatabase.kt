package br.acerola.comic.local.database

import androidx.room.Database
import androidx.room.RoomDatabase
import androidx.room.TypeConverters
import br.acerola.comic.local.converter.AcerolaTypeConverters
import br.acerola.comic.local.dao.archive.ArchiveTemplateDao
import br.acerola.comic.local.dao.archive.ChapterArchiveDao
import br.acerola.comic.local.dao.archive.ComicDirectoryDao
import br.acerola.comic.local.dao.archive.VolumeArchiveDao
import br.acerola.comic.local.dao.category.CategoryDao
import br.acerola.comic.local.dao.history.ReadingHistoryDao
import br.acerola.comic.local.dao.metadata.ComicMetadataDao
import br.acerola.comic.local.dao.metadata.relationship.AuthorDao
import br.acerola.comic.local.dao.metadata.relationship.GenreDao
import br.acerola.comic.local.dao.metadata.source.AnilistSourceDao
import br.acerola.comic.local.dao.metadata.source.MangadexSourceDao
import br.acerola.comic.local.dao.sync.SyncHistoryLogDao
import br.acerola.comic.local.dao.view.ComicSummaryDao
import br.acerola.comic.local.entity.archive.ArchiveTemplate
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.entity.archive.ComicDirectory
import br.acerola.comic.local.entity.archive.VolumeArchive
import br.acerola.comic.local.entity.category.Category
import br.acerola.comic.local.entity.category.ComicCategory
import br.acerola.comic.local.entity.history.ChapterRead
import br.acerola.comic.local.entity.history.ReadingHistory
import br.acerola.comic.local.entity.metadata.ComicMetadata
import br.acerola.comic.local.entity.metadata.relationship.Author
import br.acerola.comic.local.entity.metadata.relationship.Genre
import br.acerola.comic.local.entity.metadata.source.AnilistSource
import br.acerola.comic.local.entity.metadata.source.MangadexSource
import br.acerola.comic.local.entity.sync.SyncHistoryLog
import br.acerola.comic.local.entity.view.ComicSummaryView

@Database(
    entities = [
        ComicDirectory::class,
        ArchiveTemplate::class,
        ComicMetadata::class,
        ChapterArchive::class,
        VolumeArchive::class,
        Author::class,
        Genre::class,
        ReadingHistory::class,
        ChapterRead::class,
        MangadexSource::class,
        AnilistSource::class,
        Category::class,
        ComicCategory::class,
        SyncHistoryLog::class,
    ],
    views = [
        ComicSummaryView::class,
    ],
    exportSchema = true,
    version = 6,
)
@TypeConverters(AcerolaTypeConverters::class)
abstract class AcerolaDatabase : RoomDatabase() {
    abstract fun chapterArchiveDao(): ChapterArchiveDao

    abstract fun volumeArchiveDao(): VolumeArchiveDao

    abstract fun comicDirectoryDao(): ComicDirectoryDao

    abstract fun archiveTemplateDao(): ArchiveTemplateDao

    abstract fun comicRemoteInfoDao(): ComicMetadataDao

    abstract fun authorDao(): AuthorDao

    abstract fun genreDao(): GenreDao

    abstract fun readingHistoryDao(): ReadingHistoryDao

    abstract fun mangadexSourceDao(): MangadexSourceDao

    abstract fun anilistSourceDao(): AnilistSourceDao

    abstract fun categoryDao(): CategoryDao

    abstract fun comicSummaryDao(): ComicSummaryDao

    abstract fun syncHistoryLogDao(): SyncHistoryLogDao
}
