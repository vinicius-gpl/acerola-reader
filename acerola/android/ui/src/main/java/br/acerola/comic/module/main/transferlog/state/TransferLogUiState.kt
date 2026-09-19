package br.acerola.comic.module.main.transferlog.state

import br.acerola.comic.module.main.sync.state.TransferLogEntry

data class TransferLogUiState(
    val entries: List<TransferLogEntry> = emptyList(),
    val loaded: Boolean = false,
)
