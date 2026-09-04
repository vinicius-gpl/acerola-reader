package br.acerola.comic.usecase.chapter

import br.acerola.comic.adapter.contract.gateway.ChapterReadGateway
import br.acerola.comic.adapter.contract.gateway.ChapterSyncStatusGateway
import br.acerola.comic.adapter.contract.gateway.VolumeGateway
import br.acerola.comic.config.preference.types.ChapterSortPreferenceData
import br.acerola.comic.config.preference.types.ChapterSortType
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.dto.ChapterDto
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.dto.archive.ChapterPageDto
import br.acerola.comic.dto.archive.VolumeChapterGroupDto
import br.acerola.comic.service.cache.ChapterCacheHandler
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.impl.annotations.MockK
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Before
import org.junit.Test

class ObserveCombinedChaptersUseCaseTest {
    @MockK
    lateinit var volumeGateway: VolumeGateway

    @MockK
    lateinit var localReadGateway: ChapterReadGateway<ChapterPageDto>

    @MockK
    lateinit var localSyncStatusGateway: ChapterSyncStatusGateway

    @MockK
    lateinit var cacheHandler: ChapterCacheHandler

    private lateinit var useCase: ObserveCombinedChaptersUseCase

    private val inMemoryCache = mutableMapOf<String, ChapterDto>()

    @Before
    fun setUp() {
        MockKAnnotations.init(this)
        inMemoryCache.clear()

        every { localSyncStatusGateway.progress } returns MutableStateFlow(value = 0)
        every { localSyncStatusGateway.isIndexing } returns MutableStateFlow(value = false)

        every { cacheHandler.generateKey(any(), any(), any(), any(), any(), any(), any()) } answers {
            val args = it.invocation.args
            "${args[0]}_${args[1]}_${args[2]}_${args[3]}_${args[4]}_${args[5]}_${args[6]}"
        }
        every { cacheHandler.get(any()) } answers { inMemoryCache[it.invocation.args[0] as String] }
        every { cacheHandler.put(any(), any()) } answers {
            val key = it.invocation.args[0] as String
            val value = it.invocation.args[1] as ChapterDto
            if (value.archive.items.isNotEmpty() || value.archive.volumeSections.isNotEmpty()) {
                inMemoryCache[key] = value
            }
            Unit
        }

        useCase =
            ObserveCombinedChaptersUseCase(
                volumeGateway = volumeGateway,
                localReadGateway = localReadGateway,
                localSyncStatusGateway = localSyncStatusGateway,
                cacheHandler = cacheHandler,
            )
    }

    @Test
    fun `should observe local chapters and volume groups successfully`() =
        runTest {
            val localChapter =
                ChapterFileDto(
                    id = 1L,
                    name = "Ch 1",
                    path = "/ch1.cbz",
                    chapterSort = "1",
                )
            val localPage =
                ChapterPageDto(
                    items = listOf(localChapter),
                    pageSize = 20,
                    page = 0,
                    total = 1,
                )

            val sort = ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.ASCENDING)
            val localFlow = MutableStateFlow(localPage)
            val volumeGroupsFlow = MutableStateFlow<List<VolumeChapterGroupDto>>(emptyList())
            val hasRootFlow = MutableStateFlow(false)

            every { localReadGateway.observeChapters(1L, "NUMBER", true) } returns localFlow
            every { volumeGateway.observeVolumeGroups(1L, 20, "NUMBER", true) } returns volumeGroupsFlow
            every { volumeGateway.observeHasRootChapters(1L) } returns hasRootFlow

            val result =
                useCase
                    .observeCombined(
                        comicId = 1L,
                        remoteId = null,
                        sort = sort,
                        page = 0,
                        pageSize = 20,
                        viewMode = VolumeViewType.CHAPTER,
                        volumeOverrides = emptyMap(),
                    ).first()

            assertNotNull(result)
            assertEquals(1, result!!.archive.items.size)
            assertEquals("Ch 1", result.archive.items[0].name)
        }

    @OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)
    @Test
    fun `should not return stale cache result when new chapters arrive`() =
        runTest {
            val sort = ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.ASCENDING)
            val volumeGroupsFlow = MutableStateFlow<List<VolumeChapterGroupDto>>(emptyList())
            val hasRootFlow = MutableStateFlow(false)

            val firstChapter = ChapterFileDto(id = 1L, name = "Ch 1", path = "/ch1.cbz", chapterSort = "1")
            val firstPage = ChapterPageDto(items = listOf(firstChapter), pageSize = 20, page = 0, total = 1)
            val localFlow = MutableStateFlow(firstPage)

            every { localReadGateway.observeChapters(1L, "NUMBER", true) } returns localFlow
            every { volumeGateway.observeVolumeGroups(1L, 20, "NUMBER", true) } returns volumeGroupsFlow
            every { volumeGateway.observeHasRootChapters(1L) } returns hasRootFlow

            val results = mutableListOf<ChapterDto?>()
            val job =
                launch {
                    useCase
                        .observeCombined(
                            comicId = 1L,
                            remoteId = null,
                            sort = sort,
                            page = 0,
                            pageSize = 20,
                            viewMode = VolumeViewType.CHAPTER,
                            volumeOverrides = emptyMap(),
                        ).collect { results.add(it) }
                }

            advanceUntilIdle()
            assertEquals(
                1,
                results
                    .last()!!
                    .archive.items.size,
            )

            val secondChapter = ChapterFileDto(id = 2L, name = "Ch 2", path = "/ch2.cbz", chapterSort = "2")
            localFlow.value = firstPage.copy(items = listOf(firstChapter, secondChapter), total = 2)
            advanceUntilIdle()

            assertEquals(
                2,
                results
                    .last()!!
                    .archive.items.size,
            )

            job.cancel()
        }
}
