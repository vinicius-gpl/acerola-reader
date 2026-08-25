package br.acerola.comic.local.database.migrations

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

val MIGRATION_5_6 =
    object : Migration(5, 6) {
        override fun migrate(db: SupportSQLiteDatabase) {
            // Refactor chapter_archive (volume_id_fk -> volume_fk)
            db.execSQL(
                """
                CREATE TABLE `chapter_archive_new` (
                    `id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    `chapter` TEXT NOT NULL,
                    `path` TEXT NOT NULL,
                    `chapter_sort` TEXT NOT NULL,
                    `is_special` INTEGER NOT NULL DEFAULT 0,
                    `checksum` TEXT,
                    `comic_directory_fk` INTEGER NOT NULL,
                    `volume_fk` INTEGER,
                    `last_modified` INTEGER NOT NULL DEFAULT 0,
                    FOREIGN KEY(`comic_directory_fk`) REFERENCES `comic_directory`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
                    FOREIGN KEY(`volume_fk`) REFERENCES `volume_archive`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE
                )
                """.trimIndent(),
            )
            db.execSQL(
                """
                INSERT INTO `chapter_archive_new` (`id`, `chapter`, `path`, `chapter_sort`, `is_special`, `checksum`, `comic_directory_fk`, `volume_fk`, `last_modified`)
                SELECT `id`, `chapter`, `path`, `chapter_sort`, `is_special`, `checksum`, `comic_directory_fk`, `volume_id_fk`, `last_modified`
                FROM `chapter_archive`
                """.trimIndent(),
            )
            db.execSQL("DROP TABLE `chapter_archive`")
            db.execSQL("ALTER TABLE `chapter_archive_new` RENAME TO `chapter_archive`")
            db.execSQL(
                "CREATE UNIQUE INDEX IF NOT EXISTS `index_chapter_archive_comic_directory_fk_chapter` ON `chapter_archive` (`comic_directory_fk`, `chapter`)",
            )
            db.execSQL(
                "CREATE INDEX IF NOT EXISTS `index_chapter_archive_volume_fk` ON `chapter_archive` (`volume_fk`)",
            )

            // comic_summary_view (directory_id/metadata_id -> directory_fk/metadata_fk): Room
            // recria as views declaradas em @Database automaticamente apos rodar as migrations,
            // usando a definicao atual de ComicSummaryView — nenhum DROP/CREATE VIEW manual aqui.
        }
    }
