package br.acerola.comic.usecase.network

import br.acerola.comic.service.PeerAddress
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Before
import org.junit.Test

class SyncHistoryEntryWithPeerUseCaseTest {
    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private lateinit var useCase: SyncHistoryEntryWithPeerUseCase

    @Before
    fun setup() {
        useCase = SyncHistoryEntryWithPeerUseCase(p2pUseCase)
    }

    @Test
    fun `should resolve paired peer and fire syncHistoryEntry`() =
        runTest {
            val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())
            every { p2pUseCase.getPairedPeers() } returns listOf(peerAddress)
            coEvery { p2pUseCase.syncHistoryEntry(any(), any(), any()) } returns true

            val result = useCase("peer-1", "One Piece", listOf("1", "2"))

            assertThat(result).isEqualTo(SyncWithPeerResult.STARTED)
            coVerify { p2pUseCase.syncHistoryEntry(peerAddress, "One Piece", listOf("1", "2")) }
        }

    @Test
    fun `should not fire syncHistoryEntry when peer is not paired`() =
        runTest {
            every { p2pUseCase.getPairedPeers() } returns emptyList()

            val result = useCase("unknown-peer", "One Piece", listOf("1"))

            assertThat(result).isEqualTo(SyncWithPeerResult.NOT_PAIRED)
            coVerify(exactly = 0) { p2pUseCase.syncHistoryEntry(any(), any(), any()) }
        }

    @Test
    fun `should report declined when the mobile-data gate says no`() =
        runTest {
            val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())
            every { p2pUseCase.getPairedPeers() } returns listOf(peerAddress)
            coEvery { p2pUseCase.syncHistoryEntry(any(), any(), any()) } returns false

            val result = useCase("peer-1", "One Piece", listOf("1"))

            assertThat(result).isEqualTo(SyncWithPeerResult.DECLINED_MOBILE_DATA)
        }
}
