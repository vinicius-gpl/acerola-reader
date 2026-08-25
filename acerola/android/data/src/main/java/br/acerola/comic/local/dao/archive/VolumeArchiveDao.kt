package br.acerola.comic.local.dao.archive

import androidx.room.Dao
import androidx.room.Query
import br.acerola.comic.local.dao.BaseDao
import br.acerola.comic.local.entity.archive.VolumeArchive
import br.acerola.comic.local.entity.relation.VolumeChapterCount
import kotlinx.coroutines.flow.Flow

@Dao
interface VolumeArchiveDao : BaseDao<VolumeArchive> {
    @Query("SELECT * FROM volume_archive WHERE comic_directory_fk = :folderId")
    fun getVolumesByDirectoryId(folderId: Long): Flow<List<VolumeArchive>>

    @Query("SELECT * FROM volume_archive WHERE comic_directory_fk = :folderId")
    suspend fun getVolumesListByDirectoryId(folderId: Long): List<VolumeArchive>

    @Query("SELECT * FROM volume_archive WHERE id = :volumeId")
    suspend fun getVolumeById(volumeId: Long): VolumeArchive?

    @Query(
        """
        SELECT
            volume_archive.id AS id,
            volume_archive.name AS name,
            volume_archive.volume_sort AS volumeSort,
            volume_archive.is_special AS isSpecial,
            volume_archive.cover AS cover,
            volume_archive.banner AS banner,
            volume_archive.last_modified AS lastModified,
            COUNT(chapter_archive.id) AS chapterCount
        FROM volume_archive
        LEFT JOIN chapter_archive ON chapter_archive.volume_fk = volume_archive.id
        WHERE volume_archive.comic_directory_fk = :folderId
        GROUP BY volume_archive.id
        ORDER BY
            volume_archive.is_special ASC,
            CAST(volume_archive.volume_sort AS INTEGER) ASC,
            CAST(
                CASE
                    WHEN volume_archive.volume_sort LIKE '%.%'
                    THEN SUBSTR(volume_archive.volume_sort, INSTR(volume_archive.volume_sort, '.') + 1)
                    ELSE 0
                END AS INTEGER
            ) ASC
        """,
    )
    fun getVolumeChapterCountsByDirectoryId(folderId: Long): Flow<List<VolumeChapterCount>>

    @Query("DELETE FROM volume_archive WHERE comic_directory_fk = :folderId")
    suspend fun deleteByDirectoryId(folderId: Long)
}
