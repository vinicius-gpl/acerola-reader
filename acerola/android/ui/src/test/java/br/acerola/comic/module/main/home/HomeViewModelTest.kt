package br.acerola.comic.module.main.home

import android.content.Context
import app.cash.turbine.test
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.adapter.contract.gateway.ComicGateway
import br.acerola.comic.adapter.contract.gateway.HistoryGateway
import br.acerola.comic.config.preference.ComicSortPreference
import br.acerola.comic.config.preference.HomeFilterPreference
import br.acerola.comic.config.preference.HomeLayoutPreference
import br.acerola.comic.config.preference.types.ComicSortType
import br.acerola.comic.config.preference.types.HomeLayoutType
import br.acerola.comic.config.preference.types.HomeSortPreference
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.module.main.home.state.FilterSettings
import br.acerola.comic.usecase.chapter.GetChapterCountUseCase
import br.acerola.comic.usecase.comic.DeleteComicUseCase
import br.acerola.comic.usecase.comic.HideComicUseCase
import br.acerola.comic.usecase.comic.ObserveLibraryUseCase
import br.acerola.comic.usecase.history.ObserveHistoryUseCase
import br.acerola.comic.usecase.metadata.ClearMetadataUseCase
import br.acerola.comic.usecase.metadata.ManageCategoriesUseCase
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncComicWithPeerUseCase
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.unmockkObject
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class HomeViewModelTest {
    @get:Rule
    val coroutineRule = MainDispatcherRule()

    private val statusRepository = mockk<br.acerola.comic.sync.LibrarySyncStatusRepository>(relaxed = true)
    private val historyGateway = mockk<HistoryGateway>(relaxed = true)
    private val context = mockk<Context>(relaxed = true)
    private val mangadexRepo = mockk<ComicGateway<ComicMetadataDto>>(relaxed = true)
    private val directoryRepo = mockk<ComicGateway<ComicDirectoryDto>>(relaxed = true)
    private val manageCategoriesUseCase = mockk<ManageCategoriesUseCase>(relaxed = true)
    private val hideComicUseCase = mockk<HideComicUseCase>(relaxed = true)
    private val deleteComicUseCase = mockk<DeleteComicUseCase>(relaxed = true)
    private val clearMetadataUseCase = mockk<ClearMetadataUseCase>(relaxed = true)
    private val getChapterCountUseCase = mockk<GetChapterCountUseCase>(relaxed = true)
    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private val syncComicWithPeerUseCase = mockk<SyncComicWithPeerUseCase>(relaxed = true)

    private lateinit var observeHistoryUseCase: ObserveHistoryUseCase
    private lateinit var mangadexObserve: ObserveLibraryUseCase<ComicMetadataDto>
    private lateinit var directoryObserve: ObserveLibraryUseCase<ComicDirectoryDto>

    private val directoryFlow = MutableStateFlow<List<ComicDirectoryDto>>(emptyList())
    private val metadataFlow = MutableStateFlow<List<ComicMetadataDto>>(emptyList())
    private val chapterCountFlow = MutableStateFlow<Map<Long, Int>>(emptyMap())
    private val categoryMapFlow = MutableStateFlow<Map<Long, CategoryDto>>(emptyMap())

    private lateinit var viewModel: HomeViewModel

    @Before
    fun setup() {
        mockkObject(AcerolaLogger)
        mockkObject(HomeLayoutPreference)
        mockkObject(ComicSortPreference)
        mockkObject(HomeFilterPreference)

        every { AcerolaLogger.d(any(), any(), any()) } returns Unit
        every { AcerolaLogger.audit(any(), any(), any(), any()) } returns Unit
        every { AcerolaLogger.audit(any(), any(), any()) } returns Unit

        every { HomeLayoutPreference.layoutFlow(any()) } returns flowOf(HomeLayoutType.LIST)
        every { ComicSortPreference.sortFlow(any()) } returns flowOf(HomeSortPreference(ComicSortType.TITLE, SortDirection.ASCENDING))
        every { HomeFilterPreference.showHiddenFlow(any()) } returns flowOf(false)

        coEvery { HomeLayoutPreference.saveLayout(any(), any()) } returns Unit
        coEvery { ComicSortPreference.saveSort(any(), any()) } returns Unit
        coEvery { HomeFilterPreference.saveShowHidden(any(), any()) } returns Unit

        every { historyGateway.getAllRecentHistory() } returns MutableStateFlow(emptyList())
        every { mangadexRepo.observeLibrary() } returns metadataFlow
        every { directoryRepo.observeLibrary() } returns directoryFlow
        every { getChapterCountUseCase() } returns chapterCountFlow
        every { manageCategoriesUseCase.getAllComicCategories() } returns categoryMapFlow
        every { manageCategoriesUseCase.getAllCategories() } returns MutableStateFlow(emptyList())

        observeHistoryUseCase = ObserveHistoryUseCase(historyGateway)
        mangadexObserve = ObserveLibraryUseCase(comicRepository = mangadexRepo)
        directoryObserve = ObserveLibraryUseCase(comicRepository = directoryRepo)

        viewModel = createViewModel()
    }

    @After
    fun tearDown() {
        unmockkObject(AcerolaLogger)
        unmockkObject(HomeLayoutPreference)
        unmockkObject(ComicSortPreference)
        unmockkObject(HomeFilterPreference)
    }

    private fun createViewModel() =
        HomeViewModel(
            statusRepository = statusRepository,
            observeHistoryUseCase = observeHistoryUseCase,
            context = context,
            mangadexObserve = mangadexObserve,
            directoryObserve = directoryObserve,
            manageCategoriesUseCase = manageCategoriesUseCase,
            hideComicUseCase = hideComicUseCase,
            deleteComicUseCase = deleteComicUseCase,
            clearMetadataUseCase = clearMetadataUseCase,
            getChapterCountUseCase = getChapterCountUseCase,
            p2pUseCase = p2pUseCase,
            syncComicWithPeerUseCase = syncComicWithPeerUseCase,
        )

    @Test
    fun `should filter hidden comics by default`() =
        runTest {
            val comic1 =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 1L
                    every { name } returns "Comic 1"
                    every { hidden } returns false
                }
            val comic2 =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 2L
                    every { name } returns "Comic 2"
                    every { hidden } returns true
                }

            directoryFlow.value = listOf(comic1, comic2)

            viewModel.comics.test {
                var items = awaitItem()
                while (items == null) items = awaitItem()
                assertThat(items).hasSize(1)
                assertThat(items[0].first.directory.id).isEqualTo(1L)
            }
        }

    @Test
    fun `should show hidden comics when filter is active`() =
        runTest {
            val comic1 =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 1L
                    every { hidden } returns false
                }
            val comic2 =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 2L
                    every { hidden } returns true
                }

            directoryFlow.value = listOf(comic1, comic2)
            viewModel.updateFilterSettings(FilterSettings(showHidden = true))

            viewModel.comics.test {
                var items = awaitItem()
                while (items == null || items.size == 1) items = awaitItem()
                assertThat(items).hasSize(2)
            }
        }

    @Test
    fun `should sort comics by title`() =
        runTest {
            val comicA =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 1L
                    every { name } returns "B Comic"
                    every { hidden } returns false
                }
            val comicB =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 2L
                    every { name } returns "A Comic"
                    every { hidden } returns false
                }

            directoryFlow.value = listOf(comicA, comicB)
            viewModel.updateSortSettings(HomeSortPreference(ComicSortType.TITLE, SortDirection.ASCENDING))

            viewModel.comics.test {
                var items = awaitItem()
                while (items == null || (items.size == 2 && items[0].first.directory.name == "B Comic")) items = awaitItem()
                assertThat(items[0].first.directory.name).isEqualTo("A Comic")
                assertThat(items[1].first.directory.name).isEqualTo("B Comic")
            }
        }

    @Test
    fun `should sort comics by title descending`() =
        runTest {
            val comicA =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 1L
                    every { name } returns "A Comic"
                    every { hidden } returns false
                }
            val comicB =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 2L
                    every { name } returns "B Comic"
                    every { hidden } returns false
                }

            directoryFlow.value = listOf(comicA, comicB)
            viewModel.updateSortSettings(HomeSortPreference(ComicSortType.TITLE, SortDirection.DESCENDING))

            viewModel.comics.test {
                var items = awaitItem()
                while (items == null || (items.size == 2 && items[0].first.directory.name == "A Comic")) items = awaitItem()
                assertThat(items[0].first.directory.name).isEqualTo("B Comic")
                assertThat(items[1].first.directory.name).isEqualTo("A Comic")
            }
        }

    @Test
    fun `should filter by category`() =
        runTest {
            val cat1 =
                CategoryDto(id = 10L, name = "Cat 1", color = 0)

            val comic1 =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 1L
                    every { hidden } returns false
                }
            val comic2 =
                mockk<ComicDirectoryDto>(relaxed = true) {
                    every { id } returns 2L
                    every { hidden } returns false
                }

            directoryFlow.value = listOf(comic1, comic2)
            categoryMapFlow.value = mapOf(1L to cat1)

            viewModel.updateFilterSettings(FilterSettings(bookmarkCategoryId = 10L))

            viewModel.comics.test {
                var items = awaitItem()
                while (items == null || items.size == 2) items = awaitItem()
                assertThat(items).hasSize(1)
                assertThat(items[0].first.directory.id).isEqualTo(1L)
            }
        }

    @Test
    fun `should toggle comics selection`() =
        runTest {
            assertThat(viewModel.selectedComicIds.value).isEmpty()

            viewModel.toggleComicSelection(1L)
            assertThat(viewModel.selectedComicIds.value).containsExactly(1L)

            viewModel.toggleComicSelection(2L)
            assertThat(viewModel.selectedComicIds.value).containsExactly(1L, 2L)

            viewModel.toggleComicSelection(1L)
            assertThat(viewModel.selectedComicIds.value).containsExactly(2L)

            viewModel.clearComicSelection()
            assertThat(viewModel.selectedComicIds.value).isEmpty()
        }

    @Test
    fun `should select all and clear selection`() =
        runTest {
            viewModel.selectAllComics(listOf(1L, 2L, 3L))
            assertThat(viewModel.selectedComicIds.value).containsExactly(1L, 2L, 3L)

            viewModel.clearComicSelection()
            assertThat(viewModel.selectedComicIds.value).isEmpty()
        }

    @Test
    fun `should hide selected comics in batch`() =
        runTest {
            coEvery { hideComicUseCase(any(), any()) } returns arrow.core.Either.Right(Unit)

            viewModel.selectAllComics(listOf(10L, 20L))
            viewModel.hideSelectedComics()

            assertThat(viewModel.selectedComicIds.value).isEmpty()
            io.mockk.coVerify(exactly = 1) { hideComicUseCase(10L, hidden = true) }
            io.mockk.coVerify(exactly = 1) { hideComicUseCase(20L, hidden = true) }
        }

    @Test
    fun `should unhide selected comics in batch`() =
        runTest {
            coEvery { hideComicUseCase(any(), any()) } returns arrow.core.Either.Right(Unit)

            viewModel.selectAllComics(listOf(10L, 20L))
            viewModel.unhideSelectedComics()

            assertThat(viewModel.selectedComicIds.value).isEmpty()
            io.mockk.coVerify(exactly = 1) { hideComicUseCase(10L, hidden = false) }
            io.mockk.coVerify(exactly = 1) { hideComicUseCase(20L, hidden = false) }
        }

    @Test
    fun `should delete selected comics in batch`() =
        runTest {
            coEvery { deleteComicUseCase(any()) } returns arrow.core.Either.Right(Unit)

            viewModel.selectAllComics(listOf(100L, 200L))
            viewModel.deleteSelectedComics()

            assertThat(viewModel.selectedComicIds.value).isEmpty()
            io.mockk.coVerify(exactly = 1) { deleteComicUseCase(100L) }
            io.mockk.coVerify(exactly = 1) { deleteComicUseCase(200L) }
        }
}
