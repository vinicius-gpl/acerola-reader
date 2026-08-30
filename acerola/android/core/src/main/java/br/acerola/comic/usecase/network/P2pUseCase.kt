package br.acerola.comic.usecase.network

import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.service.ConnectedPeerInfo
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.P2pService
import br.acerola.comic.service.PeerAddress
import java.io.Closeable
import javax.inject.Inject

class P2pUseCase
    @Inject
    constructor(
        private val p2pService: P2pService,
    ) : Closeable {
        fun getLocalId(): String {
            val id = p2pService.getLocalId()
            AcerolaLogger.d("P2pUseCase", "getLocalId: $id", LogSource.NETWORK)
            return id
        }

        fun getLocalAddress(): PeerAddress {
            val addr = p2pService.getLocalAddress()
            AcerolaLogger.d("P2pUseCase", "getLocalAddress: ${addr.id}", LogSource.NETWORK)
            return addr
        }

        fun setLocalDeviceName(name: String) {
            AcerolaLogger.i("P2pUseCase", "Renaming local device to: $name", LogSource.NETWORK)
            p2pService.setLocalDeviceName(name)
        }

        fun connect(
            peerAddress: PeerAddress,
            alpn: ByteArray,
        ) {
            AcerolaLogger.i("P2pUseCase", "Connecting to peer: ${peerAddress.id}", LogSource.NETWORK)
            p2pService.connect(peerAddress, alpn)
        }

        fun syncComic(
            peerAddress: PeerAddress,
            comicName: String,
        ) {
            AcerolaLogger.i("P2pUseCase", "Syncing comic '$comicName' with peer: ${peerAddress.id}", LogSource.NETWORK)
            p2pService.syncComic(peerAddress, comicName)
        }

        fun browseLibrary(peerAddress: PeerAddress) {
            AcerolaLogger.i("P2pUseCase", "Browsing library of peer: ${peerAddress.id}", LogSource.NETWORK)
            p2pService.browseLibrary(peerAddress)
        }

        fun browseCover(
            peerAddress: PeerAddress,
            comicName: String,
            knownVersion: Long?,
        ) {
            AcerolaLogger.i(
                "P2pUseCase",
                "Browsing cover of '$comicName' from peer: ${peerAddress.id}",
                LogSource.NETWORK,
            )
            p2pService.browseCover(peerAddress, comicName, knownVersion)
        }

        fun switchToLocal() {
            AcerolaLogger.i("P2pUseCase", "Switching to LOCAL mode", LogSource.NETWORK)
            p2pService.switchToLocal()
        }

        fun switchToRelay() {
            AcerolaLogger.i("P2pUseCase", "Switching to RELAY mode", LogSource.NETWORK)
            p2pService.switchToRelay()
        }

        fun getMode(): NetworkMode = p2pService.getMode()

        fun getConnectedPeers(): Map<String, List<ByteArray>> = p2pService.getConnectedPeers()

        fun getConnectedPeersWithInfo(): List<ConnectedPeerInfo> = p2pService.getConnectedPeersWithInfo()

        fun getPairedPeers(): List<PeerAddress> = p2pService.getPairedPeers()

        fun removePairedPeer(id: String) {
            AcerolaLogger.i("P2pUseCase", "Removing paired peer: $id", LogSource.NETWORK)
            p2pService.removePairedPeer(id)
        }

        fun shutdown() {
            p2pService.shutdown()
        }

        override fun close() {
            p2pService.close()
        }
    }
