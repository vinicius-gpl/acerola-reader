use std::{path::PathBuf, sync::Arc};

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

use crate::{
    core::services::sync::file_sync::FileSyncService,
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        messages::{
            CoverRequest, CoverResponse, COVER_STATUS_CHANGED, COVER_STATUS_NOT_MODIFIED,
            COVER_STATUS_UNAVAILABLE,
        },
        protocol::{
            cover_request_registry::PendingCoverRequestRegistry,
            transfer::{
                classify_sync_error, sync_error_payload, ChapterTransfer, NO_PENDING_SCOPE_REASON,
            },
        },
    },
};

/// Lado que RESPONDE: lê `CoverRequest`, resolve a capa local (`FileSyncService::get_local_cover`)
/// e responde `unavailable`/`not_modified`/`changed` — só publica no blob store local
/// (`ChapterTransfer::publish`) quando a versão realmente mudou.
pub struct CoverBrowseInbound {
    service: FileSyncService,
    transfer: Arc<dyn ChapterTransfer>,
}

impl CoverBrowseInbound {
    pub fn new(service: FileSyncService, transfer: Arc<dyn ChapterTransfer>) -> Self {
        Self { service, transfer }
    }

    async fn run(
        &self, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        let request: CoverRequest = read_json(reader).await?;
        let local_cover = self.service.get_local_cover(&request.comic_name).await?;

        let response = match local_cover {
            None => CoverResponse {
                status: COVER_STATUS_UNAVAILABLE.to_string(),
                cover_version: None,
                cover_hash: None,
            },
            Some((cover_version, _)) if request.known_version == Some(cover_version) => {
                CoverResponse {
                    status: COVER_STATUS_NOT_MODIFIED.to_string(),
                    cover_version: Some(cover_version),
                    cover_hash: None,
                }
            },
            Some((cover_version, bytes)) => {
                let hash = self.transfer.publish(bytes).await?;
                CoverResponse {
                    status: COVER_STATUS_CHANGED.to_string(),
                    cover_version: Some(cover_version),
                    cover_hash: Some(hash),
                }
            },
        };

        write_json(writer, &response).await?;
        Ok(())
    }
}

#[async_trait]
impl Handler for CoverBrowseInbound {
    async fn handle(
        &self, _peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);
        self.run(&mut writer, &mut reader).await
    }
}

/// Lado que INICIA a busca: escreve `CoverRequest`, lê a resposta e — só se
/// `status == "changed"` — busca os bytes de verdade via `ChapterTransfer::fetch_reader` (que já
/// verifica integridade via BLAKE3 antes de devolver o leitor). O `comic_name`/`known_version`
/// vêm de `PendingCoverRequestRegistry` (mesmo padrão de `ComicSyncOutbound`/
/// `PendingComicSyncRegistry`) — o `Handler` é um singleton, não recebe parâmetro por chamada de
/// `connect()`.
pub struct CoverBrowseOutbound {
    emit: EventEmitter,
    transfer: Arc<dyn ChapterTransfer>,
    registry: Arc<PendingCoverRequestRegistry>,
    /// Cache dedicado (nunca a árvore do usuário), `<app_data_dir>/remote_covers` — chave
    /// `(peer_id, comic_name, cover_version)` embutida no nome do arquivo, pra nunca rebaixar a
    /// mesma versão duas vezes.
    cache_dir: PathBuf,
}

impl CoverBrowseOutbound {
    pub fn new(
        emit: EventEmitter, transfer: Arc<dyn ChapterTransfer>,
        registry: Arc<PendingCoverRequestRegistry>, cache_dir: PathBuf,
    ) -> Self {
        Self { emit, transfer, registry, cache_dir }
    }

    fn sanitize(value: &str) -> String {
        value
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
            .collect()
    }

    async fn save_to_cache(
        &self, peer_id: &str, comic_name: &str, cover_version: i64, bytes: &[u8],
    ) -> Result<String, P2pError> {
        tokio::fs::create_dir_all(&self.cache_dir).await.map_err(|err| {
            P2pError::StreamFailed(format!("failed to create remote covers cache dir: {err}"))
        })?;

        let file_name = format!(
            "{}_{}_{}.jpg",
            Self::sanitize(peer_id),
            Self::sanitize(comic_name),
            cover_version
        );
        let path = self.cache_dir.join(file_name);

        tokio::fs::write(&path, bytes).await.map_err(|err| {
            P2pError::StreamFailed(format!("failed to write cached cover: {err}"))
        })?;

        Ok(path.to_string_lossy().to_string())
    }

    async fn run(
        &self, comic_name: &str, known_version: Option<i64>, peer: &PeerIdentity,
        writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        write_json(writer, &CoverRequest { comic_name: comic_name.to_string(), known_version })
            .await?;

        let response: CoverResponse = read_json(reader).await?;

        match response.status.as_str() {
            COVER_STATUS_NOT_MODIFIED => {
                let cover_version = response.cover_version.ok_or_else(|| {
                    P2pError::StreamFailed(
                        "cover response missing cover_version for not_modified".into(),
                    )
                })?;
                self.emit_result(peer, comic_name, "not_modified", Some(cover_version), None);
            },
            COVER_STATUS_CHANGED => {
                let cover_version = response.cover_version.ok_or_else(|| {
                    P2pError::StreamFailed(
                        "cover response missing cover_version for changed".into(),
                    )
                })?;
                let hash = response.cover_hash.ok_or_else(|| {
                    P2pError::StreamFailed("cover response missing cover_hash for changed".into())
                })?;

                let mut blob_reader = self.transfer.fetch_reader(&hash, peer).await?;
                let mut bytes = Vec::new();
                blob_reader
                    .read_to_end(&mut bytes)
                    .await
                    .map_err(|err| P2pError::StreamFailed(err.to_string()))?;

                // Grava num cache local (`<app_data_dir>/remote_covers`) e emite o CAMINHO, não
                // os bytes — o Svelte já resolve caminhos locais pra `<img>` via
                // `convertFileSrc`/`resolveArtworkPath` (`artwork.utils.ts`), mesmo padrão já
                // usado pras capas locais.
                let path = self.save_to_cache(&peer.id, comic_name, cover_version, &bytes).await?;
                self.emit_result(peer, comic_name, "changed", Some(cover_version), Some(&path));
            },
            COVER_STATUS_UNAVAILABLE => {
                self.emit_result(peer, comic_name, "unavailable", None, None);
            },
            other => {
                return Err(P2pError::StreamFailed(format!(
                    "unknown cover response status: {other}"
                )))
            },
        }

        Ok(())
    }

    fn emit_result(
        &self, peer: &PeerIdentity, comic_name: &str, status: &str, cover_version: Option<i64>,
        path: Option<&str>,
    ) {
        (self.emit)(
            "browse:cover:result",
            serde_json::json!({
                "peerId": peer.id,
                "comicName": comic_name,
                "status": status,
                "coverVersion": cover_version,
                "path": path,
            })
            .to_string(),
        );
    }
}

#[async_trait]
impl Handler for CoverBrowseOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let Some((comic_name, known_version)) = self.registry.take(&peer.id) else {
            let error = P2pError::StreamFailed(NO_PENDING_SCOPE_REASON.into());
            let code = classify_sync_error(&error);
            (self.emit)(
                "browse:cover:error",
                sync_error_payload(&peer.id, &error.to_string(), code, None),
            );
            return Err(error);
        };

        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        if let Err(err) = self.run(&comic_name, known_version, peer, &mut writer, &mut reader).await
        {
            let message = err.to_string();
            let code = classify_sync_error(&err);
            tracing::warn!(peer = %peer.id, comic_name = %comic_name, ?code, error = %message, "[CoverBrowse] session failed");
            (self.emit)(
                "browse:cover:error",
                sync_error_payload(&peer.id, &message, code, Some(&comic_name)),
            );
            return Err(err);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::infra::sync::protocol::transfer::InMemoryChapterTransfer;

    // Mesmo alias usado em `comic_handler.rs`/`file_handler.rs` — evita o lint de
    // `type_complexity` do clippy pro tipo de retorno de `mock_emitter` abaixo.
    type RecordedEvents = Arc<Mutex<Vec<(String, String)>>>;

    fn mock_emitter() -> (EventEmitter, RecordedEvents) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);
        let emit: EventEmitter =
            Arc::new(move |name, data| events_clone.lock().unwrap().push((name.to_string(), data)));
        (emit, events)
    }

    async fn setup_service_with_cover(
        cover_version: i64, cover_bytes: &[u8],
    ) -> (FileSyncService, tempfile::TempDir) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let cover_path = temp_dir.path().join("cover.jpg");
        std::fs::write(&cover_path, cover_bytes).unwrap();

        crate::tests::utils::setup_test_db::insert_comic_directory_with_cover(
            &pool,
            1,
            "Comic A",
            "/test",
            &cover_path.to_string_lossy(),
            cover_version,
        )
        .await;

        let root = temp_dir.path().to_path_buf();
        (FileSyncService::new(pool, move || root.clone()), temp_dir)
    }

    fn make_peer(id: &str) -> PeerIdentity {
        PeerIdentity { id: id.to_string(), device_id: None }
    }

    async fn run_pair(
        service: FileSyncService, known_version: Option<i64>,
    ) -> Vec<(String, String)> {
        let (inbound_transfer, outbound_transfer) = InMemoryChapterTransfer::shared_pair();
        let (emit, events) = mock_emitter();

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let inbound = CoverBrowseInbound::new(service, inbound_transfer);
        let mut inbound_writer =
            framed_writer(Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>);
        let mut inbound_reader =
            framed_reader(Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>);

        let peer = make_peer("peer-cover");
        let registry = PendingCoverRequestRegistry::new();
        let outbound =
            CoverBrowseOutbound::new(emit, outbound_transfer, registry, std::env::temp_dir());
        let mut outbound_writer =
            framed_writer(Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>);
        let mut outbound_reader =
            framed_reader(Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>);

        let inbound_fut = inbound.run(&mut inbound_writer, &mut inbound_reader);
        let outbound_fut = outbound.run(
            "Comic A",
            known_version,
            &peer,
            &mut outbound_writer,
            &mut outbound_reader,
        );

        let (inbound_result, outbound_result) = tokio::join!(inbound_fut, outbound_fut);
        inbound_result.expect("inbound deveria completar sem erro");
        outbound_result.expect("outbound deveria completar sem erro");

        let recorded = events.lock().unwrap().clone();
        recorded
    }

    fn result_json(events: &[(String, String)]) -> serde_json::Value {
        let (_, payload) = events.iter().find(|(name, _)| name == "browse:cover:result").unwrap();
        serde_json::from_str(payload).unwrap()
    }

    #[tokio::test]
    async fn known_version_matching_local_returns_not_modified() {
        let (service, _dir) = setup_service_with_cover(42, b"cover bytes").await;

        let events = run_pair(service, Some(42)).await;
        let result = result_json(&events);

        assert_eq!(result["status"], "not_modified");
        assert_eq!(result["coverVersion"], 42);
    }

    #[tokio::test]
    async fn no_local_cover_returns_unavailable() {
        let pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        crate::tests::utils::setup_test_db::insert_comic_directory(&pool, 1, "Comic A", "/test")
            .await;
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool, move || root.clone());

        let events = run_pair(service, None).await;
        let result = result_json(&events);

        assert_eq!(result["status"], "unavailable");
    }

    #[tokio::test]
    async fn stale_known_version_fetches_real_bytes_via_blob_transfer() {
        let cover_bytes = b"brand new cover jpeg bytes".to_vec();
        let (service, _dir) = setup_service_with_cover(7, &cover_bytes).await;

        let events = run_pair(service, Some(3)).await;
        let result = result_json(&events);

        assert_eq!(result["status"], "changed");
        assert_eq!(result["coverVersion"], 7);
        let path = result["path"].as_str().unwrap();
        let written = std::fs::read(path).unwrap();
        assert_eq!(written, cover_bytes);
        std::fs::remove_file(path).ok();
    }
}
