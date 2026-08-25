package br.acerola.comic.module.main.history
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.error.UserMessage
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.history.state.HistoryItemState
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.service.network.P2pEvent
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.type.UiText
import br.acerola.comic.ui.R
import br.acerola.comic.usecase.DirectoryCase
import br.acerola.comic.usecase.MangadexCase
import br.acerola.comic.usecase.chapter.GetChapterCountUseCase
import br.acerola.comic.usecase.comic.ObserveLibraryUseCase
import br.acerola.comic.usecase.history.ObserveHistoryUseCase
import br.acerola.comic.usecase.metadata.ManageCategoriesUseCase
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncHistoryWithPeerUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class HistoryViewModel
    @Inject
    constructor(
        observeHistoryUseCase: ObserveHistoryUseCase,
        @param:MangadexCase private val mangadexObserve: ObserveLibraryUseCase<ComicMetadataDto>,
        @param:DirectoryCase private val directoryObserve: ObserveLibraryUseCase<ComicDirectoryDto>,
        private val manageCategoriesUseCase: ManageCategoriesUseCase,
        private val getChapterCountUseCase: GetChapterCountUseCase,
        private val p2pUseCase: P2pUseCase,
        private val syncHistoryWithPeerUseCase: SyncHistoryWithPeerUseCase,
        private val p2pEventBus: P2pEventBus,
    ) : ViewModel() {
        private val _uiEvents = Channel<UserMessage>(capacity = Channel.BUFFERED)
        val uiEvents: Flow<UserMessage> = _uiEvents.receiveAsFlow()

        private val _pairedPeers = MutableStateFlow<List<PairedPeer>>(emptyList())
        val pairedPeers: StateFlow<List<PairedPeer>> = _pairedPeers.asStateFlow()

        // Histórico só dispara UMA sessão de sync por vez (um único botão, não um por peer) —
        // mesma forma simplificada usada por `ComicViewModel._syncingPeerId`, que também não
        // precisa de granularidade por peer nesse contexto.
        private val _syncingPeerId = MutableStateFlow<String?>(null)
        val isSyncingWithPeer: StateFlow<Boolean> =
            _syncingPeerId
                .map { it != null }
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

        init {
            viewModelScope.launch {
                p2pEventBus.events.collect { event ->
                    val pendingPeerId = _syncingPeerId.value ?: return@collect
                    when (event) {
                        is P2pEvent.HistorySyncComplete ->
                            if (event.peerId == pendingPeerId) _syncingPeerId.value = null

                        is P2pEvent.HistorySyncError ->
                            if (event.peerId == pendingPeerId) {
                                _syncingPeerId.value = null
                                _uiEvents.send(UserMessage.Raw(event.message))
                            }

                        else -> Unit
                    }
                }
            }
        }

        /** Carrega os peers pareados pro `PeerPickerSheet` — sob demanda (não reativo), mesmo
         *  padrão de [br.acerola.comic.module.comic.ComicViewModel.loadPairedPeers]. */
        fun loadPairedPeers() {
            viewModelScope.launch(Dispatchers.IO) {
                _pairedPeers.value = p2pUseCase.getPairedPeers().map { PairedPeer(peerId = it.id, deviceName = it.deviceName) }
            }
        }

        fun syncHistoryWithPeer(peerId: String) {
            AcerolaLogger.audit(TAG, "Syncing history with peer", LogSource.VIEWMODEL, mapOf("peerId" to peerId))

            val fired = syncHistoryWithPeerUseCase(peerId)
            if (!fired) {
                viewModelScope.launch {
                    _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.error_sync_comic_peer_not_paired)))
                }
                return
            }

            _syncingPeerId.value = peerId
        }

        @OptIn(ExperimentalCoroutinesApi::class)
        val historyItems: StateFlow<List<HistoryItemState>> =
            observeHistoryUseCase()
                .flatMapLatest { historyList ->
                    combine(
                        directoryObserve(),
                        mangadexObserve(),
                        manageCategoriesUseCase.getAllComicCategories(),
                        getChapterCountUseCase(),
                    ) { directories, remoteInfos, categoryMap, chapterCounts ->
                        val list =
                            historyList.mapNotNull { history ->
                                val directory = directories.find { it.id == history.comicDirectoryId } ?: return@mapNotNull null
                                val remote = remoteInfos.find { it.comicDirectoryFk == history.comicDirectoryId }
                                HistoryItemState(
                                    comic =
                                        ComicDto(
                                            directory = directory,
                                            remoteInfo = remote,
                                            category = categoryMap[directory.id],
                                        ),
                                    history = history,
                                    chapterCount = chapterCounts[directory.id] ?: 0,
                                )
                            }
                        AcerolaLogger.d(TAG, "History items updated: ${list.size} items found", LogSource.VIEWMODEL)
                        list
                    }
                }.stateIn(
                    scope = viewModelScope,
                    started = SharingStarted.WhileSubscribed(5000),
                    initialValue = emptyList(),
                )

        companion object {
            private const val TAG = "HistoryViewModel"
        }
    }
