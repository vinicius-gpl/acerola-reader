package br.acerola.comic.local.entity.view

import androidx.room.ColumnInfo
import androidx.room.DatabaseView

@DatabaseView(
    viewName = "comic_summary_view",
    value = """
        SELECT
            md.id AS directory_fk,
            md.name AS folder_name,
            md.cover AS folder_cover,
            md.banner AS folder_banner,
            md.external_sync_enabled AS external_sync,
            mm.title AS metadata_title,
            mm.sync_source AS active_source,
            mm.id AS metadata_fk
        FROM comic_directory md
        LEFT JOIN comic_metadata mm ON md.id = mm.comic_directory_fk
    """,
)
data class ComicSummaryView(
    @ColumnInfo(name = "directory_fk")
    val directoryFk: Long,
    @ColumnInfo(name = "folder_name")
    val folderName: String,
    @ColumnInfo(name = "folder_cover")
    val folderCover: String?,
    @ColumnInfo(name = "folder_banner")
    val folderBanner: String?,
    @ColumnInfo(name = "external_sync")
    val externalSync: Boolean,
    @ColumnInfo(name = "metadata_title")
    val metadataTitle: String?,
    @ColumnInfo(name = "active_source")
    val activeSource: String?,
    @ColumnInfo(name = "metadata_fk")
    val metadataFk: Long?,
)
