package br.acerola.comic.common.viewmodel.network

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.config.preference.MobileDataSyncPreference
import br.acerola.comic.service.network.MobileDataSyncGate
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

/**
 * Ponte entre [MobileDataSyncGate] (singleton fora de qualquer ViewModel) e a UI Compose —
 * instanciada em dois lugares por motivos diferentes, sem problema já que ambos só espelham o
 * mesmo singleton/DataStore por baixo:
 * - `BaseActivity` (raiz do app): observa [awaitingConfirmation] pra mostrar o diálogo global,
 *   não importa qual tela esteja aberta.
 * - Tela de Config: lê/escreve [allowMobileDataSync] pro switch "Sincronizar usando dados
 *   móveis", que existe pra reverter a escolha "sempre permitir" feita uma vez no diálogo.
 */
@HiltViewModel
class MobileDataSyncViewModel
    @Inject
    constructor(
        private val mobileDataSyncGate: MobileDataSyncGate,
        @param:ApplicationContext private val context: Context,
    ) : ViewModel() {
        val awaitingConfirmation: StateFlow<Boolean> = mobileDataSyncGate.awaitingConfirmation

        val allowMobileDataSync: StateFlow<Boolean> =
            MobileDataSyncPreference
                .alwaysAllowFlow(context)
                .stateIn(viewModelScope, SharingStarted.Eagerly, false)

        fun confirm(remember: Boolean) = mobileDataSyncGate.confirm(remember)

        fun cancel() = mobileDataSyncGate.cancel()

        fun setAllowMobileDataSync(value: Boolean) {
            viewModelScope.launch { MobileDataSyncPreference.setAlwaysAllow(context, value) }
        }
    }
