use std::sync::Arc;

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity, protocol::EventEmitter};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

use crate::{
    core::services::{metadata::MetadataService, sync::file_sync::FileSyncService},
    infra::{
        error::RpcError,
        sync::{
            blob_context::BlobContext,
            framing::{framed_writer, read_json, write_json, FramedReader, FramedWriter},
            messages::{FileExtraHeader, FileHeader, SessionBusy, EXTRA_KIND_COMIC_INFO},
        },
    },
};

/// Publica/busca os bytes de um capítulo via `P2pBlobStore` (BLAKE3, dedup e verificação
/// automáticos), independente do stream de controle (`FramedReader`/`FramedWriter`) que só
/// carrega o `FileHeader` com o hash. Trait (em vez de `Arc<BlobContext>` direto) só pra manter
/// os testes de `send_files`/`receive_files` sem depender de um `Endpoint` iroh de verdade —
/// `BlobChapterTransfer` é a única implementação usada em produção.
#[async_trait]
pub trait ChapterTransfer: Send + Sync {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError>;

    async fn fetch_reader(
        &self, blob_hash: &str, peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError>;
}

pub struct BlobChapterTransfer {
    context: Arc<BlobContext>,
}

impl BlobChapterTransfer {
    pub fn new(context: Arc<BlobContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl ChapterTransfer for BlobChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        let store = self.context.blob_store().await?;
        let hash = store.put(bytes).await.map_err(|err| P2pError::StreamFailed(err.to_string()))?;
        Ok(hash.to_string())
    }

    async fn fetch_reader(
        &self, blob_hash: &str, peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let store = self.context.blob_store().await?;
        let hash = blob_hash
            .parse()
            .map_err(|_| P2pError::StreamFailed(format!("invalid blob hash: {blob_hash}")))?;
        let addr = self.context.resolve_addr(peer).await?;

        store.fetch(&hash, &addr).await.map_err(|err| P2pError::StreamFailed(err.to_string()))?;
        store.get(&hash).await.map_err(|err| P2pError::StreamFailed(err.to_string()))
    }
}

/// Texto de rejeição usado tanto no evento local (`emit_busy`) quanto na mensagem `SessionBusy`
/// escrita no wire (`write_session_busy`) — mesmo motivo nos dois lugares. Compartilhado entre
/// `acerola/sync-files/1` e `acerola/sync-comic/1`: os dois competem pelo mesmo
/// `FileSyncSessionGuard` por peer (ver `file_session_guard.rs`).
pub const SESSION_BUSY_REASON: &str = "file sync session already in progress for this peer";

/// Valor do campo `error` em `SessionBusy` — mesmo literal usado pelo Android
/// (`protocol/files/exchange.rs::SESSION_BUSY_TAG`), é o que os dois lados usam pra reconhecer
/// essa mensagem específica em vez de um `FileManifest` normal.
pub(super) const SESSION_BUSY_TAG: &str = "busy";

/// Classifica a mensagem técnica de um `P2pError` num `code` estável que o frontend traduz via
/// Paraglide (`SYNC_ERROR_MESSAGES` em `use-network-sync.svelte.ts`) — sem isso, texto de baixo
/// nível da stack de transporte (QUIC/iroh: "stream reset by peer", "connection lost" etc.,
/// visto em produção sem tradução: "blob fetch failed: ...: stream failed: stream failed: io:
/// stream reset by peer: error 3") vaza cru pro usuário final mesmo com o app todo em pt-BR.
/// Cobre só as causas observadas; texto não reconhecido cai no fallback (mensagem técnica crua)
/// já usado por `errors.i18n.ts` pros demais erros do app — não é uma tentativa de mapear toda
/// variante de `ConnectionError`/io error possível, só as recorrentes o bastante pra valer a
/// tradução.
pub(super) fn classify_sync_error(message: &str) -> Option<&'static str> {
    if message.contains(SESSION_BUSY_REASON) {
        return Some(SESSION_BUSY_TAG);
    }
    if message.contains("timed out") {
        return Some("timeout");
    }
    if message.contains("stream failed") || message.contains("stream error") {
        return Some("connection_lost");
    }
    None
}

/// Emitido quando `FileSyncSessionGuard` recusa uma sessão porque já existe uma ativa pro
/// mesmo peer — reaproveita o evento `sync:files:error` já tratado pelo frontend em vez de
/// introduzir um evento novo (ver `use-network-sync.svelte.ts::parseErrorPayload`). O chamador
/// passa o nome do evento (`sync:files:error` ou `sync:comic:error`) porque cada protocolo tem
/// o seu próprio canal. `message` continua em inglês (é texto técnico de log/wire, não deve ser
/// mostrado como está) — `code` é o identificador estável que o frontend usa pra traduzir via
/// Paraglide (`tauri_errors.sync.session_busy.label`), no mesmo padrão de `errors.i18n.ts`.
pub fn emit_busy(emit: &EventEmitter, event_name: &str, peer_id: &str) {
    (emit)(
        event_name,
        serde_json::json!({
            "peerId": peer_id,
            "message": SESSION_BUSY_REASON,
            "code": SESSION_BUSY_TAG,
        })
        .to_string(),
    );
}

/// Escreve a mensagem de rejeição de sessão diretamente no stream, no lugar do manifesto —
/// usado quando `FileSyncSessionGuard::try_acquire` já barrou a sessão em `handle()`, antes do
/// resto do protocolo rodar. Recebe `send` cru (não um `FramedWriter` já montado) porque é
/// exatamente o estado em que `handle()` está no momento da rejeição: o stream ainda não foi
/// envolvido em nenhum framing. Melhor esforço: se a escrita falhar (peer já caiu, stream já
/// fechado), ignora — a sessão já foi rejeitada localmente de qualquer forma.
pub async fn write_session_busy(send: Box<dyn AsyncWrite + Send + Unpin>) {
    let mut writer = framed_writer(send);
    let _ = write_json(
        &mut writer,
        &SessionBusy { error: SESSION_BUSY_TAG.to_string(), reason: SESSION_BUSY_REASON.to_string() },
    )
    .await;
}

/// Lê a primeira mensagem do peer, que pode ser tanto a mensagem esperada (`T`: `FileManifest`
/// inteiro ou escopado a um único quadrinho — ver `FileSyncService::build_manifest_for_comic`
/// — ou `ComicSyncRequest` no protocolo `acerola/sync-comic/1`) quanto uma rejeição
/// `SessionBusy` — sem tag de enum no wire (mesma convenção do resto do protocolo), a única
/// forma de saber qual é espiar o JSON bruto antes de decidir o tipo concreto. Só o(s) ponto(s)
/// de leitura inicial em cada `run()` usa(m) isso: depois da primeira mensagem, uma sessão
/// aceita nunca mais manda `SessionBusy`.
pub async fn read_or_busy<T: DeserializeOwned>(reader: &mut FramedReader) -> Result<T, P2pError> {
    let value: serde_json::Value = read_json(reader).await?;

    let is_busy = value.get("error").and_then(|tag| tag.as_str()) == Some(SESSION_BUSY_TAG);
    if is_busy {
        let reason = value.get("reason").and_then(|r| r.as_str()).unwrap_or("peer is busy");
        return Err(P2pError::StreamFailed(format!("peer busy: {reason}")));
    }

    serde_json::from_value(value).map_err(|err| RpcError::Deserialize(err.to_string()).into())
}

/// Envia, em sequência, os arquivos de `wanted` (o que o peer pediu de nós). Se um item não
/// existir mais localmente (corrida com uma deleção concorrente), envia um header
/// "indisponível" (`size: 0`) em vez de abortar a sessão inteira. Os bytes não trafegam mais
/// no stream de controle — são publicados no blob store local (`ChapterTransfer::publish`) e o
/// hash resultante vai no `FileHeader`; quem recebe busca sob demanda via `fetch_reader`.
///
/// Retorna quantos itens de `wanted` NÃO foram enviados de verdade (arquivo sumiu localmente
/// entre o want-list e o envio) — o chamador usa essa contagem pra decidir se a sessão pode
/// mesmo ser reportada como "completa" pro usuário (ver `ComicSyncOutbound`/`FileSyncOutbound`).
pub async fn send_files(
    writer: &mut FramedWriter, wanted: &[(String, String)], service: &FileSyncService,
    emit: &EventEmitter, progress_event: &str, transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let mut skipped = 0usize;

    for (comic_name, chapter) in wanted {
        let resolved = service.resolve_local_file(comic_name, chapter).await?;

        let Some((path, size, checksum, file_name)) = resolved else {
            tracing::warn!(
                comic_name = %comic_name,
                chapter = %chapter,
                "[FileSync] requested file no longer exists locally — sending unavailable header"
            );
            skipped += 1;
            write_json(
                writer,
                &FileHeader {
                    comic_name: comic_name.clone(),
                    chapter: chapter.clone(),
                    file_name: String::new(),
                    size: 0,
                    checksum: None,
                    blob_hash: None,
                },
            )
            .await?;
            continue;
        };

        let bytes = tokio::fs::read(&path).await.map_err(RpcError::from)?;
        let blob_hash = transfer.publish(bytes).await?;

        write_json(
            writer,
            &FileHeader { comic_name: comic_name.clone(), chapter: chapter.clone(), file_name, size, checksum, blob_hash: Some(blob_hash) },
        )
        .await?;

        (emit)(progress_event, format!("{} - {}", comic_name, chapter));
    }

    Ok(skipped)
}

/// Recebe, em sequência, `expected_count` arquivos: cada `FileHeader` traz um `blob_hash` em
/// vez de bytes crus no stream — busca via `ChapterTransfer::fetch_reader` (que já verifica
/// integridade via BLAKE3 antes de devolver o leitor), grava num arquivo temporário e move pro
/// destino final depois de conferir o SHA-256 legado contra o anunciado no header (continuidade
/// do histórico/telemetria existente, redundante com a verificação do blob store mas barato o
/// bastante pra manter).
///
/// Descarta headers "indisponíveis" (`size: 0`), falhas de busca do blob ou mismatches de
/// checksum sem abortar a sessão inteira — mas cada um desses casos agora loga via `tracing`
/// (antes só emitia um evento pro frontend, invisível no log do backend) e é contado no
/// retorno (`usize` = quantos capítulos de `expected_count` NÃO foram persistidos). Bug
/// reportado em 22/08/2026: um sync de 2 capítulos persistiu só 1, sem nenhum erro visível no
/// log — porque essas 4 falhas de item nunca logavam nada, só emitiam evento pro frontend, e a
/// sessão inteira ainda era reportada como "completa" pro chamador (`ComicSyncOutbound`/
/// `FileSyncOutbound`) mesmo com itens faltando.
///
/// `#[allow(too_many_arguments)]`: dívida técnica pré-existente (já eram 8 parâmetros antes
/// desta mudança) — agrupar em uma struct de contexto é um refactor maior, fora do escopo do
/// bug que motivou esta função a mudar.
#[allow(clippy::too_many_arguments)]
pub async fn receive_files(
    reader: &mut FramedReader, expected_count: usize, service: &FileSyncService, emit: &EventEmitter,
    progress_event: &str, error_event: &str, peer: &PeerIdentity, transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let mut skipped = 0usize;

    for _ in 0..expected_count {
        let header: FileHeader = read_json(reader).await?;

        tracing::debug!(
            comic_name = %header.comic_name,
            chapter = %header.chapter,
            file_name = %header.file_name,
            "[FileSync] header recebido pelo fio"
        );

        if header.size == 0 {
            tracing::warn!(
                comic_name = %header.comic_name,
                chapter = %header.chapter,
                "[FileSync] peer reported chapter as unavailable (size=0) — skipping"
            );
            skipped += 1;
            continue;
        }

        let Some(blob_hash) = header.blob_hash.as_deref() else {
            tracing::warn!(
                comic_name = %header.comic_name,
                chapter = %header.chapter,
                "[FileSync] header missing blob_hash — skipping"
            );
            skipped += 1;
            (emit)(error_event, format!("missing blob hash: {} - {}", header.comic_name, header.chapter));
            continue;
        };

        let mut blob_reader = match transfer.fetch_reader(blob_hash, peer).await {
            Ok(reader) => reader,
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    chapter = %header.chapter,
                    error = %err,
                    "[FileSync] failed to fetch chapter blob — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    format!("blob fetch failed: {} - {}: {err}", header.comic_name, header.chapter),
                );
                continue;
            },
        };

        let mut bytes = Vec::new();
        blob_reader.read_to_end(&mut bytes).await.map_err(|err| P2pError::StreamFailed(err.to_string()))?;

        let computed_checksum = {
            use sha2::{Digest, Sha256};
            format!("{:x}", Sha256::digest(&bytes))
        };

        if let Some(expected) = &header.checksum {
            if expected != &computed_checksum {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    chapter = %header.chapter,
                    expected = %expected,
                    computed = %computed_checksum,
                    "[FileSync] checksum mismatch — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    format!("checksum mismatch: {} - {}", header.comic_name, header.chapter),
                );
                continue;
            }
        }

        // Resolve a pasta do quadrinho ANTES de gravar o arquivo temporário — grava já no
        // destino final (mesma pasta que `persist_received_chapter` vai usar pro rename),
        // sem uma pasta de staging à parte que sobrava visível na biblioteca do usuário.
        let comic_dir = match service.resolve_comic_dir(&header.comic_name).await {
            Ok(comic) => std::path::PathBuf::from(comic.path),
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    chapter = %header.chapter,
                    error = %err,
                    "[FileSync] failed to resolve comic directory — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    format!("failed to resolve comic directory: {} - {}", header.comic_name, header.chapter),
                );
                continue;
            },
        };
        let temp_path = comic_dir.join(format!(".incoming-{}.tmp", rand::random::<u64>()));
        tokio::fs::write(&temp_path, &bytes).await.map_err(RpcError::from)?;

        service
            .persist_received_chapter(
                &header.comic_name,
                &header.chapter,
                &header.file_name,
                &temp_path,
                computed_checksum,
            )
            .await?;

        (emit)(progress_event, format!("{} - {}", header.comic_name, header.chapter));
    }

    Ok(skipped)
}

/// Envia, em sequência, os itens extra (capa/banner/`ComicInfo.xml`) de `wanted_extras` — mesma
/// lógica de `send_files`, só que via `FileSyncService::resolve_local_extra`/`FileExtraHeader`
/// em vez de capítulo.
pub async fn send_extras(
    writer: &mut FramedWriter, wanted_extras: &[(String, String)], service: &FileSyncService,
    emit: &EventEmitter, progress_event: &str, transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let mut skipped = 0usize;

    for (comic_name, kind) in wanted_extras {
        let resolved = service.resolve_local_extra(comic_name, kind).await?;

        let Some((path, size, checksum, file_name)) = resolved else {
            tracing::warn!(
                comic_name = %comic_name,
                kind = %kind,
                "[FileSync] requested extra no longer exists locally — sending unavailable header"
            );
            skipped += 1;
            write_json(
                writer,
                &FileExtraHeader {
                    comic_name: comic_name.clone(),
                    kind: kind.clone(),
                    file_name: String::new(),
                    size: 0,
                    checksum: None,
                    blob_hash: None,
                },
            )
            .await?;
            continue;
        };

        let bytes = tokio::fs::read(&path).await.map_err(RpcError::from)?;
        let blob_hash = transfer.publish(bytes).await?;

        write_json(
            writer,
            &FileExtraHeader {
                comic_name: comic_name.clone(),
                kind: kind.clone(),
                file_name,
                size,
                checksum,
                blob_hash: Some(blob_hash),
            },
        )
        .await?;

        (emit)(progress_event, format!("{} - {}", comic_name, kind));
    }

    Ok(skipped)
}

/// Recebe, em sequência, `expected_count` itens extra: mesma mecânica de `receive_files`
/// (busca via blob store, grava em arquivo temporário, verifica checksum), mas persistindo via
/// `FileSyncService::persist_received_extra` em vez de `persist_received_chapter` — não vai pra
/// `chapter_archive`, vai pra `comic_directory` (capa/banner) ou vira `ComicInfo.xml` na raiz.
///
/// Ao terminar de persistir um `ComicInfo.xml`, dispara `MetadataService::sync_comic_comic_info`
/// (mesmo método usado pelo botão manual "sincronizar via ComicInfo") pra já deixar o
/// título/sinopse/autor atualizados no banco sem ação do usuário — melhor esforço: uma falha
/// aqui loga um aviso e conta como item não sincronizado nas estatísticas, mas não derruba a
/// sessão (o arquivo em si já foi transferido com sucesso).
#[allow(clippy::too_many_arguments)]
pub async fn receive_extras(
    reader: &mut FramedReader, expected_count: usize, service: &FileSyncService,
    metadata_service: &Arc<MetadataService>, emit: &EventEmitter, progress_event: &str, error_event: &str,
    peer: &PeerIdentity, transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let mut skipped = 0usize;

    for _ in 0..expected_count {
        let header: FileExtraHeader = read_json(reader).await?;

        if header.size == 0 {
            tracing::warn!(
                comic_name = %header.comic_name,
                kind = %header.kind,
                "[FileSync] peer reported extra as unavailable (size=0) — skipping"
            );
            skipped += 1;
            continue;
        }

        let Some(blob_hash) = header.blob_hash.as_deref() else {
            tracing::warn!(
                comic_name = %header.comic_name,
                kind = %header.kind,
                "[FileSync] extra header missing blob_hash — skipping"
            );
            skipped += 1;
            (emit)(error_event, format!("missing blob hash: {} - {}", header.comic_name, header.kind));
            continue;
        };

        let mut blob_reader = match transfer.fetch_reader(blob_hash, peer).await {
            Ok(reader) => reader,
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    error = %err,
                    "[FileSync] failed to fetch extra blob — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    format!("blob fetch failed: {} - {}: {err}", header.comic_name, header.kind),
                );
                continue;
            },
        };

        let mut bytes = Vec::new();
        blob_reader.read_to_end(&mut bytes).await.map_err(|err| P2pError::StreamFailed(err.to_string()))?;

        let computed_checksum = {
            use sha2::{Digest, Sha256};
            format!("{:x}", Sha256::digest(&bytes))
        };

        if let Some(expected) = &header.checksum {
            if expected != &computed_checksum {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    expected = %expected,
                    computed = %computed_checksum,
                    "[FileSync] extra checksum mismatch — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    format!("checksum mismatch: {} - {}", header.comic_name, header.kind),
                );
                continue;
            }
        }

        let comic_dir = match service.resolve_comic_dir(&header.comic_name).await {
            Ok(comic) => std::path::PathBuf::from(comic.path),
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    error = %err,
                    "[FileSync] failed to resolve comic directory for extra — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    format!("failed to resolve comic directory: {} - {}", header.comic_name, header.kind),
                );
                continue;
            },
        };
        let temp_path = comic_dir.join(format!(".incoming-{}.tmp", rand::random::<u64>()));
        tokio::fs::write(&temp_path, &bytes).await.map_err(RpcError::from)?;

        let persisted = service
            .persist_received_extra(&header.comic_name, &header.kind, &header.file_name, &temp_path)
            .await;

        let comic = match persisted {
            Ok(comic) => comic,
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    error = %err,
                    "[FileSync] failed to persist received extra — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    format!("failed to persist extra: {} - {}", header.comic_name, header.kind),
                );
                continue;
            },
        };

        if header.kind == EXTRA_KIND_COMIC_INFO {
            if let Err(err) = metadata_service.sync_comic_comic_info(comic.id).await {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    error = %err,
                    "[FileSync] ComicInfo.xml recebido, mas reprocessamento de metadata falhou"
                );
            }
        }

        (emit)(progress_event, format!("{} - {}", header.comic_name, header.kind));
    }

    Ok(skipped)
}

/// Duplo em memória de `ChapterTransfer` pra testes que não têm um `Endpoint` iroh de verdade
/// (`BlobChapterTransfer`, usado em produção, precisa disso). `shared_pair()` devolve dois
/// handles apontando pro mesmo `HashMap` interno — simula os dois lados publicando/buscando no
/// mesmo "store de rede" content-addressed.
#[cfg(test)]
pub struct InMemoryChapterTransfer {
    blobs: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

#[cfg(test)]
impl InMemoryChapterTransfer {
    pub fn shared_pair() -> (Arc<dyn ChapterTransfer>, Arc<dyn ChapterTransfer>) {
        let shared =
            Arc::new(InMemoryChapterTransfer { blobs: std::sync::Mutex::new(std::collections::HashMap::new()) });
        (Arc::clone(&shared) as Arc<dyn ChapterTransfer>, shared as Arc<dyn ChapterTransfer>)
    }
}

#[cfg(test)]
#[async_trait]
impl ChapterTransfer for InMemoryChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        use sha2::{Digest, Sha256};
        let hash = format!("{:x}", Sha256::digest(&bytes));
        self.blobs.lock().unwrap().insert(hash.clone(), bytes);
        Ok(hash)
    }

    async fn fetch_reader(
        &self, blob_hash: &str, _peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let bytes = self
            .blobs
            .lock()
            .unwrap()
            .get(blob_hash)
            .cloned()
            .ok_or_else(|| P2pError::StreamFailed(format!("unknown blob hash in test transfer: {blob_hash}")))?;

        // `tokio::io::duplex` já implementa `AsyncRead`/`AsyncWrite` — mais simples que
        // implementar um `poll_read` manual só pra teste. Escreve tudo e fecha (EOF) antes de
        // devolver a ponta de leitura.
        use tokio::io::AsyncWriteExt;
        let (mut writer, reader) = tokio::io::duplex(bytes.len().max(1));
        tokio::spawn(async move {
            let _ = writer.write_all(&bytes).await;
            let _ = writer.shutdown().await;
        });
        Ok(Box::new(reader))
    }
}
