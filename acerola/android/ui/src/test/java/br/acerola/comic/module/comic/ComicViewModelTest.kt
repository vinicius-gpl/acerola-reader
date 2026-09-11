package br.acerola.comic.module.comic

import android.content.Context
import android.util.Log
import app.cash.turbine.test
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.adapter.contract.gateway.ChapterReadGateway
import br.acerola.comic.adapter.contract.gateway.ChapterSyncStatusGateway
import br.acerola.comic.adapter.contract.gateway.ComicGateway
import br.acerola.comic.adapter.contract.gateway.HistoryGateway
import br.acerola.comic.config.preference.ChapterPerPagePreference
import br.acerola.comic.config.preference.ChapterSortPreference
import br.acerola.comic.config.preference.VolumeViewPreference
import br.acerola.comic.config.preference.types.ChapterPageSizeType
import br.acerola.comic.config.preference.types.ChapterSortPreferenceData
import br.acerola.comic.config.preference.types.ChapterSortType
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.dto.ChapterDto
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.dto.archive.ChapterPageDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.archive.VolumeArchiveDto
import br.acerola.comic.dto.archive.VolumeChapterGroupDto
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.service.cache.ChapterCacheHandler
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.chapter.ObserveChaptersUseCase
import br.acerola.comic.usecase.chapter.ObserveCombinedChaptersUseCase
import br.acerola.comic.usecase.chapter.ObserveVolumeChaptersUseCase
import br.acerola.comic.usecase.comic.ObserveLibraryUseCase
import br.acerola.comic.usecase.history.ObserveComicHistoryUseCase
import br.acerola.comic.usecase.history.TrackReadingProgressUseCase
import br.acerola.comic.usecase.metadata.ExtractAllVolumeCoversUseCase
import br.acerola.comic.usecase.metadata.ExtractVolumeCoverUseCase
import br.acerola.comic.usecase.metadata.ManageCategoriesUseCase
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncComicWithPeerUseCase
import br.acerola.comic.usecase.network.SyncHistoryEntryWithPeerUseCase
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.mockkStatic
import io.mockk.unmockkObject
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@OptIn(ExperimentalCoroutinesApi::class)
@RunWith(RobolectricTestRunner::class)
class ComicViewModelTest {
    @get:Rule
    val coroutineRule = MainDispatcherRule()

    private val historyGateway = mockk<HistoryGateway>(relaxed = true)
    private val trackReadingProgressUseCase = mockk<TrackReadingProgressUseCase>(relaxed = true)
    private val context = mockk<Context>(relaxed = true)
    private val mangadexRepo = mockk<ComicGateway<ComicMetadataDto>>(relaxed = true)
    private val directoryRepo = mockk<ComicGateway<ComicDirectoryDto>>(relaxed = true)
    private val directoryChapterReadRepo = mockk<ChapterReadGateway<ChapterPageDto>>(relaxed = true)
    private val chapterSyncStatusGateway = mockk<ChapterSyncStatusGateway>(relaxed = true)
    private val manageCategoriesUseCase = mockk<ManageCategoriesUseCase>(relaxed = true)

    private lateinit var observeComicHistoryUseCase: ObserveComicHistoryUseCase
    private lateinit var mangadexObserve: ObserveLibraryUseCase<ComicMetadataDto>
    private lateinit var directoryObserve: ObserveLibraryUseCase<ComicDirectoryDto>
    private val observeChaptersUseCase = mockk<ObserveCombinedChaptersUseCase>(relaxed = true)
    private lateinit var observeAllChaptersUseCase: ObserveChaptersUseCase<ChapterPageDto>
    private val allChaptersFlow = MutableStateFlow(ChapterPageDto(emptyList(), emptyList(), 20, 0, 0))
    private val directoryObserveVolumeChapters = mockk<ObserveVolumeChaptersUseCase>(relaxed = true)
    private val extractVolumeCoverUseCase = mockk<ExtractVolumeCoverUseCase>(relaxed = true)
    private val extractAllVolumeCoversUseCase = mockk<ExtractAllVolumeCoversUseCase>(relaxed = true)
    private val cacheHandler = mockk<ChapterCacheHandler>(relaxed = true)
    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private val syncComicWithPeerUseCase = mockk<SyncComicWithPeerUseCase>(relaxed = true)
    private val syncHistoryEntryWithPeerUseCase = mockk<SyncHistoryEntryWithPeerUseCase>(relaxed = true)
    private val p2pEventBus = mockk<P2pEventBus>(relaxed = true)

    private val localChaptersFlow = MutableStateFlow(ChapterPageDto(emptyList(), emptyList(), 20, 0, 0))
    private val hasRootChaptersFlow = MutableStateFlow(true)
    private val volumeSectionsFlow = MutableStateFlow<List<VolumeChapterGroupDto>>(emptyList())

    private lateinit var viewModel: ComicViewModel

    @Before
    fun setup() {
        mockkObject(AcerolaLogger)
        mockkStatic(Log::class)
        mockkObject(ChapterPerPagePreference)
        mockkObject(ChapterSortPreference)
        mockkObject(VolumeViewPreference)

        every { Log.v(any(), any()) } returns 0
        every { Log.d(any(), any()) } returns 0
        every { Log.i(any(), any()) } returns 0
        every { Log.w(any<String>(), any<String>()) } returns 0
        every { Log.e(any(), any()) } returns 0

        every { AcerolaLogger.d(any<String>(), any<String>(), any<LogSource>()) } returns Unit
        every { AcerolaLogger.i(any<String>(), any<String>(), any<LogSource>()) } returns Unit
        every { AcerolaLogger.v(any<String>(), any<String>(), any<LogSource>()) } returns Unit
        every { AcerolaLogger.w(any<String>(), any<String>(), any<LogSource>(), any()) } returns Unit
        every { AcerolaLogger.e(any<String>(), any<String>(), any<LogSource>(), any()) } returns Unit
        every { AcerolaLogger.audit(any<String>(), any<String>(), any<LogSource>(), any()) } returns Unit

        every { ChapterPerPagePreference.chapterPerPageFlow(any()) } returns flowOf(ChapterPageSizeType.SHORT)
        every { ChapterSortPreference.sortFlow(any()) } returns
            flowOf(ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.ASCENDING))
        every { VolumeViewPreference.volumeViewFlow(any()) } returns flowOf(VolumeViewType.CHAPTER)

        coEvery { ChapterPerPagePreference.saveChapterPerPage(any(), any()) } returns Unit
        coEvery { ChapterSortPreference.saveSort(any(), any()) } returns Unit
        coEvery { VolumeViewPreference.saveVolumeView(any(), any()) } returns Unit

        every { historyGateway.getHistoryByMangaId(any()) } returns MutableStateFlow(null)
        every { historyGateway.getReadChaptersByMangaId(any()) } returns MutableStateFlow(emptyList())
        every { mangadexRepo.observeLibrary() } returns MutableStateFlow(emptyList())
        every { directoryRepo.observeLibrary() } returns
            MutableStateFlow(listOf(mockk<ComicDirectoryDto>(relaxed = true) { every { id } returns 1L }))

        every { directoryRepo.isIndexing } returns MutableStateFlow(false)
        every { directoryRepo.progress } returns MutableStateFlow(-1)
        every { mangadexRepo.isIndexing } returns MutableStateFlow(false)
        every { mangadexRepo.progress } returns MutableStateFlow(-1)

        every { observeChaptersUseCase.isIndexing } returns MutableStateFlow(false)
        every { observeChaptersUseCase.progress } returns MutableStateFlow(-1)
        every { observeChaptersUseCase.observeCombined(any(), any(), any(), any(), any(), any(), any()) } answers {
            val sort = args[2] as ChapterSortPreferenceData
            localChaptersFlow.map { page ->
                val sortedItems =
                    when (sort.type) {
                        ChapterSortType.LAST_UPDATE ->
                            if (sort.direction == SortDirection.DESCENDING) {
                                page.items.sortedByDescending { it.lastModified }
                            } else {
                                page.items.sortedBy { it.lastModified }
                            }
                        else ->
                            if (sort.direction == SortDirection.DESCENDING) {
                                page.items.reversed()
                            } else {
                                page.items
                            }
                    }
                val showHeaders = page.volumes.size > 1 && !hasRootChaptersFlow.value
                ChapterDto(
                    archive = page.copy(items = sortedItems),
                    showVolumeHeaders = showHeaders,
                    hasVolumeStructure = page.volumes.size > 1,
                    effectiveViewMode = VolumeViewType.CHAPTER,
                )
            }
        }

        every { manageCategoriesUseCase.getCategoryByComicId(any()) } returns flowOf(null)
        every { directoryObserveVolumeChapters.observeByComic(any(), any(), any(), any()) } returns volumeSectionsFlow
        every { directoryObserveVolumeChapters.observeHasRootChapters(any()) } returns hasRootChaptersFlow
        coEvery { directoryObserveVolumeChapters.loadVolumePage(any(), any(), any(), any(), any(), any()) } returns emptyList()
        every { directoryChapterReadRepo.observeChapters(any(), any(), any()) } returns allChaptersFlow
        every { p2pEventBus.events } returns MutableSharedFlow()

        observeComicHistoryUseCase = ObserveComicHistoryUseCase(historyGateway)
        mangadexObserve = ObserveLibraryUseCase(comicRepository = mangadexRepo)
        directoryObserve = ObserveLibraryUseCase(comicRepository = directoryRepo)
        observeAllChaptersUseCase =
            ObserveChaptersUseCase(
                readGateway = directoryChapterReadRepo,
                syncStatusGateway = chapterSyncStatusGateway,
            )

        viewModel = createViewModel()
        viewModel.init(1L, null)
    }

    @After
    fun tearDown() {
        unmockkObject(AcerolaLogger)
        unmockkObject(ChapterPerPagePreference)
        unmockkObject(ChapterSortPreference)
        unmockkObject(VolumeViewPreference)
    }

    private fun createViewModel() =
        ComicViewModel(
            context = context,
            observeComicHistoryUseCase = observeComicHistoryUseCase,
            trackReadingProgressUseCase = trackReadingProgressUseCase,
            mangadexObserve = mangadexObserve,
            directoryObserve = directoryObserve,
            observeChaptersUseCase = observeChaptersUseCase,
            observeAllChaptersUseCase = observeAllChaptersUseCase,
            directoryObserveVolumeChapters = directoryObserveVolumeChapters,
            manageCategoriesUseCase = manageCategoriesUseCase,
            extractVolumeCoverUseCase = extractVolumeCoverUseCase,
            extractAllVolumeCoversUseCase = extractAllVolumeCoversUseCase,
            cacheHandler = cacheHandler,
            p2pUseCase = p2pUseCase,
            syncComicWithPeerUseCase = syncComicWithPeerUseCase,
            syncHistoryEntryWithPeerUseCase = syncHistoryEntryWithPeerUseCase,
            p2pEventBus = p2pEventBus,
        )

    @Test
    fun `should sort chapters by number ascending`() =
        runTest {
            val cap1 = ChapterFileDto(id = 1L, name = "Cap 1", path = "", chapterSort = "1", lastModified = 1000L)
            val cap2 = ChapterFileDto(id = 2L, name = "Cap 2", path = "", chapterSort = "2", lastModified = 500L)

            viewModel.init(1L, null)
            // Fornecendo dados ordenados como o banco faria
            localChaptersFlow.value = ChapterPageDto(listOf(cap1, cap2), emptyList(), 20, 0, 2)
            viewModel.updateChapterSort(ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.ASCENDING))

            viewModel.chapters.test {
                // O estado inicial pode ser nulo
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) {
                    item = awaitItem()
                }

                assertThat(item.archive.items[0].id).isEqualTo(1L)
                assertThat(item.archive.items[1].id).isEqualTo(2L)
            }
        }

    @Test
    fun `should sort decimal chapters correctly - 0_01 before 0_02, 0_10 after 0_09`() =
        runTest {
            // Bug: "0.1".toDouble() == "0.10".toDouble() == 0.1
            // causava Ch.0.10 aparecer logo após Ch.0.01, antes de Ch.0.02
            val ch001 = ChapterFileDto(id = 1L, name = "Ch. 0.01", path = "", chapterSort = "0.1")
            val ch002 = ChapterFileDto(id = 2L, name = "Ch. 0.02", path = "", chapterSort = "0.2")
            val ch009 = ChapterFileDto(id = 9L, name = "Ch. 0.09", path = "", chapterSort = "0.9")
            val ch010 = ChapterFileDto(id = 10L, name = "Ch. 0.10", path = "", chapterSort = "0.10")
            val ch011 = ChapterFileDto(id = 11L, name = "Ch. 0.11", path = "", chapterSort = "0.11")

            // Fornecendo dados ordenados
            localChaptersFlow.value =
                ChapterPageDto(
                    listOf(ch001, ch002, ch009, ch010, ch011),
                    emptyList(),
                    20,
                    0,
                    5,
                )
            viewModel.updateChapterSort(ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.ASCENDING))

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) item = awaitItem()

                val ids = item.archive.items.map { it.id }
                assertThat(ids).isEqualTo(listOf(1L, 2L, 9L, 10L, 11L))
            }
        }

    @Test
    fun `should sort integer chapters correctly - 1 2 9 10 11 100`() =
        runTest {
            // Bug: "10".toDouble()=10.0 e "100".toDouble()=100.0 parecem ok,
            // mas "10" < "11" < "100" ficaria errado com sort textual.
            // Verifica que o sort numérico inteiro funciona.
            val ch1 = ChapterFileDto(id = 1L, name = "Ch. 1", path = "", chapterSort = "1")
            val ch2 = ChapterFileDto(id = 2L, name = "Ch. 2", path = "", chapterSort = "2")
            val ch9 = ChapterFileDto(id = 9L, name = "Ch. 9", path = "", chapterSort = "9")
            val ch10 = ChapterFileDto(id = 10L, name = "Ch. 10", path = "", chapterSort = "10")
            val ch11 = ChapterFileDto(id = 11L, name = "Ch. 11", path = "", chapterSort = "11")
            val ch100 = ChapterFileDto(id = 100L, name = "Ch. 100", path = "", chapterSort = "100")

            // Fornecendo dados ordenados
            localChaptersFlow.value =
                ChapterPageDto(
                    listOf(ch1, ch2, ch9, ch10, ch11, ch100),
                    emptyList(),
                    20,
                    0,
                    6,
                )
            viewModel.updateChapterSort(ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.ASCENDING))

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) item = awaitItem()

                val ids = item.archive.items.map { it.id }
                assertThat(ids).isEqualTo(listOf(1L, 2L, 9L, 10L, 11L, 100L))
            }
        }

    @Test
    fun `should sort mixed integer and decimal chapters correctly`() =
        runTest {
            val ch001 = ChapterFileDto(id = 1L, name = "Ch. 0.01", path = "", chapterSort = "0.1")
            val ch010 = ChapterFileDto(id = 2L, name = "Ch. 0.10", path = "", chapterSort = "0.10")
            val ch1 = ChapterFileDto(id = 3L, name = "Ch. 1", path = "", chapterSort = "1")
            val ch1dot5 = ChapterFileDto(id = 4L, name = "Ch. 1.5", path = "", chapterSort = "1.5")
            val ch2 = ChapterFileDto(id = 5L, name = "Ch. 2", path = "", chapterSort = "2")
            val ch10 = ChapterFileDto(id = 6L, name = "Ch. 10", path = "", chapterSort = "10")

            // Fornecendo dados ordenados
            localChaptersFlow.value =
                ChapterPageDto(
                    listOf(ch001, ch010, ch1, ch1dot5, ch2, ch10),
                    emptyList(),
                    20,
                    0,
                    6,
                )
            viewModel.updateChapterSort(ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.ASCENDING))

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) item = awaitItem()

                val ids = item.archive.items.map { it.id }
                assertThat(ids).isEqualTo(listOf(1L, 2L, 3L, 4L, 5L, 6L))
            }
        }

    @Test
    fun `should sort decimal chapters descending`() =
        runTest {
            val ch001 = ChapterFileDto(id = 1L, name = "Ch. 0.01", path = "", chapterSort = "0.1")
            val ch002 = ChapterFileDto(id = 2L, name = "Ch. 0.02", path = "", chapterSort = "0.2")
            val ch010 = ChapterFileDto(id = 3L, name = "Ch. 0.10", path = "", chapterSort = "0.10")

            // Fornecendo dados em ordem ASCENDENTE do banco. O ViewModel irá invertê-los.
            localChaptersFlow.value =
                ChapterPageDto(
                    listOf(ch001, ch002, ch010),
                    emptyList(),
                    20,
                    0,
                    3,
                )
            viewModel.updateChapterSort(ChapterSortPreferenceData(ChapterSortType.NUMBER, SortDirection.DESCENDING))

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) item = awaitItem()

                val ids = item.archive.items.map { it.id }
                assertThat(ids).isEqualTo(listOf(3L, 2L, 1L))
            }
        }

    @Test
    fun `should sort chapters by last update descending`() =
        runTest {
            val cap1 = ChapterFileDto(id = 1L, name = "Cap 1", path = "", chapterSort = "1", lastModified = 1000L)
            val cap2 = ChapterFileDto(id = 2L, name = "Cap 2", path = "", chapterSort = "2", lastModified = 2000L)

            localChaptersFlow.value = ChapterPageDto(listOf(cap1, cap2), emptyList(), 20, 0, 2)
            viewModel.updateChapterSort(ChapterSortPreferenceData(ChapterSortType.LAST_UPDATE, SortDirection.DESCENDING))

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) {
                    item = awaitItem()
                }

                assertThat(item.archive.items[0].id).isEqualTo(2L)
                assertThat(item.archive.items[1].id).isEqualTo(1L)
            }
        }

    @Test
    fun `should display headers only when there are multiple real volumes`() =
        runTest {
            val volume1 = VolumeArchiveDto(id = 10L, name = "Vol. 1", volumeSort = "1", isSpecial = false)
            val volume2 = VolumeArchiveDto(id = 20L, name = "Vol. 2", volumeSort = "2", isSpecial = false)
            hasRootChaptersFlow.value = false
            val cap1 = ChapterFileDto(id = 1L, name = "Ch. 1", path = "", chapterSort = "1", volumeId = 10L)
            val cap2 = ChapterFileDto(id = 2L, name = "Ch. 2", path = "", chapterSort = "2", volumeId = 20L)

            volumeSectionsFlow.value =
                listOf(
                    VolumeChapterGroupDto(volume1, listOf(cap1), 1, 1, false),
                    VolumeChapterGroupDto(volume2, listOf(cap2), 1, 1, false),
                )

            localChaptersFlow.value =
                ChapterPageDto(
                    items =
                        listOf(
                            ChapterFileDto(id = 1L, name = "Ch. 1", path = "", chapterSort = "1", volumeId = 10L),
                            ChapterFileDto(id = 2L, name = "Ch. 2", path = "", chapterSort = "2", volumeId = 20L),
                        ),
                    volumes = listOf(volume1, volume2),
                    pageSize = 20,
                    page = 0,
                    total = 2,
                )

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) item = awaitItem()
                assertThat(item.showVolumeHeaders).isTrue()
                cancelAndIgnoreRemainingEvents()
            }

            val cap3 = ChapterFileDto(id = 1L, name = "Ch. 1", path = "", chapterSort = "1", volumeId = 10L)
            val cap4 = ChapterFileDto(id = 2L, name = "Ch. 2", path = "", chapterSort = "2", volumeId = 10L)

            localChaptersFlow.value =
                ChapterPageDto(
                    items = listOf(cap3, cap4),
                    volumes = listOf(volume1),
                    pageSize = 20,
                    page = 0,
                    total = 2,
                )
            volumeSectionsFlow.value =
                listOf(
                    VolumeChapterGroupDto(volume1, listOf(cap3, cap4), 2, 2, false),
                )

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) item = awaitItem()
                assertThat(item.showVolumeHeaders).isFalse()
                cancelAndIgnoreRemainingEvents()
            }
        }

    @Test
    fun `should hide headers when there are mixed root chapters`() =
        runTest {
            val volume1 = VolumeArchiveDto(id = 10L, name = "Vol. 1", volumeSort = "1", isSpecial = false)
            val volume2 = VolumeArchiveDto(id = 20L, name = "Vol. 2", volumeSort = "2", isSpecial = false)
            hasRootChaptersFlow.value = true

            localChaptersFlow.value =
                ChapterPageDto(
                    items =
                        listOf(
                            ChapterFileDto(id = 1L, name = "Ch. 0", path = "", chapterSort = "0"),
                            ChapterFileDto(id = 2L, name = "Ch. 1", path = "", chapterSort = "1", volumeId = 10L),
                            ChapterFileDto(id = 3L, name = "Ch. 2", path = "", chapterSort = "2", volumeId = 20L),
                        ),
                    volumes = listOf(volume1, volume2),
                    pageSize = 20,
                    page = 0,
                    total = 3,
                )

            viewModel.chapters.test {
                var item = awaitItem()
                while (item == null || item.archive.items.isEmpty()) item = awaitItem()
                assertThat(item.showVolumeHeaders).isFalse()
                cancelAndIgnoreRemainingEvents()
            }
        }

    @Test
    fun `mark all as read should use full comic list, not only loaded page`() =
        runTest {
            val fullChapters =
                (1..5).map { n ->
                    ChapterFileDto(id = n.toLong(), name = "Ch. $n", path = "", chapterSort = n.toString())
                }

            // Lista completa vinda do banco (sem paginação)
            allChaptersFlow.value = ChapterPageDto(fullChapters, emptyList(), 20, 0, 5)
            // Apenas os 2 primeiros capítulos foram carregados na página atual (scroll infinito)
            localChaptersFlow.value = ChapterPageDto(fullChapters.take(2), emptyList(), 2, 0, 5)

            viewModel.allChapters.test {
                var item = awaitItem()
                while (item.size < 5) item = awaitItem()
                cancelAndIgnoreRemainingEvents()
            }

            viewModel.selectAllChapters(fullChapters.map { it.chapterSort })
            viewModel.markSelectedChaptersReadStatus(true)

            fullChapters.forEach { chapter ->
                coVerify(exactly = 1) {
                    trackReadingProgressUseCase.markChapterAsRead(1L, chapter.chapterSort, chapter.id)
                }
            }
        }
}
