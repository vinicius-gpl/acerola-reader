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
    data::{models::sync::SyncHistoryLogEntry, repositories::sync::SyncHistoryLogRepository},
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        messages::FileWantList,
        protocol::{
            file_session_guard::FileSyncSessionGuard,
            transfer::{
                classify_sync_error, emit_busy, read_or_busy, receive_extras, receive_files,
                send_extras, send_files, sync_error_payload, write_session_busy, ChapterTransfer,
                SyncErrorCode,
            },
        },
    },
};

const LOG_KIND: &str = "files";
const STARTED_EVENT: &str = "sync:files:started";
const PROGRESS_EVENT: &str = "sync:files:progress";
const COMPLETE_EVENT: &str = "sync:files:complete";
const ERROR_EVENT: &str = "sync:files:error";

/// Lado que INICIA a sessão de sync de arquivos. Sequência (cada passo alterna quem
/// escreve/lê, nunca os dois escrevendo ao mesmo tempo no stream):
/// 1. escreve manifesto local → 2. lê manifesto do peer → 3. escreve o que quer → 4. lê o
///    que o peer quer → 5. lê os arquivos que pediu (o peer escreve primeiro nessa fase) →
///    6. escreve os arquivos que o peer pediu.
pub struct FileSyncOutbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncOutbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, transfer }
    }

    /// Ver doc de `receive_files`/`send_files` (`transfer.rs`) — o `usize` retornado é a soma
    /// de capítulos/extras que não foram transferidos de verdade nas quatro fases. `handle()`
    /// usa isso pra não reportar a sessão como "completa" quando algo ficou faltando.
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<usize, P2pError> {
        let local_manifest = self.service.build_manifest().await?;
        write_json(writer, &local_manifest).await?;

        let peer_manifest = read_or_busy(reader).await?;

        let (my_wanted, my_wanted_extras) = self.service.diff_wanted(&peer_manifest).await?;
        write_json(
            writer,
            &FileWantList { wanted: my_wanted.clone(), wanted_extras: my_wanted_extras.clone() },
        )
        .await?;

        let their_wanted: FileWantList = read_json(reader).await?;

        // Fase 1: o peer envia primeiro o que eu pedi (capítulos, depois extras).
        let received_skipped = receive_files(
            reader,
            my_wanted.len(),
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;
        let received_extras_skipped = receive_extras(
            reader,
            my_wanted_extras.len(),
            &self.service,
            &self.metadata_service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;

        // Fase 2: eu envio o que o peer pediu (capítulos, depois extras).
        let sent_skipped = send_files(
            writer,
            &their_wanted.wanted,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;
        let sent_extras_skipped = send_extras(
            writer,
            &their_wanted.wanted_extras,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;

        Ok(received_skipped + received_extras_skipped + sent_skipped + sent_extras_skipped)
    }
}

#[async_trait]
impl Handler for FileSyncOutbound {
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
            Ok(0) => {
                (self.emit)(COMPLETE_EVENT, peer.id.clone());
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Ok(skipped) => {
                let message =
                    format!("{skipped} chapter(s) not synced — see backend log for details");
                tracing::warn!(peer = %peer.id, skipped, "[FileSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    sync_error_payload(&peer.id, &message, Some(SyncErrorCode::PartialSync), None),
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
                let code = classify_sync_error(&error);
                // Log técnico sempre em inglês, independente do `code` — é o que vai pro
                // arquivo de log/suporte; a tradução (`code`) é só pra UI.
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[FileSync] session failed");
                (self.emit)(ERROR_EVENT, sync_error_payload(&peer.id, &message, code, None));
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

/// Lado que RESPONDE à sessão de sync de arquivos — mesma sequência lógica do outbound,
/// com os papéis de leitura/escrita invertidos em cada passo.
pub struct FileSyncInbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncInbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, transfer }
    }

    /// Ver doc equivalente em `FileSyncOutbound::run`.
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<usize, P2pError> {
        let peer_manifest = read_or_busy(reader).await?;

        let local_manifest = self.service.build_manifest().await?;
        write_json(writer, &local_manifest).await?;

        let their_wanted: FileWantList = read_json(reader).await?;

        let (my_wanted, my_wanted_extras) = self.service.diff_wanted(&peer_manifest).await?;
        write_json(
            writer,
            &FileWantList { wanted: my_wanted.clone(), wanted_extras: my_wanted_extras.clone() },
        )
        .await?;

        // Fase 1: eu envio primeiro o que o peer (outbound) pediu (capítulos, depois extras).
        let sent_skipped = send_files(
            writer,
            &their_wanted.wanted,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;
        let sent_extras_skipped = send_extras(
            writer,
            &their_wanted.wanted_extras,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;

        // Fase 2: eu recebo o que eu pedi (capítulos, depois extras).
        let received_skipped = receive_files(
            reader,
            my_wanted.len(),
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;
        let received_extras_skipped = receive_extras(
            reader,
            my_wanted_extras.len(),
            &self.service,
            &self.metadata_service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;

        Ok(sent_skipped + sent_extras_skipped + received_skipped + received_extras_skipped)
    }
}

#[async_trait]
impl Handler for FileSyncInbound {
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
            Ok(0) => {
                (self.emit)(COMPLETE_EVENT, peer.id.clone());
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Ok(skipped) => {
                let message =
                    format!("{skipped} chapter(s) not synced — see backend log for details");
                tracing::warn!(peer = %peer.id, skipped, "[FileSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    sync_error_payload(&peer.id, &message, Some(SyncErrorCode::PartialSync), None),
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
                let code = classify_sync_error(&error);
                // Log técnico sempre em inglês, independente do `code` — é o que vai pro
                // arquivo de log/suporte; a tradução (`code`) é só pra UI.
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[FileSync] session failed");
                (self.emit)(ERROR_EVENT, sync_error_payload(&peer.id, &message, code, None));
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

    async fn setup(
    ) -> (SyncHistoryLogRepository, FileSyncService, Arc<MetadataService>, tempfile::TempDir) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool.clone(), move || root.clone());
        let metadata_service = Arc::new(MetadataService::new(pool.clone()));
        let log_repo = SyncHistoryLogRepository::new(pool);
        (log_repo, service, metadata_service, temp_dir)
    }

    /// Reproduz a corrida documentada no fix: uma segunda sessão inbound chega pro mesmo
    /// peer enquanto uma já está em andamento. Sem o guard, `handle()` chamaria
    /// `self.run()` (que abre o manifesto via `FileSyncService`) mesmo com outra sessão
    /// ativa. Como a lease já está reservada antes de `handle()` rodar, a asserção de que
    /// só o evento de "busy" foi emitido — nunca "started" — é a prova de que `run()` (e,
    /// portanto, o `FileSyncService`) nunca chegou a ser tocado.
    #[tokio::test]
    async fn inbound_rejects_second_session_for_same_peer_without_touching_the_service() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-busy".to_string(), device_id: None };
        let _held_lease = guard.try_acquire(&peer.id).expect("primeira reserva deveria suceder");

        let inbound =
            FileSyncInbound::new(emit, service, metadata_service, log_repo, guard, test_transfer());

        let result =
            inbound.handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty())).await;

        assert!(result.is_ok());
        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 1, "esperava só o evento de busy, recebeu: {recorded:?}");
        assert_eq!(recorded[0].0, "sync:files:error");
        assert!(recorded[0].1.contains("already in progress"));
        let payload: serde_json::Value = serde_json::from_str(&recorded[0].1).unwrap();
        assert_eq!(payload["code"], "busy", "frontend usa esse code pra traduzir a mensagem na UI");
    }

    /// Mesma corrida do teste acima, mas cruzando papéis: a lease foi reservada por uma
    /// sessão inbound e a segunda tentativa é outbound — prova que o guard é compartilhado
    /// entre os dois papéis, não só entre sessões do mesmo tipo.
    #[tokio::test]
    async fn outbound_and_inbound_share_the_same_guard_per_peer() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-cross".to_string(), device_id: None };
        let _held_lease = guard.try_acquire(&peer.id).expect("reserva inicial deveria suceder");

        let outbound = FileSyncOutbound::new(
            emit,
            service,
            metadata_service,
            log_repo,
            guard,
            test_transfer(),
        );

        let result = outbound
            .handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty()))
            .await;

        assert!(result.is_ok());
        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "sync:files:error");
    }

    /// Sem nenhuma lease pré-existente, `handle()` deve seguir normalmente até `run()` —
    /// confirma que o guard não bloqueia sessões novas depois que a anterior é liberada.
    #[tokio::test]
    async fn inbound_proceeds_to_run_when_no_session_is_active() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-free".to_string(), device_id: None };
        let inbound =
            FileSyncInbound::new(emit, service, metadata_service, log_repo, guard, test_transfer());

        let _ =
            inbound.handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty())).await;

        let recorded = events.lock().unwrap();
        assert!(recorded.iter().any(|(name, _)| name == "sync:files:started"));
    }
}
