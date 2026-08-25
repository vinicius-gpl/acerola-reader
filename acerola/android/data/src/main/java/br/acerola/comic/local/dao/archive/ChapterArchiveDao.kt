package br.acerola.comic.local.dao.archive

import androidx.room.Dao
import androidx.room.Query
import androidx.room.Transaction
import br.acerola.comic.local.dao.BaseDao
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.entity.relation.ChapterVolumeJoin
import br.acerola.comic.local.entity.relation.LibrarySummaryRow
import br.acerola.comic.local.entity.relation.MangaChapterCount
import kotlinx.coroutines.flow.Flow

@Dao
interface ChapterArchiveDao : BaseDao<ChapterArchive> {
    @Query(value = "DELETE FROM chapter_archive WHERE comic_directory_fk = :folderId")
    suspend fun deleteByDirectoryId(folderId: Long)

    @Query(value = "SELECT * FROM chapter_archive ORDER BY chapter ASC")
    fun getAllChapters(): Flow<List<ChapterArchive>>

    @Query(value = "SELECT * FROM chapter_archive WHERE id = :chapterId")
    fun getChapterById(chapterId: Long): Flow<ChapterArchive?>

    @Query(value = "SELECT COUNT(id) FROM chapter_archive WHERE comic_directory_fk = :folderId")
    suspend fun countByDirectoryId(folderId: Long): Int

    @Query("SELECT * FROM chapter_archive WHERE comic_directory_fk = :folderId")
    suspend fun getChaptersListByDirectoryId(folderId: Long): List<ChapterArchive>

    @Query("SELECT COUNT(*) FROM chapter_archive WHERE comic_directory_fk = :folderId AND volume_fk IS NULL")
    suspend fun countRootChaptersByDirectoryId(folderId: Long): Int

    @Query("SELECT COUNT(*) FROM chapter_archive WHERE comic_directory_fk = :folderId AND volume_fk IS NULL")
    fun observeRootChaptersCountByDirectoryId(folderId: Long): Flow<Int>

    @Transaction
    @Query(
        value = """
        SELECT *
        FROM chapter_archive
        LEFT JOIN volume_archive ON chapter_archive.volume_fk = volume_archive.id
        WHERE chapter_archive.comic_directory_fk = :folderId 
        ORDER BY 
            (chapter_archive.is_special OR COALESCE(volume_archive.is_special, 0)) ASC,
            (CASE WHEN volume_archive.volume_sort GLOB '*[0-9]*' THEN 0 ELSE 1 END) ASC,
            CAST(COALESCE(volume_archive.volume_sort, '0') AS REAL) ASC,
            CAST(chapter_archive.chapter_sort AS REAL) ASC, 
            chapter_archive.chapter_sort ASC
    """,
    )
    fun getChaptersByDirectoryId(folderId: Long): Flow<List<ChapterVolumeJoin>>

    @Transaction
    @Query(
        value = """
        SELECT *
        FROM chapter_archive
        LEFT JOIN volume_archive ON chapter_archive.volume_fk = volume_archive.id
        WHERE chapter_archive.comic_directory_fk = :folderId 
        ORDER BY 
            (chapter_archive.is_special OR COALESCE(volume_archive.is_special, 0)) ASC,
            (CASE WHEN volume_archive.volume_sort GLOB '*[0-9]*' THEN 0 ELSE 1 END) ASC,
            CAST(COALESCE(volume_archive.volume_sort, '0') AS REAL) DESC,
            CAST(chapter_archive.chapter_sort AS REAL) DESC, 
            chapter_archive.chapter_sort DESC
    """,
    )
    fun getChaptersByDirectoryIdDesc(folderId: Long): Flow<List<ChapterVolumeJoin>>

    @Transaction
    @Query(
        value = """
            SELECT *
            FROM chapter_archive
            LEFT JOIN volume_archive ON chapter_archive.volume_fk = volume_archive.id
            WHERE chapter_archive.comic_directory_fk = :folderId
            ORDER BY 
                (chapter_archive.is_special OR COALESCE(volume_archive.is_special, 0)) ASC,
                (CASE WHEN volume_archive.volume_sort GLOB '*[0-9]*' THEN 0 ELSE 1 END) ASC,
                CAST(COALESCE(volume_archive.volume_sort, '0') AS REAL) ASC,
                CAST(chapter_archive.chapter_sort AS REAL) ASC
            LIMIT :pageSize OFFSET :offset
        """,
    )
    suspend fun getChaptersByDirectoryPaged(
        folderId: Long,
        pageSize: Int,
        offset: Int,
    ): List<ChapterVolumeJoin>

    @Transaction
    @Query(
        value = """
            SELECT *
            FROM chapter_archive
            LEFT JOIN volume_archive ON chapter_archive.volume_fk = volume_archive.id
            WHERE chapter_archive.comic_directory_fk = :folderId
            ORDER BY 
                (chapter_archive.is_special OR COALESCE(volume_archive.is_special, 0)) ASC,
                (CASE WHEN volume_archive.volume_sort GLOB '*[0-9]*' THEN 0 ELSE 1 END) ASC,
                CAST(COALESCE(volume_archive.volume_sort, '0') AS REAL) DESC,
                CAST(chapter_archive.chapter_sort AS REAL) DESC
            LIMIT :pageSize OFFSET :offset
        """,
    )
    suspend fun getChaptersByDirectoryPagedDesc(
        folderId: Long,
        pageSize: Int,
        offset: Int,
    ): List<ChapterVolumeJoin>

    @Transaction
    @Query(
        value = """
            SELECT *
            FROM chapter_archive
            LEFT JOIN volume_archive ON chapter_archive.volume_fk = volume_archive.id
            WHERE chapter_archive.comic_directory_fk = :comicId
              AND chapter_archive.volume_fk = :volumeId
            ORDER BY 
                CAST(chapter_archive.chapter_sort AS REAL) ASC,
                (chapter_archive.is_special OR COALESCE(volume_archive.is_special, 0)) ASC
            LIMIT :pageSize OFFSET :offset
        """,
    )
    suspend fun getChaptersByVolumePaged(
        comicId: Long,
        volumeId: Long,
        pageSize: Int,
        offset: Int,
    ): List<ChapterVolumeJoin>

    @Transaction
    @Query(
        value = """
            SELECT *
            FROM chapter_archive
            LEFT JOIN volume_archive ON chapter_archive.volume_fk = volume_archive.id
            WHERE chapter_archive.comic_directory_fk = :comicId
              AND chapter_archive.volume_fk = :volumeId
            ORDER BY 
                CAST(chapter_archive.chapter_sort AS REAL) DESC,
                (chapter_archive.is_special OR COALESCE(volume_archive.is_special, 0)) ASC
            LIMIT :pageSize OFFSET :offset
        """,
    )
    suspend fun getChaptersByVolumePagedDesc(
        comicId: Long,
        volumeId: Long,
        pageSize: Int,
        offset: Int,
    ): List<ChapterVolumeJoin>

    @Query("SELECT * FROM chapter_archive WHERE comic_directory_fk = :folderId AND chapter_sort IN (:chapters)")
    fun getChaptersByDirectoryAndSorts(
        folderId: Long,
        chapters: List<String>,
    ): Flow<List<ChapterArchive>>

    @Query("SELECT * FROM chapter_archive WHERE comic_directory_fk = :folderId AND chapter = :chapter")
    suspend fun getChapterByDirectoryAndChapter(
        folderId: Long,
        chapter: String,
    ): ChapterArchive?

    @Query("SELECT comic_directory_fk, COUNT(*) as count FROM chapter_archive GROUP BY comic_directory_fk")
    fun getChapterCountsByDirectory(): Flow<List<MangaChapterCount>>

    // Só Room/SQLite, de propósito — nenhum toque em SAF/DocumentFile. Usada por
    // `browse-library` (P2P) pra listar título + contagem de capítulos sem pagar o custo de
    // uma transação binder por capítulo, que já estourou o timeout do protocolo numa
    // biblioteca grande (ver `FileSyncProviderImpl.getLibrarySummary`).
    @Query(
        value = """
            SELECT cd.name AS comic_name, COUNT(ca.id) AS chapter_count, cd.last_modified AS cover_version
            FROM comic_directory cd
            JOIN chapter_archive ca ON ca.comic_directory_fk = cd.id
            GROUP BY cd.name
            ORDER BY cd.name ASC
        """,
    )
    suspend fun getLibrarySummary(): List<LibrarySummaryRow>
}
