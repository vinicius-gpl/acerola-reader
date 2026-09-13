package br.acerola.comic.module.main.sync

import android.content.Context
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.config.network.isOnCellularConnection
import br.acerola.comic.config.preference.DeviceAliasPreference
import br.acerola.comic.config.preference.MobileDataSyncPreference
import br.acerola.comic.config.preference.PeerNicknamePreference
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.module.main.sync.state.SyncAction
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.mockkStatic
import io.mockk.unmockkObject
import io.mockk.unmockkStatic
import io.mockk.verify
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
 * Cobre só o gate de "confirmar antes de sincronizar em dados móveis" (ver
 * `SyncViewModel.runSyncActionOrConfirm`/`MobileDataSyncPreference`) — o resto do
 * `SyncViewModel` (pareamento, relay, log de transferências) ainda não tem testes.
 *
 * Roda sob Robolectric (mesmo padrão de [br.acerola.comic.module.comic.ComicViewModelTest]) —
 * `refreshLocalInfo()` do `init` chama `PairingCode.encode`, que usa `android.util.Base64`
 * de verdade (stub puro de JVM devolve null e quebra com NPE).
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
        mockkObject(MobileDataSyncPreference)
        mockkStatic("br.acerola.comic.config.network.NetworkConnectivityKt")

        every { RelayPreference.relaySettingsFlow(any()) } returns flowOf(RelayPreference.RelaySettings())
        every { DeviceAliasPreference.deviceAliasFlow(any()) } returns flowOf(null)
        every { PeerNicknamePreference.nicknamesFlow(any()) } returns flowOf(emptyMap())
        every { MobileDataSyncPreference.alwaysAllowFlow(any()) } returns flowOf(false)
        coEvery { MobileDataSyncPreference.setAlwaysAllow(any(), any()) } returns Unit

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
        unmockkObject(RelayPreference, DeviceAliasPreference, PeerNicknamePreference, MobileDataSyncPreference)
        unmockkStatic("br.acerola.comic.config.network.NetworkConnectivityKt")
    }

    private fun createViewModel() = SyncViewModel(p2pUseCase, p2pEventBus, syncHistoryLogUseCase, context)

    @Test
    fun `dispara o sync na hora quando nao esta em dados moveis`() =
        runTest {
            every { isOnCellularConnection(any()) } returns false
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.SyncHistory(peerId = "peer-1"))

            assertThat(viewModel.uiState.value.pendingMobileDataSync).isNull()
            verify(timeout = 2000) { p2pUseCase.connect(match { it.id == "peer-1" }, any()) }
        }

    @Test
    fun `pede confirmacao e NAO dispara o sync quando esta em dados moveis`() =
        runTest {
            every { isOnCellularConnection(any()) } returns true
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.SyncHistory(peerId = "peer-1"))

            assertThat(viewModel.uiState.value.pendingMobileDataSync)
                .isEqualTo(SyncAction.SyncHistory(peerId = "peer-1"))
            verify(exactly = 0) { p2pUseCase.connect(any(), any()) }
        }

    @Test
    fun `cancelar a confirmacao limpa o estado pendente sem disparar o sync`() =
        runTest {
            every { isOnCellularConnection(any()) } returns true
            val viewModel = createViewModel()
            viewModel.onAction(SyncAction.SyncFiles(peerId = "peer-1"))

            viewModel.onAction(SyncAction.CancelMobileDataSync)

            assertThat(viewModel.uiState.value.pendingMobileDataSync).isNull()
            verify(exactly = 0) { p2pUseCase.connect(any(), any()) }
        }

    @Test
    fun `confirmar sem lembrar dispara o sync mas nao salva a preferencia`() =
        runTest {
            every { isOnCellularConnection(any()) } returns true
            val viewModel = createViewModel()
            viewModel.onAction(SyncAction.SyncFiles(peerId = "peer-1"))

            viewModel.onAction(SyncAction.ConfirmMobileDataSync(remember = false))

            assertThat(viewModel.uiState.value.pendingMobileDataSync).isNull()
            verify(timeout = 2000) { p2pUseCase.connect(match { it.id == "peer-1" }, any()) }
            coVerify(exactly = 0) { MobileDataSyncPreference.setAlwaysAllow(any(), any()) }
        }

    @Test
    fun `confirmar lembrando dispara o sync e salva a preferencia`() =
        runTest {
            every { isOnCellularConnection(any()) } returns true
            val viewModel = createViewModel()
            viewModel.onAction(SyncAction.SyncComic(peerId = "peer-1", comicName = "One Piece"))

            viewModel.onAction(SyncAction.ConfirmMobileDataSync(remember = true))

            assertThat(viewModel.uiState.value.pendingMobileDataSync).isNull()
            coVerify { MobileDataSyncPreference.setAlwaysAllow(context, true) }
            verify(timeout = 2000) {
                p2pUseCase.syncComic(match { it.id == "peer-1" }, "One Piece", SyncDirection.PULL)
            }
        }

    @Test
    fun `sync procede direto quando a preferencia ja esta ligada`() =
        runTest {
            every { isOnCellularConnection(any()) } returns true
            every { MobileDataSyncPreference.alwaysAllowFlow(any()) } returns flowOf(true)
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.SyncHistory(peerId = "peer-1"))

            assertThat(viewModel.uiState.value.pendingMobileDataSync).isNull()
            verify(timeout = 2000) { p2pUseCase.connect(match { it.id == "peer-1" }, any()) }
        }
}
