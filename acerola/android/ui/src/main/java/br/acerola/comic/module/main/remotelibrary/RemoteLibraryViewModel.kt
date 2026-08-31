package br.acerola.comic.module.main.remotelibrary

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.error.UserMessage
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.remotelibrary.state.RemoteLibraryUiState
import br.acerola.comic.service.network.ComicSummary
import br.acerola.comic.service.network.P2pEvent
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.type.UiText
import br.acerola.comic.ui.R
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncComicWithPeerUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject

/**
 * Biblioteca remota de UM peer só (escolhido antes de abrir esta tela, via `PeerPickerSheet`) —
 * versão enxuta do que [br.acerola.comic.module.main.sync.SyncViewModel] já faz pra
 * `browseLibrary`/`fetchCoversFor`/`syncComic`, sem o resto do estado de pareamento/QR/log que
 * não faz sentido aqui.
 */
@HiltViewModel
class RemoteLibraryViewModel
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
        private val p2pEventBus: P2pEventBus,
        private val syncComicWithPeerUseCase: SyncComicWithPeerUseCase,
    ) : ViewModel() {
        private val _uiState = MutableStateFlow(RemoteLibraryUiState())
        val uiState: StateFlow<RemoteLibraryUiState> = _uiState.asStateFlow()

        private val _uiEvents = Channel<UserMessage>(capacity = Channel.BUFFERED)
        val uiEvents: Flow<UserMessage> = _uiEvents.receiveAsFlow()

        private var peerId: String? = null

        /** `comicName` -> última versão de capa confirmada nesta sessão — nunca rebaixa a mesma
         *  versão duas vezes (mesma ideia do `knownCoverVersions` de
         *  [br.acerola.comic.module.main.sync.SyncViewModel], mas escopado a UM peer só, então a
         *  chave é só o nome do quadrinho). */
        private val knownCoverVersions = ConcurrentHashMap<String, Long>()

        init {
            viewModelScope.launch {
                p2pEventBus.events.collect(::handleEvent)
            }
        }

        /** Chamado pela `Screen` assim que `peerId` é conhecido — idempotente pra recomposição
         *  não disparar `browseLibrary` de novo à toa. */
        fun init(
            peerId: String,
            peerDisplayName: String,
        ) {
            if (this.peerId == peerId) return
            this.peerId = peerId
            knownCoverVersions.clear()
            _uiState.value = RemoteLibraryUiState(peerDisplayName = peerDisplayName)
            browseLibrary(peerId)
        }

        private fun browseLibrary(peerId: String) {
            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(loaded = true) }
                    _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.error_sync_comic_peer_not_paired)))
                    return@launch
                }
                p2pUseCase.browseLibrary(addr)
            }
        }

        fun syncComic(comicName: String) {
            val peerId = peerId ?: return
            if (_uiState.value.syncingComicName != null) {
                // Toque no card enquanto ele só mostra o spinner de loading — sem cancelamento
                // real disponível no fluxo P2P hoje, o mínimo é avisar por que o toque não fez
                // nada, em vez de ignorar em silêncio.
                viewModelScope.launch {
                    _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.message_sync_comic_already_syncing)))
                }
                return
            }

            AcerolaLogger.audit(
                TAG,
                "Pulling comic from remote library",
                LogSource.VIEWMODEL,
                mapOf("peerId" to peerId, "comicName" to comicName),
            )

            val fired = syncComicWithPeerUseCase(peerId, comicName)
            if (!fired) {
                viewModelScope.launch {
                    _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.error_sync_comic_peer_not_paired)))
                }
                return
            }

            _uiState.update { it.copy(syncingComicName = comicName) }
        }

        /** Dispara `acerola/browse-cover/1` em paralelo pra cada quadrinho da lista — streams são
         *  baratas numa conexão já pooled por `(peer, alpn)`, então não há necessidade de
         *  serializar (mesma lógica de [br.acerola.comic.module.main.sync.SyncViewModel]). */
        private fun fetchCoversFor(comics: List<ComicSummary>) {
            val peerId = peerId ?: return
            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId } ?: return@launch
                comics.forEach { comic ->
                    if (knownCoverVersions[comic.comicName] == comic.coverVersion) return@forEach
                    p2pUseCase.browseCover(addr, comic.comicName, knownCoverVersions[comic.comicName])
                }
            }
        }

        private fun handleEvent(event: P2pEvent) {
            val currentPeerId = peerId ?: return
            when (event) {
                is P2pEvent.LibraryBrowseResult ->
                    if (event.peerId == currentPeerId) {
                        _uiState.update { it.copy(comics = event.comics, loaded = true, errorMessage = null) }
                        fetchCoversFor(event.comics)
                    }
                is P2pEvent.LibraryBrowseError ->
                    if (event.peerId == currentPeerId) {
                        _uiState.update { it.copy(loaded = true, errorMessage = event.message) }
                    }

                is P2pEvent.CoverBrowseResult ->
                    if (event.peerId == currentPeerId) {
                        event.coverVersion?.let { knownCoverVersions[event.comicName] = it }
                        val path = event.path
                        if (event.status == "changed" && path != null) {
                            _uiState.update { it.copy(coverPaths = it.coverPaths + (event.comicName to path)) }
                        }
                    }
                is P2pEvent.CoverBrowseError -> Unit // best-effort — o item continua sem thumb

                is P2pEvent.FileSyncComplete ->
                    if (event.peerId == currentPeerId) _uiState.update { it.copy(syncingComicName = null) }
                is P2pEvent.FileSyncChapterFailed ->
                    // comicName/chapter vazios = falha da sessão inteira, não de um capítulo
                    // (ver protocol::files::run_and_report_scoped do lado Rust).
                    if (event.peerId == currentPeerId && event.comicName.isEmpty() && event.chapter.isEmpty()) {
                        _uiState.update { it.copy(syncingComicName = null) }
                    }

                else -> Unit
            }
        }

        companion object {
            private const val TAG = "RemoteLibraryViewModel"
        }
    }
