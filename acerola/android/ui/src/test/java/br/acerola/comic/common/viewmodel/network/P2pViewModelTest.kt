package br.acerola.comic.common.viewmodel.network

import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.usecase.network.P2pUseCase
import com.google.common.truth.Truth.assertThat
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runTest
import org.junit.Before
import org.junit.Rule
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class P2pViewModelTest {
    @get:Rule
    val coroutineRule = MainDispatcherRule()

    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private lateinit var viewModel: P2pViewModel

    @Before
    fun setup() {
        viewModel = P2pViewModel(p2pUseCase)
    }

    @Test
    fun `should return local id from use case`() {
        every { p2pUseCase.getLocalId() } returns "my-id"
        assertThat(viewModel.getLocalId()).isEqualTo("my-id")
    }

    @Test
    fun `should delegate connection to use case`() =
        runTest {
            val peerAddress =
                PeerAddress(
                    id = "peer-1",
                    deviceId = "device-1",
                    addrs = byteArrayOf(),
                )
            val alpn = "test".toByteArray()
            viewModel.connectToPeer(peerAddress, alpn)
            coVerify(timeout = 2000) { p2pUseCase.connect(peerAddress, alpn) }
        }

    @Test
    fun `should switch to local mode`() {
        viewModel.switchToLocal()
        verify { p2pUseCase.switchToLocal() }
    }

    @Test
    fun `should switch to relay mode`() {
        viewModel.switchToRelay()
        verify { p2pUseCase.switchToRelay() }
    }

    @Test
    fun `should return current mode`() {
        every { p2pUseCase.getMode() } returns NetworkMode.LOCAL
        assertThat(viewModel.getMode()).isEqualTo(NetworkMode.LOCAL)
    }
}
