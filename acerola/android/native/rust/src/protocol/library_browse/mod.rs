mod exchange;
mod model;

use std::sync::Arc;

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{callbacks::FileSyncProvider, protocol::sync_error::classify_sync_error};
use model::LibrarySummary;

/// Lista só os nomes dos quadrinhos de um peer (com contagem de capítulos) — sem transferir
/// nada. Sem sessão/guard: é uma única leitura curta, não corre risco de duplicar trabalho de
/// I/O pesado do jeito que `acerola/sync-files/1`/`acerola/sync-comic/1` correm (ver
/// `protocol::files::FileSyncSessionGuard`).
pub(crate) const LIBRARY_BROWSE_ALPN: &[u8] = b"acerola/browse-library/1";

/// Papel inbound: responde com o resumo da biblioteca local assim que a conexão é aceita — não
/// espera nem lê o marcador que o outbound escreve (ver doc em `exchange::run_inbound`), pra não
/// acoplar a resposta a quanto tempo o path P2P leva pra ficar pronto pra dados de verdade.
pub(crate) struct LibraryBrowseInbound {
    provider: Arc<dyn FileSyncProvider>,
}

impl LibraryBrowseInbound {
    pub(crate) fn new(provider: Arc<dyn FileSyncProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Handler for LibraryBrowseInbound {
    async fn handle(
        &self,
        _peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        _recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_inbound(&self.provider, send).await
    }
}

/// Papel outbound: escreve o marcador de pedido, lê a resposta e emite o resultado como evento
/// pro Kotlin — quem disparou `P2PNode::browse_library` não tem retorno síncrono (mesmo padrão
/// fire-and-forget de `connect`), então o resultado só chega via
/// `browse:library:result`/`browse:library:error`.
pub(crate) struct LibraryBrowseOutbound {
    emit: EventEmitter,
}

impl LibraryBrowseOutbound {
    pub(crate) fn new(emit: EventEmitter) -> Self {
        Self { emit }
    }
}

#[async_trait]
impl Handler for LibraryBrowseOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        match exchange::run_outbound(send, recv).await {
            Ok(summary) => {
                (self.emit)("browse:library:result", result_payload(peer, &summary));
                Ok(())
            }
            Err(err) => {
                let code = classify_sync_error(&err);
                tracing::warn!(peer = %peer.id, ?code, error = %err, "[LibraryBrowse] query failed");
                (self.emit)(
                    "browse:library:error",
                    serde_json::json!({ "peerId": peer.id, "message": err.to_string(), "code": code }).to_string(),
                );
                Err(err)
            }
        }
    }
}

fn result_payload(peer: &PeerIdentity, summary: &LibrarySummary) -> String {
    let comics: Vec<serde_json::Value> = summary
        .comics
        .iter()
        .map(|entry| {
            serde_json::json!({
                "comicName": entry.comic_name,
                "chapterCount": entry.chapter_count,
                "coverVersion": entry.cover_version,
            })
        })
        .collect();

    serde_json::json!({ "peerId": peer.id, "comics": comics }).to_string()
}
