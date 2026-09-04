use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    core::services::sync::history_sync::HistorySyncService,
    data::{models::sync::SyncHistoryLogEntry, repositories::sync::SyncHistoryLogRepository},
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        protocol::transfer::{classify_sync_error, sync_error_payload},
    },
};

const LOG_KIND: &str = "history";

/// Lado que INICIA a sessão (papel "outbound": ativado quando este device chama
/// `connect()` no ALPN de histórico). Escreve o manifesto local primeiro, depois lê o do
/// peer — mesma disciplina de "quem escreve primeiro" que o handshake RPC da lib já usa,
/// pra nunca ter os dois lados bloqueados escrevendo ao mesmo tempo no mesmo stream.
pub struct HistorySyncOutbound {
    emit: EventEmitter,
    service: HistorySyncService,
    log_repo: SyncHistoryLogRepository,
}

impl HistorySyncOutbound {
    pub fn new(
        emit: EventEmitter, service: HistorySyncService, log_repo: SyncHistoryLogRepository,
    ) -> Self {
        Self { emit, service, log_repo }
    }

    async fn run(
        &self, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        let local_manifest = self.service.build_manifest().await?;
        write_json(writer, &local_manifest).await?;

        let peer_manifest = read_json(reader).await?;
        self.service.apply_manifest(&peer_manifest).await?;

        Ok(())
    }
}

#[async_trait]
impl Handler for HistorySyncOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)("sync:history:started", peer.id.clone());

        match self.run(&mut writer, &mut reader).await {
            Ok(()) => {
                (self.emit)("sync:history:complete", peer.id.clone());
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[HistorySync] session failed");
                (self.emit)(
                    "sync:history:error",
                    sync_error_payload(&peer.id, &message, code, None),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(error)
            },
        }
    }
}

/// Lado que RESPONDE à sessão (papel "inbound": ativado quando o peer remoto conecta
/// nesse ALPN). Lê o manifesto do peer primeiro, depois escreve o local.
pub struct HistorySyncInbound {
    emit: EventEmitter,
    service: HistorySyncService,
    log_repo: SyncHistoryLogRepository,
}

impl HistorySyncInbound {
    pub fn new(
        emit: EventEmitter, service: HistorySyncService, log_repo: SyncHistoryLogRepository,
    ) -> Self {
        Self { emit, service, log_repo }
    }

    async fn run(
        &self, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        let peer_manifest = read_json(reader).await?;

        let local_manifest = self.service.build_manifest().await?;
        write_json(writer, &local_manifest).await?;

        self.service.apply_manifest(&peer_manifest).await?;

        Ok(())
    }
}

#[async_trait]
impl Handler for HistorySyncInbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)("sync:history:started", peer.id.clone());

        match self.run(&mut writer, &mut reader).await {
            Ok(()) => {
                (self.emit)("sync:history:complete", peer.id.clone());
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[HistorySync] session failed");
                (self.emit)(
                    "sync:history:error",
                    sync_error_payload(&peer.id, &message, code, None),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(error)
            },
        }
    }
}
