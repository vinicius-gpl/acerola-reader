package br.acerola.comic.module.main.remotelibrary

import app.cash.turbine.test
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.error.UserMessage
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncComicWithPeerUseCase
import br.acerola.comic.usecase.network.SyncWithPeerResult
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runTest
import org.junit.Before
import org.junit.Rule
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class RemoteLibraryViewModelTest {
    @get:Rule
    val coroutineRule = MainDispatcherRule()

    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private val p2pEventBus = P2pEventBus()
    private val syncComicWithPeerUseCase = mockk<SyncComicWithPeerUseCase>(relaxed = true)

    private lateinit var viewModel: RemoteLibraryViewModel

    @Before
    fun setup() {
        val pairedPeer = PeerAddress(id = "peer-1", deviceId = null, addrs = byteArrayOf())
        every { p2pUseCase.getPairedPeers() } returns listOf(pairedPeer)

        viewModel = RemoteLibraryViewModel(p2pUseCase, p2pEventBus, syncComicWithPeerUseCase)
        // `browseLibrary` disparado pelo `init` já resolve o peer (stub acima) — sem erro de
        // "não pareado" competindo com os eventos que os testes abaixo esperam.
        viewModel.init(peerId = "peer-1", peerDisplayName = "Peer 1")
    }

    @Test
    fun `syncComic marks syncing when the mobile-data gate allows`() =
        runTest {
            coEvery { syncComicWithPeerUseCase(any(), any(), any(), any()) } returns SyncWithPeerResult.STARTED

            viewModel.syncComic("One Piece")

            coVerify(timeout = 2000) { syncComicWithPeerUseCase("peer-1", "One Piece", SyncDirection.PULL) }
            assertThat(viewModel.uiState.value.syncingComicName).isEqualTo("One Piece")
        }

    @Test
    fun `syncComic sends an error and does not mark syncing when the peer is not paired`() =
        runTest {
            coEvery { syncComicWithPeerUseCase(any(), any(), any(), any()) } returns SyncWithPeerResult.NOT_PAIRED

            viewModel.uiEvents.test {
                viewModel.syncComic("One Piece")
                assertThat(awaitItem()).isInstanceOf(UserMessage.Raw::class.java)
            }
            assertThat(viewModel.uiState.value.syncingComicName).isNull()
        }

    @Test
    fun `syncComic does nothing when the mobile-data gate declines`() =
        runTest {
            coEvery { syncComicWithPeerUseCase(any(), any(), any(), any()) } returns SyncWithPeerResult.DECLINED_MOBILE_DATA

            viewModel.syncComic("One Piece")

            // Confirma que a coroutine em `Dispatchers.IO` já rodou até o fim antes de checar
            // "nada mais aconteceu" — sem isso as asserções abaixo poderiam passar só por ainda
            // não ter dado tempo.
            coVerify(timeout = 2000) { syncComicWithPeerUseCase("peer-1", "One Piece", SyncDirection.PULL) }

            assertThat(viewModel.uiState.value.syncingComicName).isNull()
            viewModel.uiEvents.test {
                expectNoEvents()
            }
        }
}
