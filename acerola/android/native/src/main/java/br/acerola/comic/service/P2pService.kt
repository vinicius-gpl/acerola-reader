package br.acerola.comic.service

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import android.util.Log
import p2p.CoverBrowseProvider
import p2p.FfiNetworkMode
import p2p.FfiPeerAddr
import p2p.FileSyncProvider
import p2p.HistorySyncProvider
import p2p.P2pCallback
import p2p.P2pNode
import p2p.SecureBlobStore
import java.io.Closeable

enum class NetworkMode {
    LOCAL,
    RELAY,
}

data class PeerAddress(
    val id: String,
    val deviceId: String?,
    val addrs: ByteArray,
    /** Só preenchido pelo retorno de [getPairedPeers] — `null` nos demais usos (endereço só
     *  pra discar, ex: [connect]/[syncComic]). */
    val deviceName: String? = null,
)

data class ConnectedPeerInfo(
    val peerId: String,
    val alpns: List<ByteArray>,
    val deviceName: String?,
)

class P2pService(
    context: Context,
    relayUrlOverride: String?,
    /** Apelido custom salvo pelo usuário (DataStore, `DeviceAliasPreference`) — `null` quando
     *  nunca definido, e o `Build.MODEL` padrão é usado. Só lido aqui, na construção do node;
     *  renomear depois de iniciado é feito em runtime via [setLocalDeviceName], sem
     *  reconstruir nada. */
    deviceNameOverride: String?,
    secureStore: SecureBlobStore,
    historyProvider: HistorySyncProvider,
    fileProvider: FileSyncProvider,
    coverProvider: CoverBrowseProvider,
    private val eventListener: (event: String, data: String) -> Unit,
) : Closeable {
    private val callbackHandler =
        object : P2pCallback {
            override fun onEvent(
                event: String,
                data: String,
            ) {
                Log.d("P2pService", "Event received: $event, Data: $data")
                eventListener(event, data)
            }
        }

    // `P2pNode(...)` chama `P2PNode::new()` no Rust, que faz `runtime.block_on(...)` — uma
    // chamada síncrona bloqueante (ver TODO.md, item crítico de ANR). Antes, isso rodava direto
    // no `init{}`, então bloqueava QUALQUER thread que instanciasse `P2pService` — hoje em dia,
    // a thread principal, porque o Hilt resolve esse singleton pela primeira vez ao construir
    // `HomeViewModel`/`P2pUseCase` na primeira tela. Trocado por `lazy` (thread-safe por
    // padrão) disparado numa thread dedicada logo abaixo: o construtor de `P2pService` agora
    // retorna na hora, e a construção pesada roda em background. Quem chamar um método deste
    // serviço antes dessa construção terminar ainda bloqueia (o `Lazy` sincroniza via lock), mas
    // isso deixa de ser garantido a cada vez — só quando o uso realmente corre com o warm-up.
    private val p2pNode: P2pNode by lazy(LazyThreadSafetyMode.SYNCHRONIZED) {
        // Only used by Rust to migrate identity/peers/trust from the old plain-text format
        // into `SecureBlobStore` once — see `storage.rs`/`trust_store.rs`.
        val legacyDataDir = context.filesDir.resolve("p2p").absolutePath
        // Store local de blobs content-addressed (`iroh-blobs-adapter`) — cache separado da
        // árvore de biblioteca do usuário (SAF), reclamado por GC interno da lib, não pelo app.
        val blobsDir = context.filesDir.resolve("blobs").absolutePath
        val appVersion =
            runCatching {
                context.packageManager.getPackageInfo(context.packageName, 0).versionName
            }.getOrNull() ?: "unknown"
        P2pNode(
            callbackHandler,
            legacyDataDir,
            blobsDir,
            relayUrlOverride,
            deviceNameOverride ?: Build.MODEL,
            appVersion,
            secureStore,
            historyProvider,
            fileProvider,
            coverProvider,
        )
    }

    // Iroh (o transporte QUIC por baixo do `p2pNode`) detecta mudança de rede sozinho na
    // maioria dos sistemas, mas não no Android: o SO não expõe monitoramento de interface pra
    // código nativo, só pra Java/Kotlin via `ConnectivityManager`. Sem repassar esse evento, uma
    // troca de Wi-Fi/dados móveis (ou saída de Doze) deixa o transporte reaproveitando conexões
    // já mortas, e todo browse/sync pro mesmo peer falha com timeout até o app ser reiniciado.
    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    private val networkCallback =
        object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                Log.d("P2pService", "Network available, notifying P2P transport")
                p2pNode.notifyNetworkChange()
            }

            override fun onLost(network: Network) {
                Log.d("P2pService", "Network lost, notifying P2P transport")
                p2pNode.notifyNetworkChange()
            }

            override fun onCapabilitiesChanged(
                network: Network,
                networkCapabilities: NetworkCapabilities,
            ) {
                p2pNode.notifyNetworkChange()
            }
        }

    init {
        Thread({ p2pNode }, "P2pNode-warmup").apply { isDaemon = true }.start()
        connectivityManager.registerDefaultNetworkCallback(networkCallback)
    }

    fun getLocalId(): String = p2pNode.getLocalId()

    /** Apelido custom estilo LocalSend (em vez do `Build.MODEL` padrão) — aplica na hora no
     *  node P2P (próximo handshake já usa o nome novo, sem precisar reiniciar o app). */
    fun setLocalDeviceName(name: String) {
        Log.d("P2pService", "Renaming local device to: $name")
        p2pNode.setLocalDeviceName(name)
    }

    fun getLocalAddress(): PeerAddress {
        val addr = p2pNode.getLocalAddr()
        return PeerAddress(
            id = addr.id,
            deviceId = addr.deviceId,
            addrs = addr.addrs,
        )
    }

    fun connect(
        peerAddress: PeerAddress,
        alpn: ByteArray,
    ) {
        Log.d("P2pService", "Connecting to peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.connect(ffiAddr, alpn)
    }

    /** Sincroniza um único quadrinho com o peer — cobre tanto push (quadrinho que já tenho)
     *  quanto pull (quadrinho descoberto navegando a biblioteca remota), já que a troca do
     *  protocolo `acerola/sync-comic/1` é sempre simétrica. */
    fun syncComic(
        peerAddress: PeerAddress,
        comicName: String,
    ) {
        Log.d("P2pService", "Syncing comic '$comicName' with peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.syncComic(ffiAddr, comicName)
    }

    /** Pede a lista de quadrinhos (nome + contagem de capítulos) da biblioteca do peer, sem
     *  sincronizar nada — resultado chega via evento `browse:library:result`/`error`. */
    fun browseLibrary(peerAddress: PeerAddress) {
        Log.d("P2pService", "Browsing library of peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.browseLibrary(ffiAddr)
    }

    /** Busca a capa de UM quadrinho remoto — `knownVersion` é a versão já cacheada localmente
     *  (`null` se nunca buscou essa capa antes). Resultado chega via evento
     *  `browse:cover:result`/`browse:cover:error`, com o caminho local salvo no payload
     *  (`path`) quando `status == "changed"`. */
    fun browseCover(
        peerAddress: PeerAddress,
        comicName: String,
        knownVersion: Long?,
    ) {
        Log.d("P2pService", "Browsing cover of '$comicName' from peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.browseCover(ffiAddr, comicName, knownVersion)
    }

    fun switchToLocal() {
        p2pNode.switchToLocal()
    }

    fun switchToRelay() {
        p2pNode.switchToRelay()
    }

    fun getMode(): NetworkMode =
        when (p2pNode.getMode()) {
            FfiNetworkMode.LOCAL -> NetworkMode.LOCAL
            FfiNetworkMode.RELAY -> NetworkMode.RELAY
        }

    fun getConnectedPeers(): Map<String, List<ByteArray>> = p2pNode.getConnectedPeers()

    fun getConnectedPeersWithInfo(): List<ConnectedPeerInfo> =
        p2pNode.getConnectedPeersWithInfo().map {
            ConnectedPeerInfo(peerId = it.peerId, alpns = it.alpns, deviceName = it.deviceName)
        }

    /**
     * Every peer ever paired (TOFU), with its last known address — persists across app
     * restarts. The handshake connection itself only lasts a few seconds (PING/PONG/DeviceInfo
     * exchange, then closes), so this — not [getConnectedPeers] — is the right source for
     * "paired devices" in the UI.
     */
    fun getPairedPeers(): List<PeerAddress> =
        p2pNode.getPairedPeers().map {
            PeerAddress(id = it.id, deviceId = it.deviceId, addrs = it.addrs, deviceName = it.deviceName)
        }

    /** Desempareia um peer — some da confiança e do cache de endereços conhecidos. Uma
     *  reconexão futura desse peer passa pelo mesmo fluxo TOFU de um dispositivo novo. */
    fun removePairedPeer(id: String) {
        Log.d("P2pService", "Removing paired peer: $id")
        p2pNode.removePairedPeer(id)
    }

    fun shutdown() {
        runCatching { connectivityManager.unregisterNetworkCallback(networkCallback) }
        p2pNode.destroy()
    }

    override fun close() {
        shutdown()
    }
}
