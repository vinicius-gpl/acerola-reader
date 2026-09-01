package br.acerola.comic.module.main.sync

import android.content.Context
import android.os.Build
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.config.preference.DeviceAliasPreference
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.sync.state.ConnectError
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.module.main.sync.state.PendingConnect
import br.acerola.comic.module.main.sync.state.SyncAction
import br.acerola.comic.module.main.sync.state.SyncResult
import br.acerola.comic.module.main.sync.state.SyncUiState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.network.ComicSummary
import br.acerola.comic.service.network.P2pEvent
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import br.acerola.comic.util.p2p.PairingCode
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

private const val HANDSHAKE_ALPN = "acerola/handshake/1"
private const val HISTORY_SYNC_ALPN = "acerola/sync-history/1"
private const val FILE_SYNC_ALPN = "acerola/sync-files/1"
private const val CONNECT_TIMEOUT_MS = 15_000L
private const val MAX_LOG_ENTRIES = 50

// A real network may never deliver the completion/error event (peer vanished mid-session) —
// without this, that peer/protocol's buttons would stay disabled forever.
private const val SYNC_IN_FLIGHT_TIMEOUT_MS = 60_000L
internal const val SYNC_KIND_HISTORY = "history"
internal const val SYNC_KIND_FILES = "files"

/** Shared with [SyncScreen] to decide whether a sync button should be disabled. */
internal fun syncKey(
    peerId: String,
    kind: String,
) = "$peerId:$kind"

/** Shared with [RemoteLibrarySheet] to look up [SyncUiState.remoteCoverPaths] per item. */
internal fun coverKey(
    peerId: String,
    comicName: String,
) = "$peerId:$comicName"

@HiltViewModel
class SyncViewModel
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
        private val p2pEventBus: P2pEventBus,
        private val syncHistoryLogUseCase: SyncHistoryLogUseCase,
        @param:ApplicationContext private val context: Context,
    ) : ViewModel() {
        private val _uiState = MutableStateFlow(SyncUiState())
        val uiState: StateFlow<SyncUiState> = _uiState.asStateFlow()

        private val nextLogId = AtomicLong(0)

        /** `"$peerId:$comicName"` -> última versão de capa confirmada (cacheada ou já sabida
         *  como atual) — usado como `known_version` em `browseCover`, pra nunca rebaixar a mesma
         *  versão duas vezes na mesma sessão do app. Fora do `SyncUiState` de propósito: não
         *  precisa disparar recomposição sozinho, só [SyncUiState.remoteCoverPaths] importa pra UI. */
        private val knownCoverVersions = ConcurrentHashMap<String, Long>()

        init {
            viewModelScope.launch { refreshLocalInfo() }
            viewModelScope.launch { loadPersistedLog() }

            viewModelScope.launch {
                val override = RelayPreference.relayUrlOverrideFlow(context).first()
                _uiState.update {
                    it.copy(
                        relayUrl = override ?: RelayPreference.DEFAULT_RELAY_URL,
                        isRelayOverridden = override != null,
                    )
                }
            }

            // Apelido salvo já foi lido pra construir o node (`NetworkCaseModule`) — essa
            // segunda leitura é só pra refletir o mesmo valor aqui na UI, já que o node não
            // devolve o `DeviceInfo` que recebeu no boot. Depois disso, `localDeviceName` só
            // muda via `renameDevice` — nunca por `refreshLocalInfo`, que rodaria por cima do
            // que acabou de ser renomeado a cada evento de rede.
            viewModelScope.launch {
                val alias = DeviceAliasPreference.deviceAliasFlow(context).first()
                _uiState.update { it.copy(localDeviceName = alias ?: Build.MODEL) }
            }

            viewModelScope.launch {
                p2pEventBus.events.collect(::handleEvent)
            }
        }

        /**
         * Loads persisted sessions (most recent first) to give historical context as soon as
         * the screen opens — before any live event arrives. Negative IDs so they never collide
         * with the live IDs generated by [nextLogId] (which only grows from 0).
         */
        private suspend fun loadPersistedLog() {
            val rows = syncHistoryLogUseCase.findRecent(MAX_LOG_ENTRIES)

            val entries =
                rows.map { row ->
                    TransferLogEntry(
                        id = -row.id,
                        kind = row.kind,
                        status = row.status,
                        state = if (row.status == "complete") LogState.SUCCESS else LogState.ERROR,
                        message = row.message,
                        timestamp = row.createdAt,
                    )
                }

            val lastSynced =
                rows
                    .filter { it.status == "complete" }
                    .groupBy { it.peerId }
                    .mapValues { (_, entriesForPeer) -> entriesForPeer.maxOf { it.createdAt } }

            _uiState.update { it.copy(transferLog = entries, lastSyncedByPeer = lastSynced) }
        }

        /** Records a session's terminal result — updates "last synced" only on success, but
         *  [SyncUiState.lastSyncResultByPeer] always, so [SyncScreen] can surface a failure
         *  inline on the peer row without digging through the aggregate log. */
        private suspend fun recordSyncResult(
            peerId: String,
            kind: String,
            status: String,
            message: String?,
            errorType: SyncProtocolError? = null,
        ) {
            syncHistoryLogUseCase.record(peerId, kind, status, message)
            val now = System.currentTimeMillis()
            _uiState.update {
                it.copy(
                    lastSyncedByPeer = if (status == "complete") it.lastSyncedByPeer + (peerId to now) else it.lastSyncedByPeer,
                    lastSyncResultByPeer =
                        it.lastSyncResultByPeer +
                            (
                                peerId to
                                    SyncResult(
                                        kind = kind,
                                        state = if (status == "complete") LogState.SUCCESS else LogState.ERROR,
                                        message = message,
                                        timestamp = now,
                                        errorType = errorType,
                                    )
                            ),
                )
            }
        }

        fun onAction(action: SyncAction) {
            when (action) {
                is SyncAction.RenameDevice -> renameDevice(action.name)
                is SyncAction.ProposeConnect -> proposeConnect(action.code)
                SyncAction.ConfirmConnect -> confirmConnect()
                SyncAction.CancelConnect -> _uiState.update { it.copy(pendingConnect = null) }
                is SyncAction.SyncHistory -> triggerSync(action.peerId, HISTORY_SYNC_ALPN, SYNC_KIND_HISTORY)
                is SyncAction.SyncFiles -> triggerSync(action.peerId, FILE_SYNC_ALPN, SYNC_KIND_FILES)
                is SyncAction.SyncAll -> {
                    triggerSync(action.peerId, HISTORY_SYNC_ALPN, SYNC_KIND_HISTORY)
                    triggerSync(action.peerId, FILE_SYNC_ALPN, SYNC_KIND_FILES)
                }

                SyncAction.DismissTrustDialog -> _uiState.update { it.copy(trustedPeerDialogPeerId = null) }
                SyncAction.DismissConnectError -> _uiState.update { it.copy(connectError = null) }
                is SyncAction.RemovePeer -> removePeer(action.peerId)

                is SyncAction.BrowseLibrary -> browseLibrary(action.peerId)
                SyncAction.DismissLibraryBrowse ->
                    _uiState.update {
                        it.copy(
                            browsingPeerId = null,
                            remoteLibrary = emptyList(),
                            remoteLibraryLoaded = false,
                            browseLibraryError = null,
                        )
                    }
                is SyncAction.SyncComic -> syncComic(action.peerId, action.comicName)
            }
        }

        /** Dispara `acerola/browse-cover/1` em paralelo pra cada quadrinho da lista — streams são
         *  baratas numa conexão já pooled por `(peer, alpn)` (ver `acerola-p2p`), então não há
         *  necessidade de serializar. `known_version` vem do cache em memória
         *  ([knownCoverVersions]); ausente = primeira vez que essa capa é vista nesta sessão. */
        private fun fetchCoversFor(
            peerId: String,
            comics: List<ComicSummary>,
        ) {
            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId } ?: return@launch
                comics.forEach { comic ->
                    val key = coverKey(peerId, comic.comicName)
                    if (knownCoverVersions[key] == comic.coverVersion) return@forEach
                    p2pUseCase.browseCover(addr, comic.comicName, knownCoverVersions[key])
                }
            }
        }

        /** Só lista os quadrinhos do peer (nome + contagem) — não sincroniza nada. Resultado
         *  chega via [P2pEvent.LibraryBrowseResult]/[P2pEvent.LibraryBrowseError]. */
        private fun browseLibrary(peerId: String) {
            _uiState.update {
                it.copy(
                    browsingPeerId = peerId,
                    remoteLibrary = emptyList(),
                    remoteLibraryLoaded = false,
                    browseLibraryError = null,
                )
            }

            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(browsingPeerId = null) }
                    return@launch
                }
                p2pUseCase.browseLibrary(addr)
            }
        }

        /** Sincroniza um único quadrinho (`acerola/sync-comic/1`) — reaproveita a MESMA
         *  bookkeeping ([SYNC_KIND_FILES]/`syncingKeys`/[TransferLogEntry]) do botão
         *  "Sincronizar arquivos", já que ambos emitem os mesmos eventos `sync:files:*`
         *  (ver `protocol::files::run_and_report_scoped` do lado Rust). */
        private fun syncComic(
            peerId: String,
            comicName: String,
        ) {
            val key = syncKey(peerId, SYNC_KIND_FILES)
            if (key in _uiState.value.syncingKeys) return

            _uiState.update {
                it.copy(
                    syncingKeys = it.syncingKeys + key,
                    browsingPeerId = null,
                    remoteLibrary = emptyList(),
                    remoteLibraryLoaded = false,
                )
            }

            viewModelScope.launch {
                delay(SYNC_IN_FLIGHT_TIMEOUT_MS)
                _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
            }

            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
                    return@launch
                }
                p2pUseCase.syncComic(addr, comicName)
            }
        }

        /** Desempareia um peer — some da confiança e do cache nativo, e limpa todo estado
         *  local ligado a ele (senão o peer some da lista mas deixa lixo em
         *  [SyncUiState.lastSyncedByPeer]/[SyncUiState.lastSyncResultByPeer]/[SyncUiState.syncingKeys]). */
        private fun removePeer(peerId: String) {
            viewModelScope.launch(Dispatchers.IO) {
                p2pUseCase.removePairedPeer(peerId)
                _uiState.update {
                    it.copy(
                        pairedPeers = it.pairedPeers.filterNot { peer -> peer.peerId == peerId },
                        connectedPeerIds = it.connectedPeerIds - peerId,
                        lastSyncedByPeer = it.lastSyncedByPeer - peerId,
                        lastSyncResultByPeer = it.lastSyncResultByPeer - peerId,
                        syncingKeys = it.syncingKeys.filterNot { key -> key.startsWith("$peerId:") }.toSet(),
                    )
                }
            }
        }

        /** Sets a custom local device alias — applies on the P2P node right away (next
         *  handshake already uses it) and persists it via DataStore so it survives a restart.
         *  Updates [SyncUiState.localDeviceName] optimistically since the FFI call is
         *  fire-and-forget (no synchronous confirmation from the Rust side). */
        private fun renameDevice(name: String) {
            val trimmed = name.trim()
            if (trimmed.isEmpty()) return

            AcerolaLogger.i("SyncViewModel", "Renaming local device to: $trimmed", LogSource.UI)
            _uiState.update { it.copy(localDeviceName = trimmed) }

            viewModelScope.launch(Dispatchers.IO) {
                p2pUseCase.setLocalDeviceName(trimmed)
                DeviceAliasPreference.saveAlias(context, trimmed)
            }
        }

        /** Just decodes the code and proposes confirmation — no network activity yet. */
        private fun proposeConnect(code: String) {
            val decoded = PairingCode.decode(code)
            if (decoded == null) {
                _uiState.update { it.copy(connectError = ConnectError.INVALID_CODE) }
                return
            }

            _uiState.update {
                it.copy(
                    pendingConnect = PendingConnect(peerId = decoded.id, deviceId = decoded.deviceId, addrs = decoded.addrs),
                    connectError = null,
                )
            }
        }

        private fun confirmConnect() {
            val pending = _uiState.value.pendingConnect ?: return
            AcerolaLogger.i("SyncViewModel", "Pairing with peer: ${pending.peerId}", LogSource.UI)

            _uiState.update { it.copy(pendingConnect = null, connecting = true, connectError = null) }

            viewModelScope.launch {
                // Start listening for the handshake event BEFORE firing connect() — otherwise,
                // if the handshake finishes too fast, we risk missing the event (SharedFlow
                // doesn't replay) and wrongly concluding it failed.
                val handshakeCompleted =
                    async {
                        withTimeoutOrNull(CONNECT_TIMEOUT_MS) {
                            p2pEventBus.events.first {
                                it is P2pEvent.HandshakeCompleted && it.peerId == pending.peerId
                            }
                            true
                        }
                    }

                val peerAddress = PeerAddress(id = pending.peerId, deviceId = pending.deviceId, addrs = pending.addrs)
                withContext(Dispatchers.IO) {
                    p2pUseCase.connect(peerAddress, HANDSHAKE_ALPN.toByteArray())
                }

                // `p2pUseCase.connect` is fire-and-forget over the FFI (no synchronous
                // success/error return) — the handshake event is the most precise signal we
                // have today that "this actually worked". Without it within the timeout, we
                // treat it as a failure.
                val succeeded = handshakeCompleted.await() ?: false

                _uiState.update {
                    it.copy(
                        connecting = false,
                        connectError = if (succeeded) null else ConnectError.CONNECTION_FAILED,
                    )
                }
                refreshLocalInfo()
            }
        }

        private fun triggerSync(
            peerId: String,
            alpn: String,
            kind: String,
        ) {
            val key = syncKey(peerId, kind)
            if (key in _uiState.value.syncingKeys) return

            _uiState.update { it.copy(syncingKeys = it.syncingKeys + key) }

            viewModelScope.launch {
                delay(SYNC_IN_FLIGHT_TIMEOUT_MS)
                _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
            }

            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
                    return@launch
                }
                p2pUseCase.connect(addr, alpn.toByteArray())
            }
        }

        private fun clearSyncing(
            peerId: String,
            kind: String,
        ) {
            val key = syncKey(peerId, kind)
            _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
        }

        /**
         * [refreshLocalInfo] faz 5 chamadas síncronas bloqueantes pro Rust (`runtime.block_on`
         * do lado nativo — ver comentário lá) — disparar isso incondicionalmente pra TODO
         * evento (inclusive `network:latency`, que chega periodicamente e cai em
         * `P2pEvent.Unknown`) competia pelo mesmo runtime Tokio compartilhado que uma sessão de
         * sync-files ativa precisa pra progredir. Só chamamos explicitamente nos eventos que de
         * fato mudam o que essa função busca (peer pareado/conectado, sessão terminada).
         */
        private suspend fun handleEvent(event: P2pEvent) {
            when (event) {
                is P2pEvent.PeerTrustedFirstTime -> {
                    _uiState.update { it.copy(trustedPeerDialogPeerId = event.peerId) }
                    refreshLocalInfo()
                }
                is P2pEvent.HandshakeCompleted -> refreshLocalInfo()

                is P2pEvent.HistorySyncStarted ->
                    pushLog(SYNC_KIND_HISTORY, "started", LogState.IN_PROGRESS)
                is P2pEvent.HistorySyncComplete -> {
                    clearSyncing(event.peerId, SYNC_KIND_HISTORY)
                    recordSyncResult(event.peerId, SYNC_KIND_HISTORY, "complete", null)
                    pushLog(SYNC_KIND_HISTORY, "complete", LogState.SUCCESS)
                    refreshLocalInfo()
                }
                is P2pEvent.HistorySyncError -> {
                    clearSyncing(event.peerId, SYNC_KIND_HISTORY)
                    recordSyncResult(event.peerId, SYNC_KIND_HISTORY, "error", event.message)
                    pushLog(SYNC_KIND_HISTORY, "error", LogState.ERROR, message = event.message)
                    refreshLocalInfo()
                }

                is P2pEvent.FileSyncManifestExchanged ->
                    pushLog(SYNC_KIND_FILES, "started", LogState.IN_PROGRESS)
                is P2pEvent.FileSyncProgress -> {
                    pushLog(
                        SYNC_KIND_FILES,
                        "progress",
                        LogState.IN_PROGRESS,
                        comicName = event.comicName,
                        chapter = event.chapter,
                    )
                }
                is P2pEvent.FileSyncChapterFailed -> {
                    // Empty comicName/chapter is the sentinel for a whole-SESSION failure (see
                    // `protocol/files/mod.rs::run_and_report`), not a single chapter — only
                    // that case actually ends the session.
                    if (event.comicName.isEmpty() && event.chapter.isEmpty()) {
                        clearSyncing(event.peerId, SYNC_KIND_FILES)
                        // `event.error` já vem classificado como `SyncProtocolError` (ADT) por
                        // `P2pEventBus` — o ViewModel só propaga o valor tipado, sem `when` em
                        // cima de um `code` cru nem `Context` pra resolver texto nenhum.
                        // `event.reason` continua cru/em inglês, só pro histórico persistido
                        // (`syncHistoryLogUseCase.record`, dentro de `recordSyncResult`).
                        recordSyncResult(event.peerId, SYNC_KIND_FILES, "error", event.reason, event.error)
                        refreshLocalInfo()
                    }
                    pushLog(
                        SYNC_KIND_FILES,
                        "chapterFailed",
                        LogState.ERROR,
                        comicName = event.comicName,
                        chapter = event.chapter,
                    )
                }
                is P2pEvent.FileSyncComplete -> {
                    clearSyncing(event.peerId, SYNC_KIND_FILES)
                    recordSyncResult(event.peerId, SYNC_KIND_FILES, "complete", null)
                    pushLog(SYNC_KIND_FILES, "complete", LogState.SUCCESS)
                    refreshLocalInfo()
                }

                is P2pEvent.LibraryBrowseResult ->
                    if (event.peerId == _uiState.value.browsingPeerId) {
                        _uiState.update { it.copy(remoteLibrary = event.comics, remoteLibraryLoaded = true) }
                        fetchCoversFor(event.peerId, event.comics)
                    }
                is P2pEvent.LibraryBrowseError ->
                    if (event.peerId == _uiState.value.browsingPeerId) {
                        _uiState.update { it.copy(browseLibraryError = event.message) }
                    }

                is P2pEvent.CoverBrowseResult -> {
                    val key = coverKey(event.peerId, event.comicName)
                    event.coverVersion?.let { knownCoverVersions[key] = it }
                    val path = event.path
                    if (event.status == "changed" && path != null) {
                        _uiState.update { it.copy(remoteCoverPaths = it.remoteCoverPaths + (key to path)) }
                    }
                }
                is P2pEvent.CoverBrowseError -> Unit // best-effort — a lista continua sem thumb pra esse item

                else -> Unit
            }
        }

        private fun isSessionStatus(status: String) =
            status == "started" || status == "progress" || status == "complete" || status == "error"

        private fun isTerminalStatus(status: String) = status == "complete" || status == "error"

        private fun isOpenStatus(status: String) = status == "started" || status == "progress"

        private fun replaceInFlightEntry(
            log: List<TransferLogEntry>,
            index: Int,
            status: String,
            state: LogState,
            comicName: String?,
            chapter: String?,
            message: String?,
        ) = log.mapIndexed { position, entry ->
            if (position != index) {
                entry
            } else {
                entry.copy(
                    status = status,
                    state = state,
                    comicName = comicName ?: entry.comicName,
                    chapter = chapter ?: entry.chapter,
                    message = message,
                    timestamp = System.currentTimeMillis(),
                )
            }
        }

        /** Atualiza a linha em andamento pro `kind` no lugar, em vez de empilhar uma nova. */
        private fun updateInFlightLog(
            current: SyncUiState,
            kind: String,
            index: Int,
            status: String,
            state: LogState,
            comicName: String?,
            chapter: String?,
            message: String?,
        ): SyncUiState {
            val updatedLog = replaceInFlightEntry(current.transferLog, index, status, state, comicName, chapter, message)
            val inFlight =
                if (isTerminalStatus(status)) current.inFlightLogEntryByKind - kind else current.inFlightLogEntryByKind
            return current.copy(transferLog = updatedLog, inFlightLogEntryByKind = inFlight)
        }

        /** Empilha uma linha nova — só chega aqui quando não há sessão em andamento pra `kind`
         *  (primeiro evento) ou o status não participa da transição (`"chapterFailed"`). */
        private fun appendNewLog(
            current: SyncUiState,
            kind: String,
            status: String,
            state: LogState,
            comicName: String?,
            chapter: String?,
            message: String?,
        ): SyncUiState {
            val entry =
                TransferLogEntry(
                    id = nextLogId.getAndIncrement(),
                    kind = kind,
                    status = status,
                    state = state,
                    comicName = comicName,
                    chapter = chapter,
                    message = message,
                )
            val inFlight =
                if (isOpenStatus(status)) current.inFlightLogEntryByKind + (kind to entry.id) else current.inFlightLogEntryByKind
            return current.copy(
                transferLog = (listOf(entry) + current.transferLog).take(MAX_LOG_ENTRIES),
                inFlightLogEntryByKind = inFlight,
            )
        }

        /**
         * Uma sessão de sync tem UMA linha no log, que transiciona de estado (started ->
         * progress -> complete/error) em vez de empilhar uma linha nova por evento — sem isso,
         * "started" ficava pra sempre como uma linha separada com spinner girando ao lado da
         * linha "complete" que já resolveu a mesma sessão. `"chapterFailed"` (falha de UM
         * capítulo, não da sessão inteira — ver comentário em `handleEvent`) fica de fora dessa
         * transição de propósito: cada capítulo que falha é seu próprio evento permanente, sem
         * interromper a linha de progresso agregada da sessão.
         */
        private fun pushLog(
            kind: String,
            status: String,
            state: LogState,
            comicName: String? = null,
            chapter: String? = null,
            message: String? = null,
        ) {
            _uiState.update { current ->
                val trackedId = current.inFlightLogEntryByKind[kind].takeIf { isSessionStatus(status) }
                val index = trackedId?.let { id -> current.transferLog.indexOfFirst { it.id == id } }?.takeIf { it >= 0 }

                if (index != null) {
                    updateInFlightLog(current, kind, index, status, state, comicName, chapter, message)
                } else {
                    appendNewLog(current, kind, status, state, comicName, chapter, message)
                }
            }
        }

        /** Calls into the Rust node block the thread (JNI + `runtime.block_on` on that side). */
        private suspend fun refreshLocalInfo() =
            withContext(Dispatchers.IO) {
                val localAddress = p2pUseCase.getLocalAddress()
                val pairingCode = PairingCode.encode(localAddress.id, localAddress.deviceId, localAddress.addrs)
                // Nome do dispositivo vem de getPairedPeers() (que já resolve via
                // known_peers() do lado nativo, persistente) -- getConnectedPeersWithInfo()
                // só serve mais pra saber QUEM está conectado agora (connectedPeerIds), não
                // pra nome: aquela lista só tem dado durante os poucos segundos em que a
                // conexão de handshake está de fato aberta.
                val connectedPeers = p2pUseCase.getConnectedPeersWithInfo()
                val paired =
                    p2pUseCase.getPairedPeers().map {
                        PairedPeer(peerId = it.id, deviceName = it.deviceName)
                    }
                val localId = p2pUseCase.getLocalId()
                val mode = p2pUseCase.getMode()

                _uiState.update {
                    it.copy(
                        localId = localId,
                        pairingCode = pairingCode,
                        mode = mode,
                        pairedPeers = paired,
                        connectedPeerIds = connectedPeers.map { peer -> peer.peerId }.toSet(),
                    )
                }
            }
    }
