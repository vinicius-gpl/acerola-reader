package br.acerola.comic.local.dao.history

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import br.acerola.comic.local.entity.history.ChapterRead
import br.acerola.comic.local.entity.history.ReadingHistory
import br.acerola.comic.local.entity.relation.ChapterReadingStatus
import kotlinx.coroutines.flow.Flow

@Dao
interface ReadingHistoryDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertHistory(history: ReadingHistory)

    @Query("SELECT * FROM reading_history WHERE comic_directory_fk = :comicId")
    fun observeHistoryByDirectoryId(comicId: Long): Flow<ReadingHistory?>

    @Query("SELECT * FROM reading_history ORDER BY updated_at DESC")
    fun observeAllRecentHistories(): Flow<List<ReadingHistory>>

    @Query(
        """
        SELECT
            rh.comic_directory_fk AS comicDirectoryId,
            rh.chapter_archive_fk AS chapterArchiveId,
            rh.chapter_sort AS chapterSort,
            rh.last_page AS lastPage,
            rh.updated_at AS updatedAt,
            ca.chapter AS chapterName,
            rh.is_completed AS isCompleted
        FROM reading_history rh
        LEFT JOIN chapter_archive ca
            ON rh.comic_directory_fk = ca.comic_directory_fk
            AND rh.chapter_sort = ca.chapter_sort
        ORDER BY rh.updated_at DESC;
    """,
    )
    fun observeAllRecentHistoriesWithChapter(): Flow<List<ChapterReadingStatus>>

    @Query("DELETE FROM reading_history WHERE comic_directory_fk = :comicId")
    suspend fun deleteHistoryByDirectoryId(comicId: Long)

    @Query("UPDATE reading_history SET chapter_archive_fk = :newId WHERE comic_directory_fk = :comicId AND chapter_sort = :chapterSort")
    suspend fun updateHistoryChapterIdBySort(
        comicId: Long,
        chapterSort: String,
        newId: Long,
    )

    @Query("UPDATE chapter_read SET chapter_archive_fk = :newId WHERE comic_directory_fk = :comicId AND chapter_sort = :chapterSort")
    suspend fun updateChapterReadIdBySort(
        comicId: Long,
        chapterSort: String,
        newId: Long,
    )

    // Chapter Read
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertChapterRead(chapterRead: ChapterRead)

    @Query("SELECT chapter_sort FROM chapter_read WHERE comic_directory_fk = :comicId")
    fun observeReadChaptersByDirectoryId(comicId: Long): Flow<List<String>>

    @Query("SELECT * FROM chapter_read")
    fun observeAllChapterReads(): Flow<List<ChapterRead>>

    @Query("SELECT * FROM chapter_read WHERE comic_directory_fk = :comicId AND chapter_sort IN (:chapterSorts)")
    suspend fun getChapterReadsByDirectoryIdAndSorts(
        comicId: Long,
        chapterSorts: List<String>,
    ): List<ChapterRead>

    @Query("DELETE FROM chapter_read WHERE comic_directory_fk = :comicId AND chapter_sort = :chapterSort")
    suspend fun deleteChapterRead(
        comicId: Long,
        chapterSort: String,
    )
}
