package br.acerola.comic.service.network

import android.content.Context
import br.acerola.comic.config.network.isOnCellularConnection
import br.acerola.comic.config.preference.MobileDataSyncPreference
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Portão único por onde QUALQUER chamada P2P que move dados de verdade precisa passar antes de
 * seguir em frente (ver [P2pUseCase.connect]/[P2pUseCase.syncComic]/[P2pUseCase.syncHistoryEntry],
 * os três pontos onde [ensureAllowed] é chamado). `@Singleton` — não atrelado a nenhuma tela — de
 * propósito: Home, Histórico, Quadrinho, Biblioteca remota e a tela de Sync disparam P2P por
 * caminhos completamente diferentes (ver os use cases em `br.acerola.comic.usecase.network`), e
 * nenhuma ViewModel de tela sozinha enxerga todos eles. Colocar a checagem dentro do próprio
 * [P2pUseCase] (em vez de em cada ViewModel/tela) garante que uma nova tela que passe a chamar
 * `connect`/`syncComic`/`syncHistoryEntry` amanhã já nasce coberta, sem precisar lembrar de nada.
 *
 * [awaitingConfirmation] é observado por um único diálogo montado na raiz do app
 * (`br.acerola.comic.common.activity.BaseActivity`, via `MobileDataSyncViewModel`) — a
 * confirmação aparece por cima de qualquer tela, não importa de onde a ação foi disparada.
 */
@Singleton
class MobileDataSyncGate
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
    ) {
        private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

        private val _awaitingConfirmation = MutableStateFlow(false)
        val awaitingConfirmation: StateFlow<Boolean> = _awaitingConfirmation.asStateFlow()

        /** Sem replay: só interessa a resposta a um pedido em aberto por vez. Se duas ações
         *  pedirem confirmação ao mesmo tempo (ex.: "Sincronizar tudo" dispara histórico e
         *  arquivos juntos), uma única resposta do usuário libera as duas — não tem por que
         *  empilhar um diálogo idêntico atrás do outro. */
        private val answers = MutableSharedFlow<Boolean>(extraBufferCapacity = 1)

        /** `true` libera na hora; `false` significa "usuário cancelou", e quem chamou deve
         *  desistir da ação (ver os `if (!ensureAllowed()) return` em [P2pUseCase]). */
        suspend fun ensureAllowed(): Boolean {
            if (MobileDataSyncPreference.alwaysAllowFlow(context).first() || !isOnCellularConnection(context)) {
                return true
            }

            _awaitingConfirmation.value = true
            val allowed = answers.first()
            _awaitingConfirmation.value = false
            return allowed
        }

        fun confirm(remember: Boolean) {
            if (remember) {
                scope.launch { MobileDataSyncPreference.setAlwaysAllow(context, true) }
            }
            answers.tryEmit(true)
        }

        fun cancel() {
            answers.tryEmit(false)
        }
    }
