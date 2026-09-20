package br.acerola.comic.common.viewmodel.network

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.usecase.network.P2pUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class P2pViewModel
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
    ) : ViewModel() {
        fun getLocalId(): String {
            val id = p2pUseCase.getLocalId()
            AcerolaLogger.d("P2pViewModel", "getLocalId: $id", LogSource.UI)
            return id
        }

        fun getLocalAddress(): PeerAddress {
            val addr = p2pUseCase.getLocalAddress()
            AcerolaLogger.d("P2pViewModel", "getLocalAddress: ${addr.id}", LogSource.UI)
            return addr
        }

        fun connectToPeer(
            peerAddress: PeerAddress,
            alpn: ByteArray,
        ) {
            AcerolaLogger.i("P2pViewModel", "Connecting to peer: ${peerAddress.id}", LogSource.UI)
            viewModelScope.launch(Dispatchers.IO) { p2pUseCase.connect(peerAddress, alpn) }
        }

        fun switchToLocal() {
            AcerolaLogger.i("P2pViewModel", "Switching to LOCAL mode", LogSource.UI)
            p2pUseCase.switchToLocal()
        }

        fun switchToRelay() {
            AcerolaLogger.i("P2pViewModel", "Switching to RELAY mode", LogSource.UI)
            p2pUseCase.switchToRelay()
        }

        fun getMode(): NetworkMode = p2pUseCase.getMode()

        fun getConnectedPeers() = p2pUseCase.getConnectedPeers()

        override fun onCleared() {
            super.onCleared()
            p2pUseCase.close()
        }
    }
