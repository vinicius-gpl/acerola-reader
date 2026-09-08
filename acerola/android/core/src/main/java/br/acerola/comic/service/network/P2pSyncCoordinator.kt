package br.acerola.comic.service.network

import android.content.Context
import br.acerola.comic.core.R
import br.acerola.comic.service.P2pSyncForegroundService
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import br.acerola.comic.util.notification.NotificationHelper
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject
import javax.inject.Singleton

/** Espelham `SYNC_KIND_HISTORY`/`SYNC_KIND_FILES` de `SyncViewModel` (módulo `:ui`, não
 *  acessível daqui) — precisam continuar batendo com esses valores E com o `kind` gravado
 *  por `SyncHistoryLogUseCase`/lido de volta em `SyncViewModel.loadPersistedLog`. */
private const val SYNC_KIND_HISTORY = "history"
private const val SYNC_KIND_FILES = "files"

/**
 * Contraparte application-scoped do que `SyncViewModel` fazia sozinha antes: persiste o
 * resultado de uma sessão P2P ([SyncHistoryLogUseCase]), dispara a notificação de
 * conclusão/erro, e liga/desliga [P2pSyncForegroundService] — tudo isso independente de
 * qualquer tela estar aberta.
 *
 * Antes, essas três responsabilidades viviam dentro de `SyncViewModel` (`hiltViewModel()`
 * escopada à `NavBackStackEntry` da tela de Sync). Como [P2pEventBus] é um `SharedFlow` SEM
 * replay, um evento emitido sem nenhum coletor ativo se perde pra sempre — bastava o usuário
 * disparar um sync e apertar "voltar" antes dele terminar pra `SyncViewModel` ser destruída,
 * a inscrição no bus morrer junto, e o resultado (inclusive a notificação) nunca chegar a
 * lugar nenhum. O mesmo buraco existia pra sync disparado das telas de Histórico/
 * Quadrinho/Biblioteca remota — só `SyncViewModel` persistia/notificava, as outras só
 * atualizavam o próprio estado local de UI.
 *
 * Instanciado uma única vez (`@Singleton`, ver `NetworkCaseModule.provideP2pService`) assim
 * que o subsistema P2P existe — nunca atrelado ao ciclo de vida de nenhuma ViewModel.
 * `SyncViewModel` continua coletando o mesmo [P2pEventBus] pro estado de UI da PRÓPRIA tela
 * (log em memória, spinner dos botões); só a persistência/notificação/foreground service
 * saíram de lá pra cá.
 */
@Singleton
class P2pSyncCoordinator
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
        private val p2pEventBus: P2pEventBus,
        private val syncHistoryLogUseCase: SyncHistoryLogUseCase,
        private val notificationHelper: NotificationHelper,
    ) {
        private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

        /** `"$peerId:$kind"` de sessões em andamento — só controla o liga/desliga do
         *  foreground service, não precisa de granularidade nenhuma além disso. */
        private val activeSessions = MutableStateFlow<Set<String>>(emptySet())

        init {
            scope.launch { p2pEventBus.events.collect(::handleEvent) }

            // Ao contrário do que `SyncViewModel` fazia (ligado só aos syncs disparados PELA
            // tela de Sync), aqui reage a QUALQUER sessão do app inteiro — sync disparado da
            // Home/Histórico/Quadrinho/Biblioteca remota também passa a ganhar a isenção de
            // Doze/App Standby, não só o que passa pela tela de Sync.
            scope.launch {
                activeSessions
                    .map { it.isNotEmpty() }
                    .distinctUntilChanged()
                    .collect { active ->
                        if (active) P2pSyncForegroundService.start(context) else P2pSyncForegroundService.stop(context)
                    }
            }
        }

        private fun sessionKey(
            peerId: String,
            kind: String,
        ) = "$peerId:$kind"

        private fun startSession(
            peerId: String,
            kind: String,
        ) {
            activeSessions.update { it + sessionKey(peerId, kind) }
        }

        private fun endSession(
            peerId: String,
            kind: String,
        ) {
            activeSessions.update { it - sessionKey(peerId, kind) }
        }

        private suspend fun handleEvent(event: P2pEvent) {
            when (event) {
                is P2pEvent.HistorySyncStarted -> startSession(event.peerId, SYNC_KIND_HISTORY)
                is P2pEvent.HistorySyncComplete -> {
                    endSession(event.peerId, SYNC_KIND_HISTORY)
                    syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_HISTORY, "complete", null)
                    notifyHistorySyncComplete(event)
                }
                is P2pEvent.HistorySyncError -> {
                    endSession(event.peerId, SYNC_KIND_HISTORY)
                    syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_HISTORY, "error", event.message)
                    notifyHistorySyncFailed(event)
                }

                is P2pEvent.FileSyncManifestExchanged -> startSession(event.peerId, SYNC_KIND_FILES)
                is P2pEvent.FileSyncChapterFailed ->
                    // comicName/chapter vazios = falha da sessão INTEIRA, não de um capítulo
                    // (ver protocol/files/mod.rs::run_and_report do lado Rust) — só esse caso
                    // encerra a sessão.
                    if (event.comicName.isEmpty() && event.chapter.isEmpty()) {
                        endSession(event.peerId, SYNC_KIND_FILES)
                        syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_FILES, "error", event.reason)
                        notifyFileSyncFailed(event)
                    }
                is P2pEvent.FileSyncComplete -> {
                    endSession(event.peerId, SYNC_KIND_FILES)
                    syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_FILES, "complete", null)
                    notifyFileSyncComplete(event)
                }

                else -> Unit
            }
        }

        private fun notifyFileSyncComplete(event: P2pEvent.FileSyncComplete) {
            val content =
                if (event.failedCount > 0) {
                    context.getString(
                        R.string.notification_sync_files_complete_content_with_failures,
                        event.receivedCount,
                        event.sentCount,
                        event.failedCount,
                    )
                } else {
                    context.getString(
                        R.string.notification_sync_files_complete_content,
                        event.receivedCount,
                        event.sentCount,
                    )
                }
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_files_complete_title),
                content = content,
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }

        private fun notifyFileSyncFailed(event: P2pEvent.FileSyncChapterFailed) {
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_files_error_title),
                content = event.error?.uiMessage?.asString(context) ?: event.reason,
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }

        private fun notifyHistorySyncComplete(event: P2pEvent.HistorySyncComplete) {
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_history_complete_title),
                content =
                    context.getString(
                        R.string.notification_sync_history_complete_content,
                        event.progressApplied,
                        event.chaptersReadApplied,
                    ),
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }

        private fun notifyHistorySyncFailed(event: P2pEvent.HistorySyncError) {
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_history_error_title),
                content = event.error?.uiMessage?.asString(context) ?: event.message,
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }
    }
