mod exchange;
mod model;
mod registry;

use std::sync::Arc;

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use exchange::CoverOutcome;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{callbacks::CoverBrowseProvider, protocol::sync_error::classify_sync_error};
use super::files::ChapterTransfer;
pub(crate) use registry::PendingCoverRequestRegistry;

/// Busca a capa (thumbnail) de UM quadrinho remoto por vez, complementar ao `browse-library`.
/// Sem sessão/guard: round-trip curto (uma capa é pequena), não corre o mesmo risco de
/// I/O pesado duplicado que `sync-files`/`sync-comic` correm.
pub(crate) const COVER_BROWSE_ALPN: &[u8] = b"acerola/browse-cover/1";

pub(crate) struct CoverBrowseInbound {
    provider: Arc<dyn CoverBrowseProvider>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl CoverBrowseInbound {
    pub(crate) fn new(provider: Arc<dyn CoverBrowseProvider>, transfer: Arc<dyn ChapterTransfer>) -> Self {
        Self { provider, transfer }
    }
}

#[async_trait]
impl Handler for CoverBrowseInbound {
    async fn handle(
        &self, _peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = FramedWrite::new(send, LengthDelimitedCodec::new());
        let mut reader = FramedRead::new(recv, LengthDelimitedCodec::new());
        exchange::run_inbound(&self.provider, &self.transfer, &mut writer, &mut reader).await
    }
}

pub(crate) struct CoverBrowseOutbound {
    emit: EventEmitter,
    provider: Arc<dyn CoverBrowseProvider>,
    transfer: Arc<dyn ChapterTransfer>,
    pending_scope: Arc<PendingCoverRequestRegistry>,
}

impl CoverBrowseOutbound {
    pub(crate) fn new(
        emit: EventEmitter, provider: Arc<dyn CoverBrowseProvider>, transfer: Arc<dyn ChapterTransfer>,
        pending_scope: Arc<PendingCoverRequestRegistry>,
    ) -> Self {
        Self { emit, provider, transfer, pending_scope }
    }
}

#[async_trait]
impl Handler for CoverBrowseOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let Some((comic_name, known_version)) = self.pending_scope.take(&peer.id) else {
            let message = "no pending cover scope registered for this peer";
            (self.emit)(
                "browse:cover:error",
                serde_json::json!({ "peerId": peer.id, "message": message }).to_string(),
            );
            return Err(P2pError::StreamFailed(message.into()));
        };

        let mut writer = FramedWrite::new(send, LengthDelimitedCodec::new());
        let mut reader = FramedRead::new(recv, LengthDelimitedCodec::new());

        match exchange::run_outbound(
            comic_name.clone(),
            known_version,
            peer,
            &self.transfer,
            &mut writer,
            &mut reader,
        )
        .await
        {
            Ok(CoverOutcome::NotModified { cover_version }) => {
                (self.emit)(
                    "browse:cover:result",
                    serde_json::json!({
                        "peerId": peer.id,
                        "comicName": comic_name,
                        "status": "not_modified",
                        "coverVersion": cover_version,
                    })
                    .to_string(),
                );
                Ok(())
            },
            Ok(CoverOutcome::Unavailable) => {
                (self.emit)(
                    "browse:cover:result",
                    serde_json::json!({
                        "peerId": peer.id,
                        "comicName": comic_name,
                        "status": "unavailable",
                    })
                    .to_string(),
                );
                Ok(())
            },
            Ok(CoverOutcome::Fetched { cover_version, bytes }) => {
                let provider = Arc::clone(&self.provider);
                let peer_id = peer.id.clone();
                let comic_name_for_save = comic_name.clone();
                let path = crate::protocol::ffi_blocking::run_blocking(move || {
                    provider.save_remote_cover(peer_id, comic_name_for_save, cover_version, bytes)
                })
                .await?;

                (self.emit)(
                    "browse:cover:result",
                    serde_json::json!({
                        "peerId": peer.id,
                        "comicName": comic_name,
                        "status": "changed",
                        "coverVersion": cover_version,
                        "path": path,
                    })
                    .to_string(),
                );
                Ok(())
            },
            Err(err) => {
                let code = classify_sync_error(&err);
                tracing::warn!(peer = %peer.id, comic_name = %comic_name, ?code, error = %err, "[CoverBrowse] session failed");
                (self.emit)(
                    "browse:cover:error",
                    serde_json::json!({
                        "peerId": peer.id,
                        "comicName": comic_name,
                        "message": err.to_string(),
                        "code": code,
                    })
                    .to_string(),
                );
                Err(err)
            },
        }
    }
}
