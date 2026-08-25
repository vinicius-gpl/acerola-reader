use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    core::services::sync::file_sync::FileSyncService,
    infra::sync::{
        framing::{framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter},
        messages::{LibraryBrowseRequest, LibrarySummary},
    },
};

/// Round-trip único (sem `FileSyncSessionGuard`: só lista títulos, não transfere nada, mesmo
/// perfil de custo do `history_handler.rs`) pra descobrir o que existe na biblioteca remota
/// antes de decidir sincronizar um quadrinho específico (`acerola/sync-comic/1`). Não persiste
/// em `sync_history_log` — é uma consulta de navegação, não um sync de conteúdo.
///
/// O pedido em si não carrega nenhum dado (ver `LibraryBrowseRequest`) — mas precisa ser
/// ESCRITO no wire por QUEM DISCA (`open_bi()`), não só "existir como intenção". Regra da
/// própria `quinn`: quem chama `open_bi()` precisa escrever no `SendStream` antes do lado que
/// aceita conseguir `accept_bi()` — sem isso o handler inbound nunca é sequer invocado. Era
/// exatamente o bug reaberto em 22/08/2026 (timeout de 30s nos dois lados): esta função só
/// lia, nunca escrevia.
///
/// Importante: essa regra é sobre MATERIALIZAR o stream pro lado que aceita, não sobre o lado
/// que aceita precisar LER o que foi escrito antes de responder — isso é uma decisão de
/// protocolo separada, e `LibraryBrowseInbound` deliberadamente NÃO lê esse marcador (ver doc
/// lá). Uma correção anterior, nesta mesma rodada, tentou fazer o inbound drenar o marcador
/// antes de responder — isso quebrou a sessão inteira sempre que a materialização do stream
/// atrasa (NAT traversal lento: a conexão "estabelece" a nível de handshake bem antes do path
/// de dados ficar pronto pra carregar o marcador), porque acoplava a resposta do inbound a essa
/// entrega, quando o inbound não precisa dela pra nada.
pub struct LibraryBrowseOutbound {
    emit: EventEmitter,
}

impl LibraryBrowseOutbound {
    pub fn new(emit: EventEmitter) -> Self {
        Self { emit }
    }

    async fn run(
        &self, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<LibrarySummary, P2pError> {
        write_json(writer, &LibraryBrowseRequest::default()).await?;
        let response: LibrarySummary = read_json(reader).await?;
        Ok(response)
    }
}

#[async_trait]
impl Handler for LibraryBrowseOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        match self.run(&mut writer, &mut reader).await {
            Ok(response) => {
                // `ComicSummaryEntry` (mensagem de wire, `snake_case`, compartilhada com o
                // Android) não é o mesmo formato que o payload de evento pro frontend
                // (`camelCase`, mesma convenção de `ConnectedPeerPayload`/`PairedPeerPayload` em
                // `cmd/events/network`) — monta o JSON do evento à mão em vez de serializar a
                // struct de wire direto, mesmo padrão já usado pelos eventos `sync:*:error`
                // deste módulo (`serde_json::json!` ad-hoc, sem struct compartilhada entre
                // `infra` e `cmd`).
                let comics: Vec<serde_json::Value> = response
                    .comics
                    .iter()
                    .map(|comic| {
                        serde_json::json!({
                            "comicName": comic.comic_name,
                            "chapterCount": comic.chapter_count,
                            "coverVersion": comic.cover_version,
                        })
                    })
                    .collect();

                (self.emit)(
                    "library:query:result",
                    serde_json::json!({ "peerId": peer.id, "comics": comics }).to_string(),
                );
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                (self.emit)(
                    "library:query:error",
                    serde_json::json!({ "peerId": peer.id, "message": &message }).to_string(),
                );
                Err(error)
            },
        }
    }
}

/// Lado que RESPONDE: monta o resumo da biblioteca local a partir do mesmo
/// `FileSyncService::build_manifest()` usado pelo sync de arquivos (sem duplicar a lógica de
/// listar quadrinhos/capítulos) e escreve assim que a conexão é aceita — NÃO espera nem lê o
/// marcador que `LibraryBrowseOutbound` escreve (ver doc lá): aquele marcador existe só pra
/// satisfazer a regra da `quinn` do lado de quem disca, não é algo que este lado precise
/// consumir. Esperar por ele aqui acoplaria a resposta a quanto tempo o path P2P (NAT
/// traversal/multipath) leva pra ficar pronto pra carregar dados de verdade — que pode ser bem
/// mais que o handshake inicial da conexão (o "connection established" nos logs) — e foi
/// exatamente essa espera desnecessária que quebrou a sessão inteira sempre que o outro lado
/// dava timeout esperando uma resposta que nunca vinha porque este handler estava parado
/// esperando ler algo que não precisava.
pub struct LibraryBrowseInbound {
    service: FileSyncService,
}

impl LibraryBrowseInbound {
    pub fn new(service: FileSyncService) -> Self {
        Self { service }
    }

    /// Timeout próprio, bem menor que o `FRAME_TIMEOUT` (30s) do outro lado — se a query travar
    /// (ex: contenção no pool de conexões do SQLite por outra sessão de sync em andamento),
    /// falha rápido e loga a causa em vez de deixar o outbound estourar o timeout de frame sem
    /// nenhuma pista do que aconteceu deste lado. Ver nota no bug reaberto em 22/08/2026: o
    /// handler nunca logava nada, então não dava pra saber se travava aqui ou na escrita.
    const LIBRARY_SUMMARY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

    async fn run(&self, writer: &mut FramedWriter) -> Result<(), P2pError> {
        // SQL puro (`get_library_summary`), sem tocar disco — `build_manifest()` faz um
        // `tokio::fs::metadata()` por capítulo só pra montar `size`, que essa resposta nem usa,
        // e isso estourava o timeout do protocolo em bibliotecas grandes/discos lentos.
        tracing::debug!(layer = "library_browse", "querying local library summary");
        let comics = match tokio::time::timeout(
            Self::LIBRARY_SUMMARY_TIMEOUT,
            self.service.get_library_summary(),
        )
        .await
        {
            Ok(result) => result?,
            Err(_) => {
                tracing::warn!(
                    layer = "library_browse",
                    "get_library_summary did not return within {:?} — DB pool likely contended by another sync session",
                    Self::LIBRARY_SUMMARY_TIMEOUT
                );
                return Err(P2pError::StreamFailed("timed out querying local library summary".into()));
            },
        };

        tracing::debug!(layer = "library_browse", comics = comics.len(), "writing library summary response");
        write_json(writer, &LibrarySummary { comics }).await?;
        tracing::debug!(layer = "library_browse", "library summary response written");
        Ok(())
    }
}

#[async_trait]
impl Handler for LibraryBrowseInbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        _recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        tracing::debug!(layer = "library_browse", peer = %peer.id, "inbound connection accepted");
        let mut writer = framed_writer(send);
        let result = self.run(&mut writer).await;
        if let Err(error) = &result {
            tracing::warn!(layer = "library_browse", peer = %peer.id, %error, "inbound run failed");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_service() -> (FileSyncService, tempfile::TempDir) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db_with_comic().await;
        crate::tests::utils::setup_test_db::insert_chapter_archive_with_checksum(
            &pool,
            1,
            1,
            "Cap 1",
            "p/Cap 1.cbz",
            "abc",
        )
        .await;

        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool, move || root.clone());
        (service, temp_dir)
    }

    /// Round-trip real sobre um `tokio::io::duplex`: prova que o outbound escreve o
    /// `LibraryBrowseRequest` (satisfaz a regra da `quinn` do lado de quem disca) e ainda lê a
    /// resposta certinha, mesmo o inbound NUNCA lendo esse marcador — o inbound responde
    /// direto, sem depender de decodificar nada do outro lado primeiro (ver doc de
    /// `LibraryBrowseInbound`). Regressão de uma correção anterior nesta mesma rodada que fazia
    /// o inbound drenar o marcador antes de responder, o que travava a sessão inteira sempre
    /// que a entrega desse marcador atrasava (NAT traversal lento).
    #[tokio::test]
    async fn outbound_writes_request_but_inbound_never_reads_it_and_still_responds() {
        let (service, _dir) = setup_service().await;
        let inbound = LibraryBrowseInbound::new(service);

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (_server_recv, server_send) = tokio::io::split(server_io);

        let mut writer = framed_writer(Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>);
        inbound.run(&mut writer).await.unwrap();

        let mut outbound_writer = framed_writer(Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>);
        let mut outbound_reader = framed_reader(Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>);
        let outbound = LibraryBrowseOutbound::new(std::sync::Arc::new(|_, _| {}));
        let response = outbound.run(&mut outbound_writer, &mut outbound_reader).await.unwrap();

        assert_eq!(response.comics.len(), 1);
        assert_eq!(response.comics[0].comic_name, "Test");
        assert_eq!(response.comics[0].chapter_count, 1);
    }
}
