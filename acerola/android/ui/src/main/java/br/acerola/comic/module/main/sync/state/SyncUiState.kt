package br.acerola.comic.module.main.sync.state

import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.network.ComicSummary

data class SyncUiState(
    val localId: String = "",
    val localDeviceName: String = "",
    val pairingCode: String? = null,
    val mode: NetworkMode = NetworkMode.LOCAL,
    val relaySettings: RelaySettingsUiState = RelaySettingsUiState(),
    /** `true` logo após `SyncAction.SetIrohServicesTicket` falhar (formato inválido) — a
     *  própria [RelaySettingsUiState.hasIrohServicesTicket] não muda nesse caso. */
    val irohServicesTicketError: Boolean = false,
    val pairedPeers: List<PairedPeer> = emptyList(),
    val pendingConnect: PendingConnect? = null,
    val connecting: Boolean = false,
    val connectError: ConnectError? = null,
    val trustedPeerDialogPeerId: String? = null,
    val transferLog: List<TransferLogEntry> = emptyList(),
    /**
     * `kind` -> id da linha de [transferLog] atualmente em andamento pra esse protocolo
     * (spinner girando). Permite [TransferLogEntry] transicionar no lugar (started -> progress
     * -> complete/error) em vez de empilhar uma linha nova por evento, o que deixava a linha
     * "started" presa pra sempre como se ainda estivesse rodando.
     */
    val inFlightLogEntryByKind: Map<String, Long> = emptyMap(),
    /**
     * `"$peerId:$kind"` keys (kind = "history"/"files") with a sync session in flight —
     * disables that peer/protocol's buttons to avoid firing a second concurrent session for
     * the same (peer, ALPN) pair. Granularity is per kind, not just per peer: "Sync All"
     * fires history AND files for the same peer, and both need to run at once without
     * blocking each other.
     */
    val syncingKeys: Set<String> = emptySet(),
    /** `peerId` -> timestamp (epoch ms) of the last successfully completed session. */
    val lastSyncedByPeer: Map<String, Long> = emptyMap(),
    /** `peerId`s currently reachable in this session (subset of [pairedPeers]) — presence, not pairing. */
    val connectedPeerIds: Set<String> = emptySet(),
    /** `peerId` -> outcome of the most recent sync attempt (either kind), for inline feedback on [PeerRow]. */
    val lastSyncResultByPeer: Map<String, SyncResult> = emptyMap(),
    /** `peerId` of the peer whose [RemoteLibrarySheet] is currently open, or `null` if none.
     *  Also drives the loading state of that sheet while [remoteLibrary] hasn't arrived yet. */
    val browsingPeerId: String? = null,
    /** Result of the last `BrowseLibrary` request for [browsingPeerId] — cleared whenever a
     *  new browse starts or the sheet is dismissed. */
    val remoteLibrary: List<ComicSummary> = emptyList(),
    /** Distinguishes "still waiting for [P2pEvent.LibraryBrowseResult]" from "peer really has
     *  an empty library" — [remoteLibrary] alone can't tell those apart once it's empty. */
    val remoteLibraryLoaded: Boolean = false,
    /** Cru — usado só como fallback/argumento de template quando [browseLibraryErrorType] é
     *  `null` (ver `RemoteLibrarySheet`, mesmo padrão de `TransferLogEntry`/`SyncResult`). */
    val browseLibraryError: String? = null,
    /** Causa reconhecida (`SyncProtocolError`), quando houver. */
    val browseLibraryErrorType: SyncProtocolError? = null,
    /** `"$peerId:$comicName"` -> caminho local (`file://...`) da capa já baixada via
     *  `acerola/browse-cover/1`, pronto pro Coil carregar. Cache em memória — nunca rebaixa a
     *  mesma versão duas vezes dentro da mesma sessão do app (ver
     *  [br.acerola.comic.module.main.sync.SyncViewModel]). */
    val remoteCoverPaths: Map<String, String> = emptyMap(),
)

/**
 * Configuração de relay combinável exibida/editada no [br.acerola.comic.module.main.sync.RelaySettingsCard]
 * — espelha `RelayPreference.RelaySettings` (a fonte persistida), com nome próprio aqui pro
 * mesmo motivo de [PairedPeer]/[SyncResult]: `SyncUiState` não deve carregar tipos de outra
 * camada por conveniência, mesmo shape só reaproveitado de propósito.
 */
data class RelaySettingsUiState(
    val useAcerolaRelay: Boolean = true,
    val useIrohPublicNetwork: Boolean = false,
    val customRelayUrls: List<String> = emptyList(),
    /** Só indica SE um ticket da conta do usuário em `services.iroh.computer` já foi colado e
     *  salvo — o valor em si nunca é exposto na UI (é uma credencial real, guardada no cofre
     *  criptografado do node, não no DataStore junto das demais preferências de relay). */
    val hasIrohServicesTicket: Boolean = false,
)

data class SyncResult(
    val kind: String,
    val state: LogState,
    /** Cru/em inglês — nunca mostrado direto, só embutido num template pt-BR já traduzido
     *  (ver `SyncScreen::PeerRow`). Preferir [errorType] quando presente: é a causa reconhecida
     *  (`SyncProtocolError`), com uma mensagem própria e mais específica que o template
     *  genérico de fallback. */
    val message: String?,
    val timestamp: Long,
    val errorType: SyncProtocolError? = null,
)

data class PairedPeer(
    val peerId: String,
    val deviceName: String?,
)

data class PendingConnect(
    val peerId: String,
    val deviceId: String?,
    val addrs: ByteArray,
)

enum class ConnectError {
    INVALID_CODE,
    CONNECTION_FAILED,
}

enum class LogState {
    IN_PROGRESS,
    SUCCESS,
    ERROR,
}

/**
 * Raw data for one activity-log row — deliberately no pre-rendered display string here.
 * [br.acerola.comic.module.main.sync.SyncScreen] is the one that turns this into text via
 * `stringResource`, same as it already does for [ConnectError] in `LaunchedEffect`. Keeps
 * string-resource resolution (and thus any [android.content.Context] dependency) out of the
 * ViewModel entirely.
 */
data class TransferLogEntry(
    val id: Long,
    /** `"history"` or `"files"`. */
    val kind: String,
    /** `"started"`, `"progress"`, `"complete"`, `"error"`, or `"chapterFailed"` (files only). */
    val status: String,
    val state: LogState,
    val comicName: String? = null,
    val chapter: String? = null,
    val message: String? = null,
    val timestamp: Long = System.currentTimeMillis(),
)
