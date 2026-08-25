package br.acerola.comic.repository.adapter.local.chapter

import br.acerola.comic.adapter.library.engine.VolumeArchiveEngine
import br.acerola.comic.local.dao.archive.ChapterArchiveDao
import br.acerola.comic.local.dao.archive.VolumeArchiveDao
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.entity.relation.ChapterVolumeJoin
import br.acerola.comic.local.entity.relation.VolumeChapterCount
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.every
import io.mockk.impl.annotations.MockK
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@OptIn(ExperimentalCoroutinesApi::class)
@RunWith(RobolectricTestRunner::class)
class VolumeArchiveEngineTest {
    @MockK
    lateinit var chapterArchiveDao: ChapterArchiveDao

    @MockK
    lateinit var volumeArchiveDao: VolumeArchiveDao

    private lateinit var repository: VolumeArchiveEngine

    @Before
    fun setUp() {
        MockKAnnotations.init(this)
        repository = VolumeArchiveEngine(volumeArchiveDao, chapterArchiveDao)
    }

    @Test
    fun `observeVolumeGroups should emit groups correctly`() =
        runTest {
            val comicId = 1L
            val summaries =
                listOf(
                    VolumeChapterCount(
                        id = 101L,
                        name = "Vol 1",
                        volumeSort = "1",
                        isSpecial = false,
                        cover = null,
                        banner = null,
                        chapterCount = 2,
                    ),
                )

            val chapters =
                listOf(
                    ChapterVolumeJoin(
                        chapter = ChapterArchive(id = 1, chapter = "1", path = "p1", chapterSort = "1", folderPathFk = comicId, volumeFk = 101L),
                        volume = null,
                    ),
                )

            every { volumeArchiveDao.getVolumeChapterCountsByDirectoryId(comicId) } returns flowOf(summaries)
            coEvery { chapterArchiveDao.getChaptersByVolumePaged(comicId, 101L, 5, 0) } returns chapters

            val result = repository.observeVolumeGroups(comicId).first { it.isNotEmpty() && it[0].volume.id != -1L }

            assertEquals(1, result.size)
            assertEquals("Vol 1", result[0].volume.name)
            assertEquals(1, result[0].items.size)
            assertEquals("1", result[0].items[0].name)
        }

    @Test
    fun `getVolumeChapterPage should return chapters list`() =
        runTest {
            val comicId = 1L
            val volumeId = 101L
            val chapters =
                listOf(
                    ChapterVolumeJoin(
                        chapter =
                            ChapterArchive(
                                id = 1,
                                chapter = "1",
                                path = "p1",
                                chapterSort = "1",
                                folderPathFk = comicId,
                                volumeFk = volumeId,
                            ),
                        volume = null,
                    ),
                )

            coEvery { chapterArchiveDao.getChaptersByVolumePaged(comicId, volumeId, 20, 0) } returns chapters

            val result = repository.getVolumeChapterPage(comicId, volumeId, 0)

            assertEquals(1, result.size)
            assertEquals("1", result[0].name)
        }

    @Test
    fun `observeHasRootChapters should emit true if there are chapters in root`() =
        runTest {
            val comicId = 1L
            every { chapterArchiveDao.observeRootChaptersCountByDirectoryId(comicId) } returns flowOf(1)

            val result = repository.observeHasRootChapters(comicId).first()

            assertEquals(true, result)
        }
}
