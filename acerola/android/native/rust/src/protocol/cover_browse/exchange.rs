use std::{sync::Arc, time::Duration};

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity};
use futures::SinkExt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::model::{
    CoverRequest, CoverResponse, STATUS_CHANGED, STATUS_NOT_MODIFIED, STATUS_UNAVAILABLE,
};
use crate::{
    callbacks::CoverBrowseProvider,
    protocol::{ffi_blocking::run_blocking, files::ChapterTransfer},
};

const FRAME_READ_TIMEOUT: Duration = Duration::from_secs(15);

type Recv = FramedRead<Box<dyn AsyncRead + Send + Unpin>, LengthDelimitedCodec>;
type Writer = FramedWrite<Box<dyn AsyncWrite + Send + Unpin>, LengthDelimitedCodec>;

async fn write_json<T: serde::Serialize>(writer: &mut Writer, value: &T) -> Result<(), P2pError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|err| P2pError::StreamFailed(format!("failed to encode cover message: {err}")))?;
    writer
        .send(bytes.into())
        .await
        .map_err(|err| P2pError::StreamFailed(format!("failed to write cover message: {err}")))
}

async fn read_json<T: serde::de::DeserializeOwned>(reader: &mut Recv) -> Result<T, P2pError> {
    let frame = tokio::time::timeout(FRAME_READ_TIMEOUT, reader.next())
        .await
        .map_err(|_| P2pError::StreamFailed("timed out reading cover message".into()))?
        .ok_or_else(|| P2pError::StreamFailed("stream closed before cover message".into()))?
        .map_err(|err| P2pError::StreamFailed(format!("failed to read cover message: {err}")))?;

    serde_json::from_slice(&frame)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode cover message: {err}")))
}

/// Papel inbound: lê `CoverRequest`, resolve a capa local (`CoverBrowseProvider::get_local_cover`)
/// e responde `unavailable`/`not_modified`/`changed` — só publica no blob store local
/// (`ChapterTransfer::publish`) quando a versão realmente mudou, pra não pagar o custo de hash
/// numa capa que o outbound já tem.
pub(super) async fn run_inbound(
    provider: &Arc<dyn CoverBrowseProvider>,
    transfer: &Arc<dyn ChapterTransfer>,
    writer: &mut Writer,
    reader: &mut Recv,
) -> Result<(), P2pError> {
    let request: CoverRequest = read_json(reader).await?;

    let provider_clone = Arc::clone(provider);
    let comic_name = request.comic_name.clone();
    let entry = run_blocking(move || provider_clone.get_local_cover(comic_name)).await?;

    let response = match entry.bytes {
        None => CoverResponse {
            status: STATUS_UNAVAILABLE.to_string(),
            cover_version: None,
            cover_hash: None,
        },
        Some(_) if request.known_version == Some(entry.cover_version) => CoverResponse {
            status: STATUS_NOT_MODIFIED.to_string(),
            cover_version: Some(entry.cover_version),
            cover_hash: None,
        },
        Some(bytes) => {
            let hash = transfer.publish(bytes).await?;
            CoverResponse {
                status: STATUS_CHANGED.to_string(),
                cover_version: Some(entry.cover_version),
                cover_hash: Some(hash),
            }
        }
    };

    write_json(writer, &response).await
}

/// Resultado de uma sessão outbound — o que o chamador (`mod.rs::CoverBrowseOutbound::handle`)
/// precisa pra decidir o evento certo pro Kotlin.
pub(super) enum CoverOutcome {
    NotModified { cover_version: i64 },
    Unavailable,
    Fetched { cover_version: i64, bytes: Vec<u8> },
}

/// Papel outbound: escreve `CoverRequest`, lê a resposta e — só se `status == "changed"` — busca
/// os bytes de verdade via `ChapterTransfer::fetch_reader` (que já verifica integridade via
/// BLAKE3 antes de devolver o leitor).
pub(super) async fn run_outbound(
    comic_name: String,
    known_version: Option<i64>,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
    writer: &mut Writer,
    reader: &mut Recv,
) -> Result<CoverOutcome, P2pError> {
    write_json(
        writer,
        &CoverRequest {
            comic_name,
            known_version,
        },
    )
    .await?;

    let response: CoverResponse = read_json(reader).await?;

    match response.status.as_str() {
        STATUS_NOT_MODIFIED => {
            let cover_version = response.cover_version.ok_or_else(|| {
                P2pError::StreamFailed(
                    "cover response missing cover_version for not_modified".into(),
                )
            })?;
            Ok(CoverOutcome::NotModified { cover_version })
        }
        STATUS_CHANGED => {
            let cover_version = response.cover_version.ok_or_else(|| {
                P2pError::StreamFailed("cover response missing cover_version for changed".into())
            })?;
            let hash = response.cover_hash.ok_or_else(|| {
                P2pError::StreamFailed("cover response missing cover_hash for changed".into())
            })?;

            let mut blob_reader = transfer.fetch_reader(&hash, peer).await?;
            let mut bytes = Vec::new();
            blob_reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|err| P2pError::StreamFailed(err.to_string()))?;

            Ok(CoverOutcome::Fetched {
                cover_version,
                bytes,
            })
        }
        STATUS_UNAVAILABLE => Ok(CoverOutcome::Unavailable),
        other => Err(P2pError::StreamFailed(format!(
            "unknown cover response status: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{callbacks::FfiCoverEntry, protocol::files::transfer::InMemoryChapterTransfer};

    struct StubCoverProvider {
        cover_version: i64,
        bytes: Option<Vec<u8>>,
    }

    impl CoverBrowseProvider for StubCoverProvider {
        fn get_local_cover(&self, _comic_name: String) -> FfiCoverEntry {
            FfiCoverEntry {
                cover_version: self.cover_version,
                bytes: self.bytes.clone(),
            }
        }

        fn save_remote_cover(
            &self,
            _peer_id: String,
            _comic_name: String,
            _cover_version: i64,
            _bytes: Vec<u8>,
        ) -> String {
            unreachable!("save_remote_cover roda só do lado outbound, não usado nestes testes")
        }
    }

    fn make_peer(id: &str) -> PeerIdentity {
        PeerIdentity {
            id: id.to_string(),
            device_id: None,
        }
    }

    async fn run_pair(
        inbound_provider: Arc<dyn CoverBrowseProvider>,
        known_version: Option<i64>,
    ) -> Result<CoverOutcome, P2pError> {
        let (inbound_transfer, outbound_transfer) = InMemoryChapterTransfer::shared_pair();

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let mut inbound_writer = FramedWrite::new(
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut inbound_reader = FramedRead::new(
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut outbound_writer = FramedWrite::new(
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut outbound_reader = FramedRead::new(
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        let peer = make_peer("peer-cover");
        let inbound_fut = run_inbound(
            &inbound_provider,
            &inbound_transfer,
            &mut inbound_writer,
            &mut inbound_reader,
        );
        let outbound_fut = run_outbound(
            "Comic A".to_string(),
            known_version,
            &peer,
            &outbound_transfer,
            &mut outbound_writer,
            &mut outbound_reader,
        );

        let (inbound_result, outbound_result) = tokio::join!(inbound_fut, outbound_fut);
        inbound_result.expect("inbound deveria completar sem erro");
        outbound_result
    }

    #[tokio::test]
    async fn known_version_matching_local_returns_not_modified() {
        let provider: Arc<dyn CoverBrowseProvider> = Arc::new(StubCoverProvider {
            cover_version: 42,
            bytes: Some(b"cover bytes".to_vec()),
        });

        let outcome = run_pair(provider, Some(42)).await.unwrap();

        match outcome {
            CoverOutcome::NotModified { cover_version } => assert_eq!(cover_version, 42),
            _ => panic!("esperava NotModified"),
        }
    }

    #[tokio::test]
    async fn no_local_cover_returns_unavailable() {
        let provider: Arc<dyn CoverBrowseProvider> = Arc::new(StubCoverProvider {
            cover_version: 0,
            bytes: None,
        });

        let outcome = run_pair(provider, None).await.unwrap();

        assert!(matches!(outcome, CoverOutcome::Unavailable));
    }

    #[tokio::test]
    async fn stale_known_version_fetches_real_bytes_via_blob_transfer() {
        let cover_bytes = b"brand new cover jpeg bytes".to_vec();
        let provider: Arc<dyn CoverBrowseProvider> = Arc::new(StubCoverProvider {
            cover_version: 7,
            bytes: Some(cover_bytes.clone()),
        });

        let outcome = run_pair(provider, Some(3)).await.unwrap();

        match outcome {
            CoverOutcome::Fetched {
                cover_version,
                bytes,
            } => {
                assert_eq!(cover_version, 7);
                assert_eq!(bytes, cover_bytes);
            }
            _ => panic!("esperava Fetched"),
        }
    }
}
