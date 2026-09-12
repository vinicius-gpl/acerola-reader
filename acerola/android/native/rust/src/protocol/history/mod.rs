mod exchange;
mod model;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{callbacks::HistorySyncProvider, protocol::sync_error::classify_sync_error};

pub(crate) const HISTORY_SYNC_ALPN: &[u8] = b"acerola/sync-history/1";
/// Push individual de UMA entrada de histórico, sem trocar o manifesto da biblioteca inteira —
/// ver `exchange::run_entry_exchange`. Complementar a `HISTORY_SYNC_ALPN`, não substitui.
pub(crate) const HISTORY_ENTRY_SYNC_ALPN: &[u8] = b"acerola/sync-history-entry/1";

/// `peer_id` -> `(comic_name, chapter_sorts)` pendente entre a chamada FFI `sync_history_entry`
/// e `HistoryEntrySyncOutbound::handle` — mesma técnica de `protocol::files::PendingComicScope`.
pub(crate) type PendingHistoryEntryScope = Arc<Mutex<HashMap<String, (String, Vec<String>)>>>;

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

/// Papel outbound do protocolo `acerola/sync-history-entry/1` — `comic_name` vem de
/// `PendingHistoryEntryScope`, gravado pela chamada FFI `sync_history_entry` antes de
/// `connect()` (mesma técnica de `ComicSyncOutbound`/`PendingComicScope`).
pub(crate) struct HistoryEntrySyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
    pending_scope: PendingHistoryEntryScope,
}

impl HistoryEntrySyncOutbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn HistorySyncProvider>,
        pending_scope: PendingHistoryEntryScope,
    ) -> Self {
        Self {
            emit,
            provider,
            pending_scope,
        }
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let scope = self
            .pending_scope
            .lock()
            .expect("pending history entry scope mutex poisoned")
            .remove(&peer.id);

        exchange::run_entry_exchange(true, scope, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history-entry:error", error_payload(peer, err)))
    }
}

/// Papel inbound do protocolo `acerola/sync-history-entry/1` — `comic_name` já vem dentro da
/// própria entrada recebida, então não precisa de nenhum escopo prévio.
pub(crate) struct HistoryEntrySyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
}

impl HistoryEntrySyncInbound {
    pub(crate) fn new(emit: EventEmitter, provider: Arc<dyn HistorySyncProvider>) -> Self {
        Self { emit, provider }
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_entry_exchange(false, None, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history-entry:error", error_payload(peer, err)))
    }
}
