use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use super::history_entry_registry::PendingHistoryEntryRegistry;
use crate::{
    core::services::sync::history_sync::HistorySyncService,
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        messages::HistoryManifest,
        protocol::transfer::{classify_sync_error, sync_error_payload},
    },
};

const LOG_KIND: &str = "history-entry";

/// Push UNIDIRECIONAL de um `HistoryManifest` restrito ao(s) capítulo(s) selecionado(s) de UM
/// quadrinho — existe pra não precisar de uma sessão `acerola/sync-history/1` completa (que
/// troca a biblioteca inteira nos dois sentidos) só pra levar o progresso/marcador de "lido" de
/// capítulo(s) que o usuário escolheu enviar. O escopo (quadrinho + `chapter_sort`s) vem de
/// `PendingHistoryEntryRegistry` (mesma técnica de `ComicSyncOutbound`/
/// `PendingComicSyncRegistry`), gravado pelo comando Tauri `sync_history_entry` antes de chamar
/// `connect()`.
pub struct HistoryEntrySyncOutbound {
    emit: EventEmitter,
    service: HistorySyncService,
    registry: std::sync::Arc<PendingHistoryEntryRegistry>,
}

impl HistoryEntrySyncOutbound {
    pub fn new(
        emit: EventEmitter, service: HistorySyncService,
        registry: std::sync::Arc<PendingHistoryEntryRegistry>,
    ) -> Self {
        Self { emit, service, registry }
    }

    async fn run(
        &self, peer_id: &str, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        let scope = self.registry.take(peer_id).ok_or_else(|| {
            P2pError::StreamFailed("no pending history entry sync scope for this peer".into())
        })?;

        let manifest =
            self.service.build_manifest_for_chapters(&scope.comic_name, &scope.chapter_ids).await?;
        write_json(writer, &manifest).await?;

        // Espera o ack antes de considerar a sessão concluída — sem isso, o lado outbound
        // poderia fechar a conexão antes do inbound terminar de ler/aplicar a entrada.
        let _ack: serde_json::Value = read_json(reader).await?;

        Ok(())
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)("sync:history-entry:started", peer.id.clone());

        match self.run(&peer.id, &mut writer, &mut reader).await {
            Ok(()) => {
                (self.emit)("sync:history-entry:complete", peer.id.clone());
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[HistoryEntrySync] session failed");
                (self.emit)(
                    "sync:history-entry:error",
                    sync_error_payload(&peer.id, &message, code, None),
                );
                Err(error)
            },
        }
    }
}

/// Lado que RESPONDE ao push — aplica o manifesto recebido localmente e confirma com um ack
/// vazio. O manifesto já vem restrito ao(s) capítulo(s) selecionado(s) pelo lado outbound, então
/// não precisa de nenhum pedido/escopo prévio como `ComicSyncRequest`.
pub struct HistoryEntrySyncInbound {
    emit: EventEmitter,
    service: HistorySyncService,
}

impl HistoryEntrySyncInbound {
    pub fn new(emit: EventEmitter, service: HistorySyncService) -> Self {
        Self { emit, service }
    }

    async fn run(
        &self, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        let manifest: HistoryManifest = read_json(reader).await?;
        self.service.apply_manifest(&manifest).await?;

        write_json(writer, &serde_json::json!({})).await?;

        Ok(())
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncInbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)("sync:history-entry:started", peer.id.clone());

        match self.run(&mut writer, &mut reader).await {
            Ok(()) => {
                (self.emit)("sync:history-entry:complete", peer.id.clone());
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[HistoryEntrySync] session failed");
                (self.emit)(
                    "sync:history-entry:error",
                    sync_error_payload(&peer.id, &message, code, None),
                );
                Err(error)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use acerola_p2p::api::peer::PeerIdentity;

    use super::super::history_entry_registry::HistoryEntryScope;
    use super::*;
    use crate::{
        data::repositories::history::reading_history_repo::ReadingHistoryRepository,
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    async fn setup() -> (sqlx::SqlitePool, HistorySyncService) {
        let pool = setup_test_db_with_comic().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', 'p', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        (pool.clone(), HistorySyncService::new(pool))
    }

    fn noop_emitter() -> EventEmitter {
        Arc::new(|_, _| {})
    }

    #[tokio::test]
    async fn outbound_pushes_only_the_scoped_comic_and_inbound_applies_it() {
        let (outbound_pool, outbound_service) = setup().await;
        let (_inbound_pool, inbound_service) = setup().await;

        ReadingHistoryRepository::new(outbound_pool)
            .upsert(&crate::data::models::history::reading_history::ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 7,
                is_completed: false,
                updated_at: 5000,
            })
            .await
            .unwrap();

        let registry = PendingHistoryEntryRegistry::new();
        let peer = PeerIdentity { id: "peer-a".to_string(), device_id: None };
        registry.set(
            peer.id.clone(),
            HistoryEntryScope { comic_name: "Test".to_string(), chapter_ids: vec![1] },
        );

        let outbound = HistoryEntrySyncOutbound::new(noop_emitter(), outbound_service, registry);
        let inbound = HistoryEntrySyncInbound::new(noop_emitter(), inbound_service.clone());

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let outbound_fut = outbound.handle(&peer, Box::new(client_send), Box::new(client_recv));
        let inbound_fut = inbound.handle(&peer, Box::new(server_send), Box::new(server_recv));

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound deveria completar sem erro");
        inbound_result.expect("inbound deveria completar sem erro");

        let applied = inbound_service.build_manifest().await.unwrap();
        assert_eq!(applied.entries.len(), 1);
        assert_eq!(applied.entries[0].comic_name, "Test");
        assert_eq!(applied.entries[0].last_page, 7);
    }

    #[tokio::test]
    async fn outbound_only_sends_read_markers_for_the_selected_chapters() {
        use crate::data::repositories::history::chapter_read_repo::ChapterReadRepository;

        let (outbound_pool, outbound_service) = setup().await;
        let (inbound_pool, inbound_service) = setup().await;

        for pool in [&outbound_pool, &inbound_pool] {
            sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
                .execute(pool)
                .await
                .unwrap();
        }
        ChapterReadRepository::new(outbound_pool.clone())
            .insert_batch(1, &[1, 2], 900)
            .await
            .unwrap();

        let registry = PendingHistoryEntryRegistry::new();
        let peer = PeerIdentity { id: "peer-b".to_string(), device_id: None };
        registry.set(
            peer.id.clone(),
            HistoryEntryScope { comic_name: "Test".to_string(), chapter_ids: vec![2] },
        );

        let outbound = HistoryEntrySyncOutbound::new(noop_emitter(), outbound_service, registry);
        let inbound = HistoryEntrySyncInbound::new(noop_emitter(), inbound_service.clone());

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let outbound_fut = outbound.handle(&peer, Box::new(client_send), Box::new(client_recv));
        let inbound_fut = inbound.handle(&peer, Box::new(server_send), Box::new(server_recv));

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound deveria completar sem erro");
        inbound_result.expect("inbound deveria completar sem erro");

        let applied = inbound_service.build_manifest().await.unwrap();
        assert_eq!(applied.read_markers.len(), 1);
        assert_eq!(applied.read_markers[0].chapter, "2");
    }

    #[tokio::test]
    async fn outbound_fails_fast_when_no_scope_was_registered_for_the_peer() {
        let (_, service) = setup().await;
        let registry = PendingHistoryEntryRegistry::new();
        let outbound = HistoryEntrySyncOutbound::new(noop_emitter(), service, registry);
        let peer = PeerIdentity { id: "peer-without-scope".to_string(), device_id: None };

        let (client_io, _server_io) = tokio::io::duplex(1024);
        let (client_recv, client_send) = tokio::io::split(client_io);

        let result = outbound.handle(&peer, Box::new(client_send), Box::new(client_recv)).await;
        assert!(result.is_err());
    }
}
