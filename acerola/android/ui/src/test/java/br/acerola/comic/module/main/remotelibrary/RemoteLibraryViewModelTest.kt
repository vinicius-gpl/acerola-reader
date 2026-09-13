package br.acerola.comic.module.main.remotelibrary

import app.cash.turbine.test
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.error.UserMessage
import br.acerola.comic.module.main.remotelibrary.state.RemoteLibraryUiState
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

    /** `Thread.sleep` de verdade (não `delay()` — o trabalho que se está esperando roda em
     *  `Dispatchers.IO` real, thread pool própria, não afetada pelo tempo virtual do
     *  `TestDispatcher` deste teste). Necessário pro caminho `STARTED`: `syncComic` agenda um
     *  `viewModelScope.launch` (timeout de 60s) logo depois de marcar `syncingComicName` — sem
     *  esperar por esse `launch` já ter sido de fato disparado (não só a chamada mockada ter
     *  sido observada por `coVerify`), o teste pode terminar e o `@get:Rule` resetar o
     *  `Dispatchers.Main` ANTES dessa coroutine em `Dispatchers.IO` conseguir chegar lá — o que
     *  crasha com "Dispatchers.Main was accessed... test dispatcher was unset" numa thread sem
     *  handler, e esse erro não cai neste teste, cai no PRÓXIMO que rodar na mesma JVM (visto em
     *  CI: apareceu como falha em `ReaderViewModelTest`, sem relação nenhuma). */
    private fun awaitState(
        viewModel: RemoteLibraryViewModel,
        timeoutMs: Long = 2000,
        predicate: (RemoteLibraryUiState) -> Boolean,
    ): RemoteLibraryUiState {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            val state = viewModel.uiState.value
            if (predicate(state)) return state
            Thread.sleep(20)
        }
        return viewModel.uiState.value
    }

    @Test
    fun `syncComic marks syncing when the mobile-data gate allows`() =
        runTest {
            coEvery { syncComicWithPeerUseCase(any(), any(), any(), any()) } returns SyncWithPeerResult.STARTED

            viewModel.syncComic("One Piece")

            val state = awaitState(viewModel) { it.syncingComicName != null }
            assertThat(state.syncingComicName).isEqualTo("One Piece")
            coVerify { syncComicWithPeerUseCase("peer-1", "One Piece", SyncDirection.PULL) }
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
