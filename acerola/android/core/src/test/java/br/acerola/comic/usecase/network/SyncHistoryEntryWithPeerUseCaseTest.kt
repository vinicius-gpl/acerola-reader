package br.acerola.comic.usecase.network

import br.acerola.comic.service.PeerAddress
import com.google.common.truth.Truth.assertThat
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
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
    fun `should resolve paired peer and fire syncHistoryEntry`() {
        val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())
        every { p2pUseCase.getPairedPeers() } returns listOf(peerAddress)

        val fired = useCase("peer-1", "One Piece", listOf("1", "2"))

        assertThat(fired).isTrue()
        verify { p2pUseCase.syncHistoryEntry(peerAddress, "One Piece", listOf("1", "2")) }
    }

    @Test
    fun `should not fire syncHistoryEntry when peer is not paired`() {
        every { p2pUseCase.getPairedPeers() } returns emptyList()

        val fired = useCase("unknown-peer", "One Piece", listOf("1"))

        assertThat(fired).isFalse()
        verify(exactly = 0) { p2pUseCase.syncHistoryEntry(any(), any(), any()) }
    }
}
