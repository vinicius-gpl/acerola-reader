use std::sync::Arc;

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    core::services::{metadata::MetadataService, sync::file_sync::FileSyncService},
    data::{
        models::sync::SyncHistoryLogEntry,
        repositories::sync::SyncHistoryLogRepository,
    },
    infra::sync::{
        framing::{framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter},
        messages::{ComicSyncRequest, FileManifest, FileWantList},
        protocol::{
            comic_sync_registry::PendingComicSyncRegistry,
            file_session_guard::FileSyncSessionGuard,
            transfer::{
                classify_sync_error, emit_busy, read_or_busy, receive_extras, receive_files, send_extras,
                send_files, write_session_busy, ChapterTransfer,
            },
        },
    },
};

const LOG_KIND: &str = "comic";
const STARTED_EVENT: &str = "sync:comic:started";
const PROGRESS_EVENT: &str = "sync:comic:progress";
const COMPLETE_EVENT: &str = "sync:comic:complete";
const ERROR_EVENT: &str = "sync:comic:error";

/// Lado que INICIA um sync individual de UM quadrinho (`acerola/sync-comic/1`). Reaproveita o
/// mesmo mecanismo de diff/want-list/transfer de `acerola/sync-files/1`
/// (`file_handler.rs`/`transfer.rs`), só trocando manifesto de biblioteca inteira por
/// manifesto escopado a um `comic_name` — ver o comentário de design no
/// `FileSyncService::build_manifest_for_comic`. Isso resolve push e pull com o mesmo código:
/// se eu tenho o quadrinho e o peer não, o manifesto dele vem vazio e o `diff_wanted` dele pede
/// tudo de mim; se é o contrário, o meu vem vazio e sou eu que peço tudo dele.
///
/// Sequência (uma mensagem a mais que `FileSyncOutbound`, no começo, pra informar QUAL
/// quadrinho — sem isso o lado que responde não teria como escopar quando o meu manifesto
/// vier vazio, caso "pull"):
/// 1. escreve `ComicSyncRequest{comic_name}` → 2. escreve manifesto local escopado → 3. lê
///    manifesto do peer (escopado) → 4. escreve o que quer → 5. lê o que o peer quer → 6. lê
///    os arquivos que pediu → 7. escreve os arquivos que o peer pediu.
pub struct ComicSyncOutbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    registry: Arc<PendingComicSyncRegistry>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl ComicSyncOutbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>,
        registry: Arc<PendingComicSyncRegistry>, transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, registry, transfer }
    }

    /// Retorna `(skipped, comic_name)` — `skipped` é quantos capítulos/extras (de qualquer uma
    /// das duas fases) não foram transferidos de verdade, usado por `handle()` pra decidir se a
    /// sessão pode mesmo ser reportada como "completa" (ver doc de `receive_files`/`send_files`
    /// em `transfer.rs`); `comic_name` viaja de volta só pra `handle()` poder incluir no
    /// payload do evento `sync:comic:complete`/`error`, já que só `run()` sabe qual foi o
    /// quadrinho escopado desta sessão.
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(usize, String), P2pError> {
        let comic_name = self.registry.take(&peer.id).ok_or_else(|| {
            P2pError::StreamFailed("no pending comic sync scope registered for this peer".into())
        })?;

        write_json(writer, &ComicSyncRequest { comic_name: comic_name.clone() }).await?;

        let local_manifest = self.service.build_manifest_for_comic(&comic_name).await?;
        write_json(writer, &local_manifest).await?;

        let peer_manifest: FileManifest = read_or_busy(reader).await?;

        let (my_wanted, my_wanted_extras) = self.service.diff_wanted(&peer_manifest).await?;
        write_json(
            writer,
            &FileWantList { wanted: my_wanted.clone(), wanted_extras: my_wanted_extras.clone() },
        )
        .await?;

        let their_wanted: FileWantList = read_json(reader).await?;

        // Fase 1: o peer envia primeiro o que eu pedi (capítulos, depois extras).
        let received_skipped = receive_files(
            reader, my_wanted.len(), &self.service, &self.emit, PROGRESS_EVENT, ERROR_EVENT, peer,
            &self.transfer,
        )
        .await?;
        let received_extras_skipped = receive_extras(
            reader, my_wanted_extras.len(), &self.service, &self.metadata_service, &self.emit, PROGRESS_EVENT,
            ERROR_EVENT, peer, &self.transfer,
        )
        .await?;

        // Fase 2: eu envio o que o peer pediu (capítulos, depois extras).
        let sent_skipped =
            send_files(writer, &their_wanted.wanted, &self.service, &self.emit, PROGRESS_EVENT, &self.transfer)
                .await?;
        let sent_extras_skipped = send_extras(
            writer, &their_wanted.wanted_extras, &self.service, &self.emit, PROGRESS_EVENT, &self.transfer,
        )
        .await?;

        Ok((received_skipped + received_extras_skipped + sent_skipped + sent_extras_skipped, comic_name))
    }
}

#[async_trait]
impl Handler for ComicSyncOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let Some(_lease) = self.guard.try_acquire(&peer.id) else {
            write_session_busy(send).await;
            emit_busy(&self.emit, ERROR_EVENT, &peer.id);
            return Ok(());
        };

        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)(STARTED_EVENT, peer.id.clone());

        match self.run(peer, &mut writer, &mut reader).await {
            Ok((0, comic_name)) => {
                (self.emit)(
                    COMPLETE_EVENT,
                    serde_json::json!({ "peerId": peer.id, "comicName": comic_name }).to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            // Sessão terminou sem erro de protocolo, mas nem todos os capítulos chegaram —
            // reportar como "complete" aqui esconderia exatamente o tipo de perda de dado
            // silenciosa que motivou essa mudança (ver doc de `receive_files`).
            Ok((skipped, comic_name)) => {
                let message = format!(
                    "{skipped} capítulo(s) não sincronizado(s) — veja o log do backend pra detalhes"
                );
                tracing::warn!(peer = %peer.id, skipped, "[ComicSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    serde_json::json!({ "peerId": peer.id, "comicName": comic_name, "message": &message })
                        .to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(P2pError::StreamFailed(message))
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&message);
                (self.emit)(
                    ERROR_EVENT,
                    serde_json::json!({ "peerId": peer.id, "message": &message, "code": code }).to_string(),
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

/// Lado que RESPONDE à sessão de sync individual — mesma sequência lógica do outbound, com os
/// papéis de leitura/escrita invertidos em cada passo. O `comic_name` a escopar vem do
/// `ComicSyncRequest` recebido, não de um registro local (só o lado que inicia precisa do
/// `PendingComicSyncRegistry`).
pub struct ComicSyncInbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl ComicSyncInbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>, transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, transfer }
    }

    /// Ver doc equivalente em `ComicSyncOutbound::run` — mesmo contrato de retorno
    /// `(skipped, comic_name)`.
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(usize, String), P2pError> {
        let request: ComicSyncRequest = read_or_busy(reader).await?;

        let peer_manifest: FileManifest = read_json(reader).await?;

        let local_manifest = self.service.build_manifest_for_comic(&request.comic_name).await?;
        write_json(writer, &local_manifest).await?;

        let their_wanted: FileWantList = read_json(reader).await?;

        let (my_wanted, my_wanted_extras) = self.service.diff_wanted(&peer_manifest).await?;
        write_json(
            writer,
            &FileWantList { wanted: my_wanted.clone(), wanted_extras: my_wanted_extras.clone() },
        )
        .await?;

        // Fase 1: eu envio primeiro o que o peer (outbound) pediu (capítulos, depois extras).
        let sent_skipped =
            send_files(writer, &their_wanted.wanted, &self.service, &self.emit, PROGRESS_EVENT, &self.transfer)
                .await?;
        let sent_extras_skipped = send_extras(
            writer, &their_wanted.wanted_extras, &self.service, &self.emit, PROGRESS_EVENT, &self.transfer,
        )
        .await?;

        // Fase 2: eu recebo o que eu pedi (capítulos, depois extras).
        let received_skipped = receive_files(
            reader, my_wanted.len(), &self.service, &self.emit, PROGRESS_EVENT, ERROR_EVENT, peer,
            &self.transfer,
        )
        .await?;
        let received_extras_skipped = receive_extras(
            reader, my_wanted_extras.len(), &self.service, &self.metadata_service, &self.emit, PROGRESS_EVENT,
            ERROR_EVENT, peer, &self.transfer,
        )
        .await?;

        Ok((sent_skipped + sent_extras_skipped + received_skipped + received_extras_skipped, request.comic_name))
    }
}

#[async_trait]
impl Handler for ComicSyncInbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let Some(_lease) = self.guard.try_acquire(&peer.id) else {
            write_session_busy(send).await;
            emit_busy(&self.emit, ERROR_EVENT, &peer.id);
            return Ok(());
        };

        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)(STARTED_EVENT, peer.id.clone());

        match self.run(peer, &mut writer, &mut reader).await {
            Ok((0, comic_name)) => {
                (self.emit)(
                    COMPLETE_EVENT,
                    serde_json::json!({ "peerId": peer.id, "comicName": comic_name }).to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Ok((skipped, comic_name)) => {
                let message = format!(
                    "{skipped} capítulo(s) não sincronizado(s) — veja o log do backend pra detalhes"
                );
                tracing::warn!(peer = %peer.id, skipped, "[ComicSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    serde_json::json!({ "peerId": peer.id, "comicName": comic_name, "message": &message })
                        .to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(P2pError::StreamFailed(message))
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&message);
                (self.emit)(
                    ERROR_EVENT,
                    serde_json::json!({ "peerId": peer.id, "message": &message, "code": code }).to_string(),
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use acerola_p2p::api::peer::PeerIdentity;

    use super::*;
    use crate::infra::sync::protocol::transfer::InMemoryChapterTransfer;

    type RecordedEvents = Arc<Mutex<Vec<(String, String)>>>;

    fn test_transfer() -> Arc<dyn ChapterTransfer> {
        InMemoryChapterTransfer::shared_pair().0
    }

    fn mock_emitter() -> (EventEmitter, RecordedEvents) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);
        let emit: EventEmitter =
            Arc::new(move |name, data| events_clone.lock().unwrap().push((name.to_string(), data)));
        (emit, events)
    }

    async fn setup() -> (SyncHistoryLogRepository, FileSyncService, Arc<MetadataService>, tempfile::TempDir) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool.clone(), move || root.clone());
        let metadata_service = Arc::new(MetadataService::new(pool.clone()));
        let log_repo = SyncHistoryLogRepository::new(pool);
        (log_repo, service, metadata_service, temp_dir)
    }

    /// Sem `comic_name` registrado pro peer, o outbound aborta a sessão sem nunca chegar a
    /// escrever nada no stream — evita disparar o resto do protocolo com um escopo
    /// indefinido, o que aconteceria se o comando Tauri `sync_comic` esquecesse de chamar
    /// `registry.set(...)` antes de `connect()`.
    #[tokio::test]
    async fn outbound_fails_fast_when_no_comic_is_pending_for_the_peer() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-no-scope".to_string(), device_id: None };
        let outbound =
            ComicSyncOutbound::new(emit, service, metadata_service, log_repo, guard, registry, test_transfer());

        let result = outbound
            .handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty()))
            .await;

        assert!(result.is_err());
        let recorded = events.lock().unwrap();
        assert!(recorded.iter().any(|(name, _)| name == "sync:comic:error"));
    }

    /// Mesma corrida documentada em `file_handler.rs`, aplicada ao protocolo individual: uma
    /// segunda sessão pro mesmo peer enquanto uma já está ativa é rejeitada sem tocar o
    /// serviço, reaproveitando o `FileSyncSessionGuard` compartilhado com `FILE_SYNC_ALPN`.
    #[tokio::test]
    async fn inbound_rejects_second_session_for_same_peer_without_touching_the_service() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-busy".to_string(), device_id: None };
        let _held_lease = guard.try_acquire(&peer.id).expect("primeira reserva deveria suceder");

        let inbound = ComicSyncInbound::new(emit, service, metadata_service, log_repo, guard, test_transfer());

        let result =
            inbound.handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty())).await;

        assert!(result.is_ok());
        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 1, "esperava só o evento de busy, recebeu: {recorded:?}");
        assert_eq!(recorded[0].0, "sync:comic:error");
        assert!(recorded[0].1.contains("already in progress"));
        let payload: serde_json::Value = serde_json::from_str(&recorded[0].1).unwrap();
        assert_eq!(payload["code"], "busy", "frontend usa esse code pra traduzir a mensagem na UI");
    }

    /// Trava o payload novo de `sync:comic:complete` — antes carregava só a string crua do
    /// peer id (`peer.id.clone()`), agora um JSON com `comicName` (o quadrinho escopado desta
    /// sessão) pra o frontend saber SE precisa atualizar a página do quadrinho aberto sem
    /// precisar adivinhar por qual peer/kind veio (ver `use-network-sync.svelte.ts`).
    #[tokio::test]
    async fn complete_event_payload_carries_the_scoped_comic_name() {
        let (outbound_log, outbound_service, outbound_metadata, _outbound_dir) = setup().await;
        let (inbound_log, inbound_service, inbound_metadata, _inbound_dir) = setup().await;

        let guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (outbound_emit, outbound_events) = mock_emitter();
        let (inbound_emit, inbound_events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-complete".to_string(), device_id: None };
        registry.set(peer.id.clone(), "Quadrinho Compartilhado".to_string());

        let outbound = ComicSyncOutbound::new(
            outbound_emit,
            outbound_service,
            outbound_metadata,
            outbound_log,
            Arc::clone(&guard),
            registry,
            test_transfer(),
        );
        let inbound =
            ComicSyncInbound::new(inbound_emit, inbound_service, inbound_metadata, inbound_log, guard, test_transfer());

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let outbound_fut = outbound.handle(&peer, Box::new(client_send), Box::new(client_recv));
        let inbound_fut = inbound.handle(&peer, Box::new(server_send), Box::new(server_recv));

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound deveria completar sem erro (nenhum capítulo em nenhum dos lados)");
        inbound_result.expect("inbound deveria completar sem erro");

        let outbound_recorded = outbound_events.lock().unwrap();
        let complete_event = outbound_recorded
            .iter()
            .find(|(name, _)| name == "sync:comic:complete")
            .expect("deveria ter emitido sync:comic:complete");
        let payload: serde_json::Value = serde_json::from_str(&complete_event.1).unwrap();
        assert_eq!(payload["comicName"], "Quadrinho Compartilhado");
        assert_eq!(payload["peerId"], "peer-complete");

        let inbound_recorded = inbound_events.lock().unwrap();
        let inbound_complete = inbound_recorded
            .iter()
            .find(|(name, _)| name == "sync:comic:complete")
            .expect("inbound também deveria ter emitido sync:comic:complete");
        let inbound_payload: serde_json::Value = serde_json::from_str(&inbound_complete.1).unwrap();
        assert_eq!(inbound_payload["comicName"], "Quadrinho Compartilhado");
    }
}
