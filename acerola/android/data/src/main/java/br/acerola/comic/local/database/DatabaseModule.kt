package br.acerola.comic.local.database

import android.content.Context
import androidx.room.Room
import androidx.room.RoomDatabase
import androidx.sqlite.db.SupportSQLiteDatabase
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
import br.acerola.comic.local.database.migrations.MIGRATION_1_2
import br.acerola.comic.local.database.migrations.MIGRATION_2_3
import br.acerola.comic.local.database.migrations.MIGRATION_3_4
import br.acerola.comic.local.database.migrations.MIGRATION_4_5
import br.acerola.comic.local.database.migrations.MIGRATION_5_6
import br.acerola.comic.local.database.seeds.seedArchiveTemplates
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object DatabaseModule {
    @Provides
    @Singleton
    fun provideDatabase(
        @ApplicationContext context: Context,
    ): AcerolaDatabase =
        Room
            .databaseBuilder(
                context,
                AcerolaDatabase::class.java,
                "acerola_database",
            ).addCallback(
                object : RoomDatabase.Callback() {
                    override fun onCreate(db: SupportSQLiteDatabase) {
                        super.onCreate(db)
                        seedArchiveTemplates(db)
                    }
                },
            ).addMigrations(MIGRATION_1_2, MIGRATION_2_3, MIGRATION_3_4, MIGRATION_4_5, MIGRATION_5_6)
            .build()

    @Provides
    fun provideChapterArchiveDao(db: AcerolaDatabase): ChapterArchiveDao = db.chapterArchiveDao()

    @Provides
    fun provideVolumeArchiveDao(db: AcerolaDatabase): VolumeArchiveDao = db.volumeArchiveDao()

    @Provides
    fun provideMangaDirectoryDao(db: AcerolaDatabase): ComicDirectoryDao = db.comicDirectoryDao()

    @Provides
    fun provideArchiveTemplateDao(db: AcerolaDatabase): ArchiveTemplateDao = db.archiveTemplateDao()

    @Provides
    fun provideMangaRemoteInfoDao(db: AcerolaDatabase): ComicMetadataDao = db.comicRemoteInfoDao()

    @Provides
    fun provideAuthorDao(db: AcerolaDatabase): AuthorDao = db.authorDao()

    @Provides
    fun provideGenreDao(db: AcerolaDatabase): GenreDao = db.genreDao()

    @Provides
    fun provideReadingHistoryDao(db: AcerolaDatabase): ReadingHistoryDao = db.readingHistoryDao()

    @Provides
    fun provideMangadexSourceDao(db: AcerolaDatabase): MangadexSourceDao = db.mangadexSourceDao()

    @Provides
    fun provideAnilistSourceDao(db: AcerolaDatabase): AnilistSourceDao = db.anilistSourceDao()

    @Provides
    fun provideCategoryDao(db: AcerolaDatabase): CategoryDao = db.categoryDao()

    @Provides
    fun provideMangaSummaryDao(db: AcerolaDatabase): ComicSummaryDao = db.comicSummaryDao()

    @Provides
    fun provideSyncHistoryLogDao(db: AcerolaDatabase): SyncHistoryLogDao = db.syncHistoryLogDao()
}
