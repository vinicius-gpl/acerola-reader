package br.acerola.comic.usecase.network

import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.SyncDirection
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Before
import org.junit.Test

class SyncComicWithPeerUseCaseTest {
    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private lateinit var useCase: SyncComicWithPeerUseCase

    @Before
    fun setup() {
        useCase = SyncComicWithPeerUseCase(p2pUseCase)
    }

    @Test
    fun `should resolve paired peer and fire syncComic`() =
        runTest {
            val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())
            every { p2pUseCase.getPairedPeers() } returns listOf(peerAddress)
            coEvery { p2pUseCase.syncComic(any(), any(), any(), any()) } returns true

            val result = useCase("peer-1", "One Piece", SyncDirection.PUSH)

            assertThat(result).isEqualTo(SyncWithPeerResult.STARTED)
            coVerify { p2pUseCase.syncComic(peerAddress, "One Piece", SyncDirection.PUSH) }
        }

    @Test
    fun `should not fire syncComic when peer is not paired`() =
        runTest {
            every { p2pUseCase.getPairedPeers() } returns emptyList()

            val result = useCase("unknown-peer", "One Piece", SyncDirection.PULL)

            assertThat(result).isEqualTo(SyncWithPeerResult.NOT_PAIRED)
            coVerify(exactly = 0) { p2pUseCase.syncComic(any(), any(), any()) }
        }

    @Test
    fun `should report declined when the mobile-data gate says no`() =
        runTest {
            val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())
            every { p2pUseCase.getPairedPeers() } returns listOf(peerAddress)
            coEvery { p2pUseCase.syncComic(any(), any(), any(), any()) } returns false

            val result = useCase("peer-1", "One Piece", SyncDirection.PUSH)

            assertThat(result).isEqualTo(SyncWithPeerResult.DECLINED_MOBILE_DATA)
        }

    @Test
    fun `should forward chapters to scope the session to specific chapters`() =
        runTest {
            val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())
            every { p2pUseCase.getPairedPeers() } returns listOf(peerAddress)
            coEvery { p2pUseCase.syncComic(any(), any(), any(), any()) } returns true

            val result = useCase("peer-1", "One Piece", SyncDirection.PUSH, listOf("Cap 1", "Cap 2"))

            assertThat(result).isEqualTo(SyncWithPeerResult.STARTED)
            coVerify { p2pUseCase.syncComic(peerAddress, "One Piece", SyncDirection.PUSH, listOf("Cap 1", "Cap 2")) }
        }
}
