use std::{sync::Arc, time::Duration};

use acerola_p2p::api::error::P2pError;
use futures::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::model::{ComicSummaryEntry, LibrarySummary};
use crate::{callbacks::FileSyncProvider, protocol::ffi_blocking::run_blocking};

const RESPONSE_READ_TIMEOUT: Duration = Duration::from_secs(15);

type Recv = FramedRead<Box<dyn AsyncRead + Send + Unpin>, LengthDelimitedCodec>;
type Writer = FramedWrite<Box<dyn AsyncWrite + Send + Unpin>, LengthDelimitedCodec>;

/// Usa `get_library_summary()` (consulta só de Room/SQLite, sem SAF) em vez de
/// `get_file_manifest()` — este último faz um `DocumentFile.exists()` por capítulo (uma
/// transação binder cada), que é custo desnecessário só pra listar títulos e já foi a causa de
/// `browse-library` estourar `RESPONSE_READ_TIMEOUT` numa biblioteca grande. Ver o comentário em
/// `callbacks::FileSyncProvider::get_library_summary`.
pub(super) async fn build_summary(
    provider: &Arc<dyn FileSyncProvider>,
) -> Result<LibrarySummary, P2pError> {
    let provider = Arc::clone(provider);
    let entries = run_blocking(move || provider.get_library_summary()).await?;

    let mut comics: Vec<ComicSummaryEntry> = entries
        .into_iter()
        .map(|entry| ComicSummaryEntry {
            comic_name: entry.comic_name,
            chapter_count: entry.chapter_count,
            cover_version: entry.cover_version,
        })
        .collect();
    comics.sort_by(|a, b| a.comic_name.cmp(&b.comic_name));

    Ok(LibrarySummary { comics })
}

/// Papel inbound: o marcador que `run_outbound` escreve existe só pra satisfazer a regra do
/// quinn do lado de quem disca (`open_bi()` precisa escrever antes do `accept_bi()` remoto
/// enxergar o stream) — este lado NÃO espera nem lê esse marcador antes de responder. Ler aqui
/// acoplaria a resposta a quanto tempo o path P2P (NAT traversal/multipath) leva pra ficar
/// pronto pra carregar dados de verdade, que pode ser bem maior que o handshake inicial da
/// conexão — travando a sessão inteira nesse meio tempo (mesmo problema já corrigido no Desktop,
/// ver `library_browse_handler.rs` lá).
pub(super) async fn run_inbound(
    provider: &Arc<dyn FileSyncProvider>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
) -> Result<(), P2pError> {
    let summary = build_summary(provider).await?;

    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let bytes = serde_json::to_vec(&summary).map_err(|err| {
        P2pError::StreamFailed(format!("failed to encode library summary: {err}"))
    })?;
    writer
        .send(bytes.into())
        .await
        .map_err(|err| P2pError::StreamFailed(format!("failed to write library summary: {err}")))
}

/// Papel outbound: escreve um marcador mínimo (`{}`) antes de esperar a resposta — sem essa
/// escrita, o lado que chamou `open_bi()` nunca garante que o inbound vai ver o stream via
/// `accept_bi()` (regra do quinn, a lib QUIC por baixo do iroh), o que causava timeout/demora
/// aleatória em vez de uma troca previsível.
pub(super) async fn run_outbound(
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<LibrarySummary, P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    writer.send(b"{}".to_vec().into()).await.map_err(|err| {
        P2pError::StreamFailed(format!("failed to write library browse request: {err}"))
    })?;

    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());
    let frame = tokio::time::timeout(RESPONSE_READ_TIMEOUT, reader.next())
        .await
        .map_err(|_| P2pError::StreamFailed("timed out reading library summary".into()))?
        .ok_or_else(|| P2pError::StreamFailed("stream closed before library summary".into()))?
        .map_err(|err| P2pError::StreamFailed(format!("failed to read library summary: {err}")))?;

    serde_json::from_slice(&frame)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode library summary: {err}")))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::callbacks::{FfiComicSummaryEntry, FfiFileManifestEntry};

    struct StubProvider {
        summary: Vec<FfiComicSummaryEntry>,
    }

    impl FileSyncProvider for StubProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            vec![]
        }
        fn get_library_summary(&self) -> Vec<FfiComicSummaryEntry> {
            self.summary.clone()
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

    fn summary_entry(comic_name: &str, chapter_count: u32) -> FfiComicSummaryEntry {
        FfiComicSummaryEntry {
            comic_name: comic_name.to_string(),
            chapter_count,
            cover_version: 0,
        }
    }

    /// `build_summary` só mapeia/ordena o que `get_library_summary()` (agrupamento já feito no
    /// SQL, ver `ChapterArchiveDao.getLibrarySummary`) devolveu — não agrupa mais nada em Rust.
    #[tokio::test]
    async fn build_summary_sorts_by_comic_name() {
        let provider: Arc<dyn FileSyncProvider> = Arc::new(StubProvider {
            summary: vec![summary_entry("Comic B", 1), summary_entry("Comic A", 3)],
        });

        let summary = build_summary(&provider).await.unwrap();

        assert_eq!(summary.comics.len(), 2);
        assert_eq!(summary.comics[0].comic_name, "Comic A");
        assert_eq!(summary.comics[0].chapter_count, 3);
        assert_eq!(summary.comics[1].comic_name, "Comic B");
        assert_eq!(summary.comics[1].chapter_count, 1);
    }

    /// Round-trip real sobre um `tokio::io::duplex`: prova que `run_inbound`/`run_outbound`
    /// falam o mesmo formato de wire (sem depender de nenhum handler/ALPN, só o par de
    /// funções puras de I/O) — o outbound escreve o marcador de pedido, mas o inbound nunca lê
    /// nada, só responde direto (mesma garantia que o teste equivalente do Desktop cobre).
    #[tokio::test]
    async fn run_outbound_writes_request_but_inbound_never_reads_it_and_still_responds() {
        let provider: Arc<dyn FileSyncProvider> = Arc::new(StubProvider {
            summary: vec![summary_entry("Solo Leveling", 2)],
        });

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (_server_recv, server_send) = tokio::io::split(server_io);

        let inbound_fut = run_inbound(
            &provider,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
        );
        let outbound_fut = run_outbound(
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (inbound_result, outbound_result) = tokio::join!(inbound_fut, outbound_fut);
        inbound_result.expect("inbound deveria completar sem erro");
        let summary = outbound_result.expect("outbound deveria receber o resumo sem erro");

        assert_eq!(summary.comics.len(), 1);
        assert_eq!(summary.comics[0].comic_name, "Solo Leveling");
        assert_eq!(summary.comics[0].chapter_count, 2);
    }
}
