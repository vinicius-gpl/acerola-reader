package br.acerola.comic.module.main.home

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.config.preference.ComicSortPreference
import br.acerola.comic.config.preference.HomeFilterPreference
import br.acerola.comic.config.preference.HomeLayoutPreference
import br.acerola.comic.config.preference.types.ComicSortType
import br.acerola.comic.config.preference.types.HomeLayoutType
import br.acerola.comic.config.preference.types.HomeSortPreference
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.history.ReadingHistoryDto
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.error.UserMessage
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.home.state.FilterSettings
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.type.UiText
import br.acerola.comic.ui.R
import br.acerola.comic.usecase.DirectoryCase
import br.acerola.comic.usecase.MangadexCase
import br.acerola.comic.usecase.chapter.GetChapterCountUseCase
import br.acerola.comic.usecase.comic.DeleteComicUseCase
import br.acerola.comic.usecase.comic.HideComicUseCase
import br.acerola.comic.usecase.comic.ObserveLibraryUseCase
import br.acerola.comic.usecase.history.ObserveHistoryUseCase
import br.acerola.comic.usecase.metadata.ClearMetadataUseCase
import br.acerola.comic.usecase.metadata.ManageCategoriesUseCase
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncComicWithPeerUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

private data class HomeCombinedArgs(
    val comicDirectories: List<ComicDirectoryDto>,
    val remoteMangaInfo: List<ComicMetadataDto>,
    val historyList: List<ReadingHistoryDto>,
    val categoryMap: Map<Long, CategoryDto>,
    val chapterCounts: Map<Long, Int>,
)

@HiltViewModel
class HomeViewModel
    @Inject
    constructor(
        statusRepository: br.acerola.comic.sync.LibrarySyncStatusRepository,
        observeHistoryUseCase: ObserveHistoryUseCase,
        getChapterCountUseCase: GetChapterCountUseCase,
        private val manageCategoriesUseCase: ManageCategoriesUseCase,
        private val hideComicUseCase: HideComicUseCase,
        private val deleteComicUseCase: DeleteComicUseCase,
        private val clearMetadataUseCase: ClearMetadataUseCase,
        @param:ApplicationContext private val context: Context,
        @param:MangadexCase private val mangadexObserve: ObserveLibraryUseCase<ComicMetadataDto>,
        @param:DirectoryCase private val directoryObserve: ObserveLibraryUseCase<ComicDirectoryDto>,
        private val p2pUseCase: P2pUseCase,
        private val syncComicWithPeerUseCase: SyncComicWithPeerUseCase,
    ) : ViewModel() {
        private val _uiEvents = Channel<UserMessage>(capacity = Channel.BUFFERED)
        val uiEvents: Flow<UserMessage> = _uiEvents.receiveAsFlow()

        private val _pairedPeers = MutableStateFlow<List<PairedPeer>>(emptyList())
        val pairedPeers: StateFlow<List<PairedPeer>> = _pairedPeers.asStateFlow()

        private val _selectedHomeLayout = MutableStateFlow(value = HomeLayoutType.LIST)
        val selectedHomeLayout: StateFlow<HomeLayoutType> = _selectedHomeLayout.asStateFlow()

        private val _sortSettings = MutableStateFlow(HomeSortPreference(ComicSortType.TITLE, SortDirection.ASCENDING))
        val sortSettings: StateFlow<HomeSortPreference> = _sortSettings.asStateFlow()

        private val _filterSettings = MutableStateFlow(FilterSettings())
        val filterSettings: StateFlow<FilterSettings> = _filterSettings.asStateFlow()

        private val _searchQuery = MutableStateFlow("")
        val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

        private val _isSearchExpanded = MutableStateFlow(false)
        val isSearchExpanded: StateFlow<Boolean> = _isSearchExpanded.asStateFlow()

        val allCategories: StateFlow<List<CategoryDto>> =
            manageCategoriesUseCase
                .getAllCategories()
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(stopTimeoutMillis = 5_000), emptyList())

        val isIndexing: StateFlow<Boolean> = statusRepository.isIndexing
        val progress: StateFlow<Int> = statusRepository.progress

        val comics: StateFlow<List<Triple<ComicDto, ReadingHistoryDto?, Int>>?> =
            combine(
                combine(
                    directoryObserve(),
                    mangadexObserve(),
                    observeHistoryUseCase.invokeRecent(),
                    manageCategoriesUseCase.getAllComicCategories(),
                    getChapterCountUseCase(),
                ) { directories, remote, history, categories, counts ->
                    HomeCombinedArgs(directories, remote, history, categories, counts)
                },
                _sortSettings,
                _filterSettings,
                _searchQuery,
            ) { args, sort, filter, query ->
                val remoteInfoMap =
                    args.remoteMangaInfo
                        .filter { it.comicDirectoryFk != null }
                        .associateBy { it.comicDirectoryFk!! }

                val historyMap = args.historyList.associateBy { it.comicDirectoryId }

                val list =
                    args.comicDirectories
                        .filter { directory ->
                            val matchesHidden = filter.showHidden || !directory.hidden
                            val matchesCategory =
                                filter.bookmarkCategoryId == null || args.categoryMap[directory.id]?.id == filter.bookmarkCategoryId

                            val source = remoteInfoMap[directory.id]?.syncSource?.displayName
                            val matchesSource =
                                when (filter.metadataSource) {
                                    null -> true
                                    "NONE" -> source == null
                                    else -> source == filter.metadataSource
                                }

                            val title = remoteInfoMap[directory.id]?.title ?: directory.name
                            val matchesQuery = query.isEmpty() || title.contains(query, ignoreCase = true)

                            matchesHidden && matchesCategory && matchesSource && matchesQuery
                        }.map { directory ->
                            val comic =
                                ComicDto(
                                    directory = directory,
                                    remoteInfo = remoteInfoMap[directory.id],
                                    category = args.categoryMap[directory.id],
                                )
                            Triple(comic, historyMap[directory.id], args.chapterCounts[directory.id] ?: 0)
                        }

                val sortedList =
                    when (sort.type) {
                        ComicSortType.TITLE -> list.sortedBy { it.first.remoteInfo?.title ?: it.first.directory.name }
                        ComicSortType.CHAPTER_COUNT -> list.sortedBy { it.third }
                        ComicSortType.LAST_UPDATE -> list.sortedBy { it.first.directory.lastModified }
                    }

                val finalList = if (sort.direction == SortDirection.DESCENDING) sortedList.reversed() else sortedList

                AcerolaLogger.d(TAG, "Library loaded: ${finalList.size} comics found", LogSource.VIEWMODEL)
                finalList
            }.stateIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(stopTimeoutMillis = 5000),
                initialValue = null,
            )

        init {
            observeHomeLayout()
            observeSortSettings()
            observeFilterSettings()
        }

        private val _selectedComicIds = MutableStateFlow<Set<Long>>(emptySet())
        val selectedComicIds: StateFlow<Set<Long>> = _selectedComicIds.asStateFlow()

        fun toggleComicSelection(comicId: Long) {
            _selectedComicIds.value =
                if (_selectedComicIds.value.contains(comicId)) {
                    _selectedComicIds.value - comicId
                } else {
                    _selectedComicIds.value + comicId
                }
        }

        fun selectAllComics(allIds: List<Long>) {
            _selectedComicIds.value = allIds.toSet()
        }

        fun clearComicSelection() {
            _selectedComicIds.value = emptySet()
        }

        fun hideManga(comicId: Long) {
            viewModelScope.launch {
                hideComicUseCase(comicId).onLeft { error ->
                    _uiEvents.send(error)
                }
            }
        }

        fun setHideSelectedComics(hidden: Boolean) {
            val idsToUpdate = _selectedComicIds.value
            viewModelScope.launch {
                idsToUpdate.forEach { id ->
                    hideComicUseCase(id, hidden = hidden).onLeft { error ->
                        _uiEvents.send(error)
                    }
                }
                clearComicSelection()
            }
        }

        fun hideSelectedComics() {
            setHideSelectedComics(hidden = true)
        }

        fun unhideSelectedComics() {
            setHideSelectedComics(hidden = false)
        }

        fun deleteComic(comicId: Long) {
            viewModelScope.launch {
                deleteComicUseCase(comicId).onLeft { error ->
                    _uiEvents.send(error)
                }
            }
        }

        fun deleteSelectedComics() {
            val idsToDelete = _selectedComicIds.value
            viewModelScope.launch {
                idsToDelete.forEach { id ->
                    deleteComicUseCase(id).onLeft { error ->
                        _uiEvents.send(error)
                    }
                }
                clearComicSelection()
            }
        }

        fun clearMetadata(comicId: Long) {
            viewModelScope.launch {
                clearMetadataUseCase(comicId)
            }
        }

        fun clearMetadataForSelectedComics() {
            val idsToUpdate = _selectedComicIds.value
            viewModelScope.launch {
                idsToUpdate.forEach { id -> clearMetadataUseCase(id) }
                clearComicSelection()
            }
        }

        /** Carrega os peers pareados pro `PeerPickerSheet` — sob demanda, não reativo.
         *  `deviceName` já vem resolvido de `getPairedPeers()` (via `known_peers()` do lado
         *  nativo, persistente) — não precisa cruzar com `getConnectedPeersWithInfo()`, que só
         *  tem dado durante os poucos segundos em que a conexão de handshake está aberta. */
        fun loadPairedPeers() {
            viewModelScope.launch(Dispatchers.IO) {
                _pairedPeers.value = p2pUseCase.getPairedPeers().map { PairedPeer(peerId = it.id, deviceName = it.deviceName) }
            }
        }

        fun syncComicWithPeer(
            peerId: String,
            comicName: String,
            direction: SyncDirection,
        ) {
            AcerolaLogger.audit(
                TAG,
                "Syncing comic with peer",
                LogSource.VIEWMODEL,
                mapOf("peerId" to peerId, "comicName" to comicName, "direction" to direction.name),
            )

            val fired = syncComicWithPeerUseCase(peerId, comicName, direction)
            if (!fired) {
                viewModelScope.launch {
                    _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.error_sync_comic_peer_not_paired)))
                }
            }
        }

        fun setMangaCategory(
            comicId: Long,
            categoryId: Long?,
        ) {
            viewModelScope.launch {
                manageCategoriesUseCase.updateComicCategory(comicId, categoryId)
            }
        }

        fun setSelectedComicsCategory(categoryId: Long?) {
            val idsToUpdate = _selectedComicIds.value
            viewModelScope.launch {
                idsToUpdate.forEach { id ->
                    manageCategoriesUseCase.updateComicCategory(id, categoryId)
                }
                clearComicSelection()
            }
        }

        fun updateHomeLayout(layout: HomeLayoutType) {
            if (_selectedHomeLayout.value == layout) return
            _selectedHomeLayout.value = layout

            AcerolaLogger.audit(TAG, "User changed home layout to ${layout.name}", LogSource.VIEWMODEL)

            viewModelScope.launch {
                HomeLayoutPreference.saveLayout(context, layout)
            }
        }

        fun updateSortSettings(sort: HomeSortPreference) {
            _sortSettings.value = sort
            viewModelScope.launch {
                ComicSortPreference.saveSort(context, sort)
            }
        }

        fun updateFilterSettings(filter: FilterSettings) {
            val oldShowHidden = _filterSettings.value.showHidden
            _filterSettings.value = filter

            if (oldShowHidden != filter.showHidden) {
                viewModelScope.launch {
                    HomeFilterPreference.saveShowHidden(context, filter.showHidden)
                }
            }
        }

        fun updateSearchQuery(query: String) {
            _searchQuery.value = query
        }

        fun setSearchExpanded(expanded: Boolean) {
            _isSearchExpanded.value = expanded
            if (!expanded) {
                _searchQuery.value = ""
            }
        }

        private fun observeHomeLayout() {
            viewModelScope.launch {
                HomeLayoutPreference.layoutFlow(context).collect { layout ->
                    if (_selectedHomeLayout.value != layout) {
                        _selectedHomeLayout.value = layout
                    }
                }
            }
        }

        private fun observeSortSettings() {
            viewModelScope.launch {
                ComicSortPreference.sortFlow(context).collect { sort ->
                    _sortSettings.value = sort
                }
            }
        }

        private fun observeFilterSettings() {
            viewModelScope.launch {
                HomeFilterPreference.showHiddenFlow(context).collect { showHidden ->
                    if (_filterSettings.value.showHidden != showHidden) {
                        _filterSettings.value = _filterSettings.value.copy(showHidden = showHidden)
                    }
                }
            }
        }

        companion object {
            private const val TAG = "HomeViewModel"
        }
    }
