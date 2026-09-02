package br.acerola.comic.usecase.network

import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.SyncDirection
import com.google.common.truth.Truth.assertThat
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
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
    fun `should resolve paired peer and fire syncComic`() {
        val peerAddress = PeerAddress(id = "peer-1", deviceId = "device-1", addrs = byteArrayOf())
        every { p2pUseCase.getPairedPeers() } returns listOf(peerAddress)

        val fired = useCase("peer-1", "One Piece", SyncDirection.PUSH)

        assertThat(fired).isTrue()
        verify { p2pUseCase.syncComic(peerAddress, "One Piece", SyncDirection.PUSH) }
    }

    @Test
    fun `should not fire syncComic when peer is not paired`() {
        every { p2pUseCase.getPairedPeers() } returns emptyList()

        val fired = useCase("unknown-peer", "One Piece", SyncDirection.PULL)

        assertThat(fired).isFalse()
        verify(exactly = 0) { p2pUseCase.syncComic(any(), any(), any()) }
    }
}
