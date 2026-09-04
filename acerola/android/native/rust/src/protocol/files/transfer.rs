//! Abstração de transferência de bytes de um capítulo entre peers, usada pela fase 3 de
//! `exchange.rs` (`publish_chapter_to_blob`/`receive_chapter_from_blob`). Existe como trait (em
//! vez de `Arc<BlobContext>` direto) só pra manter os testes de `exchange.rs` rápidos e sem
//! rede real — `InMemoryChapterTransfer` simula o par publish/fetch com um `HashMap`
//! compartilhado entre os dois lados do teste, o mesmo espírito do `InMemoryFileSyncProvider`
//! já usado ali. Em produção, `BlobChapterTransfer` é a única implementação usada.

use std::sync::Arc;

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity};
use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::protocol::blob_context::BlobContext;

#[async_trait]
pub(crate) trait ChapterTransfer: Send + Sync {
    /// Publica os bytes localmente (`P2pBlobStore::put`) e devolve o hash de blob (hex) —
    /// vai no `FileHeader.blob_hash` que o outro lado usa pra buscar de volta.
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError>;

    /// Busca os bytes referenciados por `blob_hash` no `peer` informado (`P2pBlobStore::fetch`
    /// seguido de `get`) e devolve um leitor — quem chama decide como consumir (chunk a chunk,
    /// pra progresso incremental via `FileSyncProvider::write_chapter_chunk`).
    async fn fetch_reader(
        &self,
        blob_hash: &str,
        peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError>;
}

pub(crate) struct BlobChapterTransfer {
    context: Arc<BlobContext>,
}

impl BlobChapterTransfer {
    pub(crate) fn new(context: Arc<BlobContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl ChapterTransfer for BlobChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        let store = self.context.blob_store().await?;
        // `P2pBlobStore::put` já retorna `ConnectionError` (= `P2pError`, mesmo tipo — ver
        // `acerola_p2p::api::error`), já classificado corretamente na origem
        // (`classify_connection_error`/`classify_get_error` em `lib/p2p`). Propaga direto em vez
        // de achatar em `StreamFailed(err.to_string())` — isso jogava fora a classificação e
        // obrigava `classify_sync_error` (`mod.rs`) a adivinhar a causa via substring matching
        // no texto, frágil e sem garantia de cobrir os casos reais.
        let hash = store.put(bytes).await?;
        Ok(hash.to_string())
    }

    async fn fetch_reader(
        &self,
        blob_hash: &str,
        peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let store = self.context.blob_store().await?;
        // Formato do hash é local/estático (não vem da rede) — sem paralelo em `ConnectionError`
        // além do genérico `StreamFailed`, mantém `.map_err` só aqui.
        let hash = blob_hash
            .parse()
            .map_err(|_| P2pError::StreamFailed(format!("invalid blob hash: {blob_hash}")))?;
        let addr = self.context.resolve_addr(peer).await?;

        // Ver comentário em `publish` — propaga o `ConnectionError` estruturado direto.
        store.fetch(&hash, &addr).await?;
        store.get(&hash).await
    }
}

#[cfg(test)]
pub(crate) struct InMemoryChapterTransfer {
    blobs: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

#[cfg(test)]
impl InMemoryChapterTransfer {
    /// Cria um par de handles que compartilham o mesmo `HashMap` interno — simula dois lados
    /// distintos publicando/buscando no MESMO "store de rede" (o hash de um lado já está
    /// disponível pro outro, exatamente como conteúdo content-addressed real seria).
    pub(crate) fn shared_pair() -> (Arc<dyn ChapterTransfer>, Arc<dyn ChapterTransfer>) {
        let shared = Arc::new(InMemoryChapterTransfer {
            blobs: std::sync::Mutex::new(std::collections::HashMap::new()),
        });
        (
            Arc::clone(&shared) as Arc<dyn ChapterTransfer>,
            shared as Arc<dyn ChapterTransfer>,
        )
    }
}

#[cfg(test)]
#[async_trait]
impl ChapterTransfer for InMemoryChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        let hash = blake3::hash(&bytes).to_hex().to_string();
        self.blobs.lock().unwrap().insert(hash.clone(), bytes);
        Ok(hash)
    }

    async fn fetch_reader(
        &self,
        blob_hash: &str,
        _peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let bytes = self
            .blobs
            .lock()
            .unwrap()
            .get(blob_hash)
            .cloned()
            .ok_or_else(|| {
                P2pError::StreamFailed(format!("unknown blob hash in test transfer: {blob_hash}"))
            })?;

        // `tokio::io::duplex` já implementa `AsyncRead`/`AsyncWrite` — mais simples que
        // implementar um `poll_read` manual só pra teste. Escreve tudo e fecha (EOF) antes de
        // devolver a ponta de leitura; o buffer é dimensionado pro tamanho exato do payload
        // pra não bloquear a escrita.
        use tokio::io::AsyncWriteExt;
        let (mut writer, reader) = tokio::io::duplex(bytes.len().max(1));
        tokio::spawn(async move {
            let _ = writer.write_all(&bytes).await;
            let _ = writer.shutdown().await;
        });
        Ok(Box::new(reader))
    }
}
