package br.acerola.comic.error.message

import br.acerola.comic.error.UserMessage
import br.acerola.comic.infra.R
import br.acerola.comic.type.UiText

/**
 * Erros do protocolo de sync P2P (`acerola/sync-files/1`, `acerola/sync-comic/1`) que têm uma
 * causa reconhecida — classificados na origem, a partir do `code` estável que o Rust anexa ao
 * evento (`classify_sync_error`, `protocol/files/mod.rs`), nunca por `when`/substring matching
 * em cima do texto técnico (esse continua cru, em inglês, só pra log/fallback — ver
 * `P2pEvent.FileSyncChapterFailed.reason`).
 */
sealed interface SyncProtocolError : UserMessage {
    data object SessionBusy : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_session_busy)
    }

    data object Timeout : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_timeout)
    }

    data object ConnectionLost : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_connection_lost)
    }

    /** Diferente das outras três (causas de transporte/protocolo, de `classify_sync_error`),
     *  este `code` vem de `chapter_failed_payload`/`extra_failed_payload`
     *  (`protocol/files/exchange.rs`) — anexado quando `write_chapter_chunk`/fetch do blob NÃO
     *  falharam, mas o Kotlin (`finalizeChapterWrite`/`finalizeExtraWrite`) mesmo assim recusou
     *  o arquivo por checksum. */
    data object ChecksumMismatch : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_checksum_mismatch)
    }

    companion object {
        /** Único lugar que interpreta o `code` cru do wire (`"busy"`/`"timeout"`/
         *  `"connection_lost"`/`"checksum_mismatch"`) — `null` (causa não reconhecida) sinaliza
         *  pra quem chama usar o `reason` cru como fallback, em vez de um `SyncProtocolError`. */
        fun fromCode(code: String?): SyncProtocolError? =
            when (code) {
                "busy" -> SessionBusy
                "timeout" -> Timeout
                "connection_lost" -> ConnectionLost
                "checksum_mismatch" -> ChecksumMismatch
                else -> null
            }
    }
}
