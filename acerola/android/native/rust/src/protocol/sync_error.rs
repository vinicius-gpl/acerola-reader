use acerola_p2p::api::error::P2pError;

/// Identificador estável de causa de erro de sync, serializado em `snake_case` (`"busy"`,
/// `"timeout"`, `"connection_lost"`) — o lado Kotlin (`SyncProtocolError.fromCode`, `:infra`)
/// mapeia isso pra um `UiText.StringResource`, já que Android não tem troca de idioma (só
/// pt-BR). Compartilhado por todos os protocolos P2P (files/comic, history, cover-browse,
/// library-browse) — mesmo `code` em todos, mesmo padrão do desktop (`SyncErrorCode` em
/// `transfer.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncErrorCode {
    Busy,
    Timeout,
    ConnectionLost,
}

/// Classifica um `P2pError` (não seu texto) num `SyncErrorCode`. Só é confiável porque os
/// `ChapterTransfer`/blob-fetch de cada protocolo pararam de achatar `ConnectionError` em
/// `StreamFailed(err.to_string())` — o erro que chega aqui já é a variante de verdade que
/// `lib/p2p` classificou na origem (`classify_connection_error`/`classify_get_error`, crate
/// `acerola-p2p`), não texto pra adivinhar via substring matching. `Busy` é a exceção: não é
/// uma variante de `ConnectionError` (é um conceito de protocolo de app, não de transporte) —
/// cada `run_and_report*`/guard monta esse texto inline com o peer id interpolado, então o
/// match aqui é por substring numa frase que O PRÓPRIO MÓDULO controla, não uma heurística
/// sobre texto de terceiros.
pub(crate) fn classify_sync_error(error: &P2pError) -> Option<SyncErrorCode> {
    match error {
        P2pError::Timeout => Some(SyncErrorCode::Timeout),
        P2pError::PeerDisconnected(_) => Some(SyncErrorCode::ConnectionLost),
        P2pError::StreamFailed(msg) if msg.contains("already in progress") => {
            Some(SyncErrorCode::Busy)
        }
        _ => None,
    }
}
