package br.acerola.comic.local.entity.archive

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey

@Entity(
    tableName = "chapter_archive",
    indices = [
        Index(value = ["comic_directory_fk", "chapter"], unique = true),
        Index(value = ["volume_fk"]),
    ],
    foreignKeys = [
        ForeignKey(
            entity = ComicDirectory::class,
            parentColumns = ["id"],
            childColumns = ["comic_directory_fk"],
            onDelete = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = VolumeArchive::class,
            parentColumns = ["id"],
            childColumns = ["volume_fk"],
            onDelete = ForeignKey.CASCADE,
        ),
    ],
)
data class ChapterArchive(
    @PrimaryKey(autoGenerate = true)
    val id: Long = 0,
    @ColumnInfo(name = "chapter")
    val chapter: String,
    @ColumnInfo(name = "path")
    val path: String,
    @ColumnInfo(name = "chapter_sort")
    val chapterSort: String,
    @ColumnInfo(name = "is_special", defaultValue = "0")
    val isSpecial: Boolean = false,
    // NOTE: Campo vai manter um hash do arquivo, para se tiver quebrado, ele ignorar no frontend
    @ColumnInfo(name = "checksum")
    val checksum: String? = null,
    @ColumnInfo(name = "comic_directory_fk")
    val folderPathFk: Long,
    @ColumnInfo(name = "volume_fk")
    val volumeFk: Long? = null,
    @ColumnInfo(name = "last_modified", defaultValue = "0")
    val lastModified: Long = 0,
)
