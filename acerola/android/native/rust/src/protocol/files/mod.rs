mod exchange;
mod model;
mod session_guard;
pub(crate) mod transfer;

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

use crate::{callbacks::FileSyncProvider, protocol::sync_error::classify_sync_error};
use model::FileSyncStats;
pub(crate) use model::SyncDirection;
pub(crate) use session_guard::FileSyncSessionGuard;
pub(crate) use transfer::{BlobChapterTransfer, ChapterTransfer};

pub(crate) const FILE_SYNC_ALPN: &[u8] = b"acerola/sync-files/1";

/// ALPN do protocolo `acerola/sync-comic/1` — mesma máquina de estados do `sync-files`
/// (manifesto -> want-list -> transferência), mas filtrada a um único quadrinho por uma fase 0
/// extra (`ComicSyncScope`, ver `exchange::run_exchange_scoped`). ALPN dedicado (em vez de
/// reaproveitar `FILE_SYNC_ALPN`) porque o outro lado precisa rotear pro handler certo antes de
/// saber que a sessão é escopada — a única forma de expressar isso sem tocar a lib de
/// transporte (`acerola-p2p`, que roteia por ALPN exato e não carrega payload em `connect()`).
pub(crate) const COMIC_SYNC_ALPN: &[u8] = b"acerola/sync-comic/1";

/// `peer_id -> (comic_name, direction)` escolhidos pelo usuário na chamada FFI
/// `P2PNode::sync_comic`, lidos e removidos por `ComicSyncOutbound::handle` assim que a sessão
/// outbound começa. Único jeito de levar essa escolha (que só existe do lado Kotlin) até o
/// `Handler` que a lib de transporte invoca sem contexto por chamada — ver comentário em
/// `COMIC_SYNC_ALPN`.
pub(crate) type PendingComicScope = Arc<Mutex<HashMap<String, (String, SyncDirection)>>>;

/// Papel outbound do protocolo `acerola/sync-files/1` — este lado iniciou a conexão.
pub(crate) struct FileSyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncOutbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
        }
    }
}

#[async_trait]
impl Handler for FileSyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        run_and_report(
            true,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            send,
            recv,
        )
        .await
    }
}

/// Papel inbound do protocolo `acerola/sync-files/1` — o peer iniciou a conexão.
pub(crate) struct FileSyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncInbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
        }
    }
}

#[async_trait]
impl Handler for FileSyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        run_and_report(
            false,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            send,
            recv,
        )
        .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_and_report(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    session_guard: &Arc<FileSyncSessionGuard>,
    transfer: &Arc<dyn ChapterTransfer>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let result = match session_guard.try_acquire(&peer.id) {
        Some(_lease) => {
            exchange::run_exchange(outbound_role, peer, emit, provider, transfer, send, recv).await
        }
        None => {
            let reason = format!(
                "sync-files session already in progress for peer {}",
                peer.id
            );
            // Melhor esforço: avisa o outro lado do stream que a rejeição foi por ocupação, não
            // por falha de conexão. Se essa escrita falhar (peer já caiu, stream já fechado),
            // ignora — a sessão já foi rejeitada localmente de qualquer forma, o erro abaixo é
            // retornado igual.
            let _ = exchange::write_session_busy(send, &reason).await;
            Err(P2pError::StreamFailed(reason))
        }
    };

    match result {
        Ok(stats) => {
            emit("sync:files:complete", complete_payload(peer, &stats));
            Ok(())
        }
        Err(err) => {
            let reason = err.to_string();
            let code = classify_sync_error(&err);
            // Log técnico sempre em inglês, independente do `code` — é o que vai pro log/
            // suporte; a tradução (`code`) é só pra UI.
            tracing::warn!(peer = %peer.id, ?code, error = %reason, "[FileSync] session failed");
            emit(
                "sync:files:chapter_failed",
                serde_json::json!({
                    "peerId": peer.id,
                    "comicName": "",
                    "chapter": "",
                    "reason": reason,
                    "code": code,
                })
                .to_string(),
            );
            Err(err)
        }
    }
}

/// Papel outbound do protocolo `acerola/sync-comic/1` — este lado discou. Recupera (e remove)
/// o `comic_name` que o usuário escolheu, gravado em `pending_scope` por `P2PNode::sync_comic`
/// logo antes de chamar `connect()` — é a única forma de essa escolha, que só existe do lado
/// Kotlin, chegar até aqui (ver `COMIC_SYNC_ALPN`).
pub(crate) struct ComicSyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
    pending_scope: PendingComicScope,
}

impl ComicSyncOutbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
        pending_scope: PendingComicScope,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
            pending_scope,
        }
    }
}

#[async_trait]
impl Handler for ComicSyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let pending = self
            .pending_scope
            .lock()
            .expect("pending comic scope mutex poisoned")
            .remove(&peer.id);
        run_and_report_scoped(
            true,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            pending,
            send,
            recv,
        )
        .await
    }
}

/// Papel inbound do protocolo `acerola/sync-comic/1` — o peer discou pedindo (ou oferecendo) um
/// quadrinho específico. O `comic_name` chega pelo wire (fase 0, `ComicSyncScope`), não é
/// conhecido localmente antes da sessão começar — por isso não precisa de `pending_scope`.
pub(crate) struct ComicSyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl ComicSyncInbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
        }
    }
}

#[async_trait]
impl Handler for ComicSyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        run_and_report_scoped(
            false,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            None,
            send,
            recv,
        )
        .await
    }
}

/// Mesma orquestração de `run_and_report` (guard -> exchange -> emitir complete/failed), só que
/// pro exchange escopado a um quadrinho (`exchange::run_exchange_scoped`). `session_guard` é a
/// MESMA instância compartilhada com `acerola/sync-files/1` (ver `api.rs::P2PNode::new`) — um
/// sync completo e um sync de um quadrinho só para o mesmo peer nunca rodam ao mesmo tempo,
/// exatamente pelo mesmo motivo (dobrar a varredura de biblioteca/I-O) que já vale entre duas
/// sessões `sync-files`. Por isso os eventos emitidos são os MESMOS `sync:files:*` de sempre —
/// nunca há ambiguidade sobre qual sessão está ativa pra um peer.
#[allow(clippy::too_many_arguments)]
async fn run_and_report_scoped(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    session_guard: &Arc<FileSyncSessionGuard>,
    transfer: &Arc<dyn ChapterTransfer>,
    comic_scope_outbound: Option<(String, SyncDirection)>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let result = match session_guard.try_acquire(&peer.id) {
        Some(_lease) => {
            exchange::run_exchange_scoped(
                outbound_role,
                peer,
                emit,
                provider,
                transfer,
                comic_scope_outbound,
                send,
                recv,
            )
            .await
        }
        None => {
            let reason = format!(
                "sync-comic session already in progress for peer {}",
                peer.id
            );
            let _ = exchange::write_session_busy(send, &reason).await;
            Err(P2pError::StreamFailed(reason))
        }
    };

    match result {
        Ok(stats) => {
            emit("sync:files:complete", complete_payload(peer, &stats));
            Ok(())
        }
        Err(err) => {
            let reason = err.to_string();
            let code = classify_sync_error(&err);
            // Log técnico sempre em inglês, independente do `code` — é o que vai pro log/
            // suporte; a tradução (`code`) é só pra UI.
            tracing::warn!(peer = %peer.id, ?code, error = %reason, "[FileSync] session failed");
            emit(
                "sync:files:chapter_failed",
                serde_json::json!({
                    "peerId": peer.id,
                    "comicName": "",
                    "chapter": "",
                    "reason": reason,
                    "code": code,
                })
                .to_string(),
            );
            Err(err)
        }
    }
}

fn complete_payload(peer: &PeerIdentity, stats: &FileSyncStats) -> String {
    serde_json::json!({
        "peerId": peer.id,
        "receivedCount": stats.received_count,
        "sentCount": stats.sent_count,
        "failedCount": stats.failed_count,
    })
    .to_string()
}

#[cfg(test)]
mod concurrency_tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex as StdMutex,
    };

    use super::*;
    use crate::callbacks::FfiFileManifestEntry;

    fn test_transfer() -> Arc<dyn ChapterTransfer> {
        transfer::InMemoryChapterTransfer::shared_pair().0
    }

    struct CountingProvider {
        manifest_calls: Arc<AtomicUsize>,
    }

    impl FileSyncProvider for CountingProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            self.manifest_calls.fetch_add(1, Ordering::SeqCst);
            vec![]
        }
        fn get_library_summary(&self) -> Vec<crate::callbacks::FfiComicSummaryEntry> {
            vec![]
        }
        fn open_chapter_for_read(&self, _comic_name: String, _chapter: String) -> i64 {
            -1
        }
        fn read_chapter_chunk(&self, _handle: i64, _chunk_size: u32) -> Vec<u8> {
            vec![]
        }
        fn close_read_handle(&self, _handle: i64) {}
        fn begin_chapter_write(
            &self,
            _comic_name: String,
            _chapter: String,
            _file_name: String,
            _expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            -1
        }
        fn write_chapter_chunk(&self, _handle: i64, _bytes: Vec<u8>) -> bool {
            false
        }
        fn finalize_chapter_write(&self, _handle: i64) -> bool {
            false
        }
        fn abort_chapter_write(&self, _handle: i64) {}
        fn get_extras_manifest(&self) -> Vec<crate::callbacks::FfiExtraManifestEntry> {
            vec![]
        }
        fn open_extra_for_read(&self, _comic_name: String, _kind: String) -> i64 {
            -1
        }
        fn begin_extra_write(
            &self,
            _comic_name: String,
            _kind: String,
            _file_name: String,
            _expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            -1
        }
        fn finalize_extra_write(&self, _handle: i64) -> bool {
            false
        }
        fn abort_extra_write(&self, _handle: i64) {}
    }

    /// Prova o fio completo: com o lease da primeira sessão ainda em mãos, uma segunda
    /// tentativa pro MESMO peer é rejeitada sem nunca tocar o provider (nenhuma varredura de
    /// biblioteca disparada), e emite o evento já existente `sync:files:chapter_failed` — sem
    /// inventar um evento novo fora do padrão.
    #[tokio::test]
    async fn second_concurrent_session_for_same_peer_is_rejected_and_emits_chapter_failed() {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-x".into(),
            device_id: None,
        };

        let events: Arc<StdMutex<Vec<(String, String)>>> = Arc::new(StdMutex::new(vec![]));
        let events_clone = Arc::clone(&events);
        let emit: EventEmitter = Arc::new(move |event, data| {
            events_clone.lock().unwrap().push((event.to_string(), data));
        });

        // Primeira sessão adquire o lease e "fica em andamento" (mantido vivo manualmente).
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        let (_client, server) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(server);
        let result = run_and_report(
            true,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &test_transfer(),
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(
            manifest_calls.load(Ordering::SeqCst),
            0,
            "provider não deveria ser tocado na sessão rejeitada"
        );

        let recorded = events.lock().unwrap();
        let (event_name, payload) = recorded.last().unwrap();
        assert_eq!(event_name, "sync:files:chapter_failed");
        assert!(payload.contains("already in progress"));
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(
            parsed["code"], "busy",
            "lado Kotlin usa esse code pra traduzir a mensagem na UI"
        );
        drop(recorded);

        drop(lease); // libera a primeira sessão
        assert!(session_guard.try_acquire(&peer.id).is_some());
    }

    /// Antes deste fix, o lado rejeitado não escrevia nada no stream — quem esperava do outro
    /// lado só via o stream fechar sem contexto (`StreamFailed("stream closed")`), igual a uma
    /// conexão que caiu de verdade. Agora a rejeição escreve `SessionBusy` no stream antes de
    /// devolver o erro local, então quem lê consegue distinguir "peer ocupado" de "conexão
    /// realmente caiu" pela mensagem de erro.
    #[tokio::test]
    async fn peer_reading_rejected_stream_sees_busy_error_not_generic_stream_failed() {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-x".into(),
            device_id: None,
        };
        let emit: EventEmitter = Arc::new(|_event, _data| {});

        // Sessão já em andamento localmente (simula a stream #1 aceita na race do fix 1).
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        // Dois pipes unidirecionais (não um `duplex` bidirecional só) de propósito: um
        // `SendStream`/`RecvStream` do iroh são objetos independentes — fechar um lado da
        // leitura de um não derruba a escrita do outro. Um `duplex` bidirecional único acopla
        // as duas direções por trás de um `Arc` compartilhado, então dropar o lado do
        // rejeitador (que acontece assim que `run_and_report` retorna, ainda dentro do mesmo
        // `join!`) derrubaria também a escrita do lado do dialer antes dele conseguir mandar o
        // manifesto — um artefato do mock, não um comportamento real do transporte.
        let (dialer_to_server, server_reads_dialer) = tokio::io::duplex(4096);
        let (server_to_dialer, dialer_reads_server) = tokio::io::duplex(4096);
        let (_unused_read, client_send) = tokio::io::split(dialer_to_server);
        let (server_recv, _unused_write) = tokio::io::split(server_reads_dialer);
        let (_unused_read2, server_send) = tokio::io::split(server_to_dialer);
        let (client_recv, _unused_write2) = tokio::io::split(dialer_reads_server);

        let dialer_transfer = test_transfer();
        let rejecter_transfer = test_transfer();

        // Lado que discou (outbound) esperando o manifesto normalmente.
        let dialer = exchange::run_exchange(
            true,
            &peer,
            &emit,
            &provider,
            &dialer_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        // Lado que já está ocupado (simula a stream #2 duplicada, rejeitada pelo guard).
        let rejecter = run_and_report(
            false,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &rejecter_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (dialer_result, rejecter_result) = tokio::join!(dialer, rejecter);

        let dialer_error = dialer_result.expect_err("dialer deveria falhar, peer estava ocupado");
        assert!(
            dialer_error.to_string().contains("peer busy"),
            "esperava erro tipado de peer ocupado, recebeu: {dialer_error}"
        );

        let rejecter_error = rejecter_result.expect_err("rejeição local também deve retornar erro");
        assert!(rejecter_error.to_string().contains("already in progress"));
    }

    /// `acerola/sync-comic/1` compartilha a MESMA instância de `FileSyncSessionGuard` com
    /// `acerola/sync-files/1` (ver `api.rs::P2PNode::new`) — uma sessão de sync completo já em
    /// andamento pro peer deve rejeitar uma tentativa de sync de um quadrinho só, mesmo que os
    /// ALPNs sejam diferentes. Sem isso, os dois protocolos rodariam ao mesmo tempo pro mesmo
    /// peer e dobrariam a varredura de biblioteca via `run_blocking`.
    #[tokio::test]
    async fn comic_sync_session_is_rejected_while_a_full_sync_files_session_is_active_for_the_same_peer(
    ) {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-shared-guard".into(),
            device_id: None,
        };
        let emit: EventEmitter = Arc::new(|_event, _data| {});

        // Simula uma sessão `sync-files` completa já em andamento pro peer.
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        let (_client, server) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(server);
        let result = run_and_report_scoped(
            true,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &test_transfer(),
            Some(("Comic A".to_string(), SyncDirection::Push)),
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(
            result.is_err(),
            "sync-comic deveria ser rejeitado com sync-files ativo pro mesmo peer"
        );
        assert_eq!(
            manifest_calls.load(Ordering::SeqCst),
            0,
            "provider não deveria ser tocado na sessão rejeitada"
        );
    }

    /// Simétrico ao teste acima: uma sessão `sync-comic` já em andamento também rejeita uma
    /// tentativa de `sync-files` completo pro mesmo peer, pelo mesmo guard compartilhado.
    #[tokio::test]
    async fn full_sync_files_session_is_rejected_while_a_comic_sync_session_is_active_for_the_same_peer(
    ) {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-shared-guard-2".into(),
            device_id: None,
        };
        let emit: EventEmitter = Arc::new(|_event, _data| {});

        // Simula uma sessão `sync-comic` já em andamento pro peer.
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        let (_client, server) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(server);
        let result = run_and_report(
            true,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &test_transfer(),
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(
            result.is_err(),
            "sync-files deveria ser rejeitado com sync-comic ativo pro mesmo peer"
        );
        assert_eq!(
            manifest_calls.load(Ordering::SeqCst),
            0,
            "provider não deveria ser tocado na sessão rejeitada"
        );
    }
}
