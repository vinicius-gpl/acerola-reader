package br.acerola.comic.module.main.sync

import android.content.Context
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.config.preference.DeviceAliasPreference
import br.acerola.comic.config.preference.PeerNicknamePreference
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.module.main.sync.state.ConnectError
import br.acerola.comic.module.main.sync.state.PendingConnect
import br.acerola.comic.module.main.sync.state.SyncAction
import br.acerola.comic.module.main.sync.state.SyncUiState
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.unmockkObject
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/**
 * Cobre a reação do [SyncViewModel] a um `P2pUseCase.connect` recusado pelo
 * `MobileDataSyncGate` (usuário cancelou o diálogo global de dados móveis) — sem isso, o
 * spinner de "sincronizando"/"conectando" ficava preso esperando um evento que nunca chega
 * (60s de timeout pra `syncingKeys`, ou o `CONNECT_TIMEOUT_MS` de 15s pro pareamento, ambos
 * substituídos por uma reação imediata ao `false` retornado). O gate em si já é coberto por
 * `P2pUseCaseTest`/`MobileDataSyncGateTest` (módulo `core`) — aqui só a reação da UI.
 *
 * Roda sob Robolectric (mesmo motivo de [br.acerola.comic.module.comic.ComicViewModelTest]):
 * `refreshLocalInfo()` do `init` chama `PairingCode.encode`, que usa `android.util.Base64` de
 * verdade.
 */
@OptIn(ExperimentalCoroutinesApi::class)
@RunWith(RobolectricTestRunner::class)
class SyncViewModelTest {
    @get:Rule
    val coroutineRule = MainDispatcherRule()

    private val context = mockk<Context>(relaxed = true)
    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private val syncHistoryLogUseCase = mockk<SyncHistoryLogUseCase>(relaxed = true)
    private val p2pEventBus = P2pEventBus()

    private val pairedPeer = PeerAddress(id = "peer-1", deviceId = null, addrs = byteArrayOf(), deviceName = "Peer 1")

    @Before
    fun setup() {
        mockkObject(RelayPreference)
        mockkObject(DeviceAliasPreference)
        mockkObject(PeerNicknamePreference)

        every { RelayPreference.relaySettingsFlow(any()) } returns flowOf(RelayPreference.RelaySettings())
        every { DeviceAliasPreference.deviceAliasFlow(any()) } returns flowOf(null)
        every { PeerNicknamePreference.nicknamesFlow(any()) } returns flowOf(emptyMap())

        every { p2pUseCase.getLocalAddress() } returns pairedPeer.copy(id = "local-id", deviceName = null)
        every { p2pUseCase.getLocalId() } returns "local-id"
        every { p2pUseCase.getMode() } returns NetworkMode.LOCAL
        every { p2pUseCase.getPairedPeers() } returns listOf(pairedPeer)
        every { p2pUseCase.getConnectedPeersWithInfo() } returns emptyList()
        every { p2pUseCase.hasIrohServicesTicket() } returns false
        coEvery { syncHistoryLogUseCase.findRecent(any()) } returns emptyList()
    }

    @After
    fun tearDown() {
        unmockkObject(RelayPreference, DeviceAliasPreference, PeerNicknamePreference)
    }

    private fun createViewModel() = SyncViewModel(p2pUseCase, p2pEventBus, syncHistoryLogUseCase, context)

    /** `Thread.sleep` de verdade (não `delay()` — o trabalho que se está esperando roda em
     *  `Dispatchers.IO` real, uma thread pool própria, não afetada pelo tempo virtual do
     *  `TestDispatcher` deste teste) — só existe pra dar tempo do `viewModelScope.launch
     *  (Dispatchers.IO)` correspondente terminar antes da asserção. */
    private fun awaitState(
        viewModel: SyncViewModel,
        timeoutMs: Long = 2000,
        predicate: (SyncUiState) -> Boolean,
    ): SyncUiState {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            val state = viewModel.uiState.value
            if (predicate(state)) return state
            Thread.sleep(20)
        }
        return viewModel.uiState.value
    }

    @Test
    fun `triggerSync clears the spinner immediately when the mobile-data gate declines`() =
        runTest {
            coEvery { p2pUseCase.connect(any(), any()) } returns false
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.SyncHistory(peerId = "peer-1"))

            assertThat(viewModel.uiState.value.syncingKeys).contains("peer-1:history")

            val finalState = awaitState(viewModel) { it.syncingKeys.isEmpty() }
            assertThat(finalState.syncingKeys).isEmpty()
        }

    @Test
    fun `triggerSync keeps the spinner on when the gate allows`() =
        runTest {
            coEvery { p2pUseCase.connect(any(), any()) } returns true
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.SyncHistory(peerId = "peer-1"))

            val state = awaitState(viewModel) { "peer-1:history" in it.syncingKeys }
            assertThat(state.syncingKeys).contains("peer-1:history")
        }

    @Test
    fun `confirmConnect resets without an error when the mobile-data gate declines`() =
        runTest {
            coEvery { p2pUseCase.connect(any(), any()) } returns false
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.ProposeConnect(encodePairingCode("peer-2")))
            assertThat(viewModel.uiState.value.pendingConnect).isNotNull()

            viewModel.onAction(SyncAction.ConfirmConnect)

            val finalState = awaitState(viewModel) { !it.connecting }
            assertThat(finalState.connecting).isFalse()
            // Recusar dados móveis não é uma falha de conexão — mostrar `CONNECTION_FAILED`
            // aqui seria enganoso (o próprio diálogo global já comunicou a escolha).
            assertThat(finalState.connectError).isNull()
        }

    private fun encodePairingCode(peerId: String) =
        br.acerola.comic.util.p2p.PairingCode.encode(id = peerId, deviceId = null, addrs = byteArrayOf())
}
