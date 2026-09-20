package br.acerola.comic.module.main.transferlog

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.module.main.transferlog.state.TransferLogUiState
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val MAX_ENTRIES = 50

/**
 * ViewModel dedicado da tela cheia de histórico ([TransferLogActivity]) — de propósito leve,
 * só depende de [SyncHistoryLogUseCase]. Diferente de
 * [br.acerola.comic.module.main.sync.SyncViewModel] (que sobe peers, relay e o módulo P2P
 * inteiro), essa tela só lista sessões já persistidas (`complete`/`error`); sessões ao vivo
 * continuam visíveis via os spinners por peer na aba de Rede
 * ([br.acerola.comic.module.main.sync.PeerRow]).
 */
@HiltViewModel
class TransferLogViewModel
    @Inject
    constructor(
        private val syncHistoryLogUseCase: SyncHistoryLogUseCase,
    ) : ViewModel() {
        private val _uiState = MutableStateFlow(TransferLogUiState())
        val uiState: StateFlow<TransferLogUiState> = _uiState.asStateFlow()

        init {
            refresh()
        }

        fun refresh() {
            viewModelScope.launch(Dispatchers.IO) {
                val rows = syncHistoryLogUseCase.findRecent(MAX_ENTRIES)
                val entries =
                    rows.map { row ->
                        TransferLogEntry(
                            id = row.id,
                            kind = row.kind,
                            status = row.status,
                            state = if (row.status == "complete") LogState.SUCCESS else LogState.ERROR,
                            message = row.message,
                            timestamp = row.createdAt,
                        )
                    }
                _uiState.update { it.copy(entries = entries, loaded = true) }
            }
        }

        fun clear() {
            viewModelScope.launch(Dispatchers.IO) {
                syncHistoryLogUseCase.clearAll()
                _uiState.update { it.copy(entries = emptyList()) }
            }
        }
    }
