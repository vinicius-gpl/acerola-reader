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

use crate::{callbacks::HistorySyncProvider, protocol::sync_error::classify_sync_error};

pub(crate) const HISTORY_SYNC_ALPN: &[u8] = b"acerola/sync-history/1";

/// Papel outbound do protocolo `acerola/sync-history/1` — usado quando este lado é quem
/// iniciou a conexão via `AcerolaP2p::connect`. Escreve seu manifesto primeiro (regra 4).
pub(crate) struct HistorySyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
}

impl HistorySyncOutbound {
    pub(crate) fn new(emit: EventEmitter, provider: Arc<dyn HistorySyncProvider>) -> Self {
        Self { emit, provider }
    }
}

#[async_trait]
impl Handler for HistorySyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_exchange(true, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history:error", error_payload(peer, err)))
    }
}

/// Papel inbound do protocolo `acerola/sync-history/1` — usado quando o peer é quem iniciou
/// a conexão. Lê o manifesto do peer primeiro (regra 4).
pub(crate) struct HistorySyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
}

impl HistorySyncInbound {
    pub(crate) fn new(emit: EventEmitter, provider: Arc<dyn HistorySyncProvider>) -> Self {
        Self { emit, provider }
    }
}

#[async_trait]
impl Handler for HistorySyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_exchange(false, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history:error", error_payload(peer, err)))
    }
}

fn error_payload(peer: &PeerIdentity, err: &P2pError) -> String {
    let code = classify_sync_error(err);
    tracing::warn!(peer = %peer.id, ?code, error = %err, "[HistorySync] session failed");
    serde_json::json!({ "peerId": peer.id, "message": err.to_string(), "code": code }).to_string()
}
