package br.acerola.comic.usecase.network

import br.acerola.comic.service.P2pService
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.service.network.MobileDataSyncGate
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Before
import org.junit.Test

/**
 * Cobre só o gate de dados móveis (ver `MobileDataSyncGate`) — [P2pUseCase.connect]/
 * [P2pUseCase.syncComic]/[P2pUseCase.syncHistoryEntry] são os três pontos por onde QUALQUER
 * tela (Home/Histórico/Quadrinho/Biblioteca remota/Sync) acaba passando pra mover dados de
 * verdade, então a checagem tem que estar aqui, não em cada ViewModel de tela.
 */
class P2pUseCaseTest {
    private val p2pService = mockk<P2pService>(relaxed = true)
    private val mobileDataSyncGate = mockk<MobileDataSyncGate>()
    private lateinit var p2pUseCase: P2pUseCase

    private val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())

    @Before
    fun setup() {
        p2pUseCase = P2pUseCase(p2pService, mobileDataSyncGate)
    }

    @Test
    fun `connect proceeds and returns true when the gate allows`() =
        runTest {
            coEvery { mobileDataSyncGate.ensureAllowed() } returns true

            val started = p2pUseCase.connect(peerAddress, "alpn".toByteArray())

            assertThat(started).isTrue()
            coVerify { p2pService.connect(peerAddress, "alpn".toByteArray()) }
        }

    @Test
    fun `connect does nothing and returns false when the gate declines`() =
        runTest {
            coEvery { mobileDataSyncGate.ensureAllowed() } returns false

            val started = p2pUseCase.connect(peerAddress, "alpn".toByteArray())

            assertThat(started).isFalse()
            coVerify(exactly = 0) { p2pService.connect(any(), any()) }
        }

    @Test
    fun `syncComic proceeds and returns true when the gate allows`() =
        runTest {
            coEvery { mobileDataSyncGate.ensureAllowed() } returns true

            val started = p2pUseCase.syncComic(peerAddress, "One Piece", SyncDirection.PUSH)

            assertThat(started).isTrue()
            coVerify { p2pService.syncComic(peerAddress, "One Piece", SyncDirection.PUSH, emptyList()) }
        }

    @Test
    fun `syncComic does nothing and returns false when the gate declines`() =
        runTest {
            coEvery { mobileDataSyncGate.ensureAllowed() } returns false

            val started = p2pUseCase.syncComic(peerAddress, "One Piece", SyncDirection.PUSH)

            assertThat(started).isFalse()
            coVerify(exactly = 0) { p2pService.syncComic(any(), any(), any(), any()) }
        }

    @Test
    fun `syncHistoryEntry proceeds and returns true when the gate allows`() =
        runTest {
            coEvery { mobileDataSyncGate.ensureAllowed() } returns true

            val started = p2pUseCase.syncHistoryEntry(peerAddress, "One Piece", listOf("1"))

            assertThat(started).isTrue()
            coVerify { p2pService.syncHistoryEntry(peerAddress, "One Piece", listOf("1")) }
        }

    @Test
    fun `syncHistoryEntry does nothing and returns false when the gate declines`() =
        runTest {
            coEvery { mobileDataSyncGate.ensureAllowed() } returns false

            val started = p2pUseCase.syncHistoryEntry(peerAddress, "One Piece", listOf("1"))

            assertThat(started).isFalse()
            coVerify(exactly = 0) { p2pService.syncHistoryEntry(any(), any(), any()) }
        }
}
