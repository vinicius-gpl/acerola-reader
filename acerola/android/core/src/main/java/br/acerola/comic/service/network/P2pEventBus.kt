package br.acerola.comic.service.network

import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import org.json.JSONObject
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Canal de eventos do node P2P, ouvido por qualquer tela interessada.
 *
 * Ser um singleton application-scoped (não pertencer a uma ViewModel) é necessário, mas NÃO
 * suficiente pra evitar perder eventos enquanto nenhuma tela está aberta: `_events` tem
 * `replay = 0`, então um valor emitido sem nenhum coletor inscrito é descartado pra sempre —
 * `extraBufferCapacity` só amortece rajadas pra coletores JÁ inscritos, não alimenta
 * retroativamente um que se inscreve depois. Quem de fato garante que "sync complete"/"peer
 * trusted for the first time" sobrevivem a nenhuma tela de sync estar aberta é o
 * [P2pSyncCoordinator], um coletor sempre ativo, instanciado uma única vez a partir de
 * [br.acerola.comic.usecase.NetworkCaseModule] — nunca atrelado ao ciclo de vida de nenhuma
 * ViewModel. ViewModels escopadas a tela também podem coletar [events] pro próprio estado
 * local de UI, mas não podem ser a ÚNICA ouvinte de nada que precise sobreviver à navegação.
 */
@Singleton
class P2pEventBus
    @Inject
    constructor() {
        private val _events = MutableSharedFlow<P2pEvent>(extraBufferCapacity = 64)
        val events: SharedFlow<P2pEvent> = _events.asSharedFlow()

        fun emit(
            event: String,
            data: String,
        ) {
            val parsed = parse(event, data)
            AcerolaLogger.d("P2pEventBus", "emit: $event -> $parsed", LogSource.NETWORK)
            _events.tryEmit(parsed)
        }

        private fun parse(
            event: String,
            data: String,
        ): P2pEvent =
            runCatching {
                when (event) {
                    "peer:trusted_first_time" -> P2pEvent.PeerTrustedFirstTime(peerId = data)

                    // Emitted by the library's own handshake (RpcClientHandler/RpcServerHandler)
                    // as soon as the DeviceInfo exchange finishes — `data` is the raw peer id,
                    // not JSON.
                    "rpc:device_info_received", "rpc:device_info_exchanged" ->
                        P2pEvent.HandshakeCompleted(peerId = data)

                    "sync:history:started" ->
                        JSONObject(data).let {
                            P2pEvent.HistorySyncStarted(peerId = it.getString("peerId"))
                        }

                    "sync:history:complete" ->
                        JSONObject(data).let {
                            P2pEvent.HistorySyncComplete(
                                peerId = it.getString("peerId"),
                                progressApplied = it.getInt("progressApplied"),
                                progressSkipped = it.getInt("progressSkipped"),
                                chaptersReadApplied = it.getInt("chaptersReadApplied"),
                                chaptersReadSkipped = it.getInt("chaptersReadSkipped"),
                            )
                        }

                    "sync:history:error" ->
                        JSONObject(data).let {
                            P2pEvent.HistorySyncError(
                                peerId = it.getString("peerId"),
                                message = it.getString("message"),
                                error = SyncProtocolError.fromCode(if (it.isNull("code")) null else it.getString("code")),
                            )
                        }

                    "sync:files:manifest_exchanged" ->
                        JSONObject(data).let {
                            P2pEvent.FileSyncManifestExchanged(
                                peerId = it.getString("peerId"),
                                missingCount = it.getInt("missingCount"),
                                offeringCount = it.getInt("offeringCount"),
                            )
                        }

                    "sync:files:progress" ->
                        JSONObject(data).let {
                            P2pEvent.FileSyncProgress(
                                peerId = it.getString("peerId"),
                                comicName = it.getString("comicName"),
                                chapter = it.getString("chapter"),
                                bytesTransferred = it.getLong("bytesTransferred"),
                                totalBytes = it.getLong("totalBytes"),
                            )
                        }

                    "sync:files:chapter_complete" ->
                        JSONObject(data).let {
                            P2pEvent.FileSyncChapterComplete(
                                peerId = it.getString("peerId"),
                                comicName = it.getString("comicName"),
                                chapter = it.getString("chapter"),
                            )
                        }

                    "sync:files:chapter_failed" ->
                        JSONObject(data).let {
                            val code = if (it.isNull("code")) null else it.getString("code")
                            P2pEvent.FileSyncChapterFailed(
                                peerId = it.getString("peerId"),
                                comicName = it.getString("comicName"),
                                chapter = it.getString("chapter"),
                                reason = it.getString("reason"),
                                error = SyncProtocolError.fromCode(code),
                            )
                        }

                    "sync:files:extra_complete" ->
                        JSONObject(data).let {
                            P2pEvent.FileSyncExtraComplete(
                                peerId = it.getString("peerId"),
                                comicName = it.getString("comicName"),
                                kind = it.getString("kind"),
                            )
                        }

                    "sync:files:extra_failed" ->
                        JSONObject(data).let {
                            val code = if (it.isNull("code")) null else it.getString("code")
                            P2pEvent.FileSyncExtraFailed(
                                peerId = it.getString("peerId"),
                                comicName = it.getString("comicName"),
                                kind = it.getString("kind"),
                                reason = it.getString("reason"),
                                error = SyncProtocolError.fromCode(code),
                            )
                        }

                    "sync:files:complete" ->
                        JSONObject(data).let {
                            P2pEvent.FileSyncComplete(
                                peerId = it.getString("peerId"),
                                receivedCount = it.getInt("receivedCount"),
                                sentCount = it.getInt("sentCount"),
                                failedCount = it.getInt("failedCount"),
                            )
                        }

                    "browse:library:result" ->
                        JSONObject(data).let { json ->
                            val comicsArray = json.getJSONArray("comics")
                            val comics =
                                (0 until comicsArray.length()).map { index ->
                                    val entry = comicsArray.getJSONObject(index)
                                    ComicSummary(
                                        comicName = entry.getString("comicName"),
                                        chapterCount = entry.getInt("chapterCount"),
                                        coverVersion = entry.optLong("coverVersion", 0L),
                                    )
                                }
                            P2pEvent.LibraryBrowseResult(peerId = json.getString("peerId"), comics = comics)
                        }

                    "browse:library:error" ->
                        JSONObject(data).let {
                            P2pEvent.LibraryBrowseError(
                                peerId = it.getString("peerId"),
                                message = it.getString("message"),
                                error = SyncProtocolError.fromCode(if (it.isNull("code")) null else it.getString("code")),
                            )
                        }

                    "browse:cover:result" ->
                        JSONObject(data).let {
                            P2pEvent.CoverBrowseResult(
                                peerId = it.getString("peerId"),
                                comicName = it.getString("comicName"),
                                status = it.getString("status"),
                                coverVersion = if (it.has("coverVersion")) it.getLong("coverVersion") else null,
                                path = if (it.has("path")) it.getString("path") else null,
                            )
                        }

                    "browse:cover:error" ->
                        JSONObject(data).let {
                            P2pEvent.CoverBrowseError(
                                peerId = it.getString("peerId"),
                                comicName = it.optString("comicName", ""),
                                message = it.getString("message"),
                            )
                        }

                    else -> P2pEvent.Unknown(event, data)
                }
            }.getOrElse { error ->
                AcerolaLogger.e("P2pEventBus", "Failed to parse event '$event': $error", LogSource.NETWORK)
                P2pEvent.Unknown(event, data)
            }
    }
