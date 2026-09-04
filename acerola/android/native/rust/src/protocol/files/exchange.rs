use std::{collections::HashMap, sync::Arc, time::Duration};

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity, protocol::EventEmitter};
use futures::SinkExt;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::{
    model::{
        build_manifest, merge_extras, ComicSyncScope, FileComicInfo, FileExtraHeader, FileHeader,
        FileManifest, FileSyncStats, FileWantList, SessionBusy, SyncDirection, EXTRA_KIND_BANNER,
        EXTRA_KIND_COMIC_INFO, EXTRA_KIND_COVER,
    },
    transfer::ChapterTransfer,
};
use crate::{callbacks::FileSyncProvider, protocol::ffi_blocking::run_blocking};

/// Valor do campo `error` em `SessionBusy` — mesmo literal usado pelo Desktop
/// (`file_handler.rs`), é o que os dois lados usam para reconhecer essa mensagem específica
/// em vez de um `FileManifest` normal.
pub(super) const SESSION_BUSY_TAG: &str = "busy";

const FRAME_READ_TIMEOUT: Duration = Duration::from_secs(30);
const CHUNK_SIZE: usize = 64 * 1024;
const PROGRESS_EVERY_BYTES: u64 = 1024 * 1024;

pub(super) type Recv = FramedRead<Box<dyn AsyncRead + Send + Unpin>, LengthDelimitedCodec>;
pub(super) type Writer = FramedWrite<Box<dyn AsyncWrite + Send + Unpin>, LengthDelimitedCodec>;

/// Escreve uma mensagem de controle como frame JSON solto (sem tag de enum) — mesmo
/// formato que o Desktop usa em `infra/sync/framing.rs::write_json`.
async fn write_json<T: Serialize>(send: &mut Writer, value: &T) -> Result<(), P2pError> {
    let bytes = serde_json::to_vec(value).map_err(|err| {
        P2pError::StreamFailed(format!("failed to encode file sync message: {err}"))
    })?;
    send.send(bytes.into())
        .await
        .map_err(|err| P2pError::StreamFailed(format!("failed to write file sync message: {err}")))
}

async fn read_json<T: DeserializeOwned>(recv: &mut Recv) -> Result<T, P2pError> {
    let frame = tokio::time::timeout(FRAME_READ_TIMEOUT, recv.next())
        .await
        .map_err(|_| P2pError::StreamFailed("timed out reading file sync message".into()))?
        .ok_or_else(|| P2pError::StreamFailed("stream closed before file sync message".into()))?
        .map_err(|err| {
            P2pError::StreamFailed(format!("failed to read file sync message: {err}"))
        })?;

    serde_json::from_slice(&frame)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode file sync message: {err}")))
}

/// Escreve a mensagem de rejeição de sessão diretamente no stream, no lugar do manifesto —
/// usado quando `FileSyncSessionGuard::try_acquire` já barrou a sessão em `run_and_report`,
/// antes de `run_exchange` ser chamado. Recebe `send` cru (não um `Writer` já montado) porque
/// esse é exatamente o estado em que `run_and_report` está no momento da rejeição: o stream
/// ainda não foi envolvido em nenhum framing.
pub(super) async fn write_session_busy(
    send: Box<dyn AsyncWrite + Send + Unpin>,
    reason: &str,
) -> Result<(), P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    write_json(
        &mut writer,
        &SessionBusy {
            error: SESSION_BUSY_TAG.to_string(),
            reason: reason.to_string(),
        },
    )
    .await
}

/// Lê a primeira mensagem do peer podendo ser tanto o `FileManifest` normal quanto uma
/// rejeição `SessionBusy` — a única forma de saber qual é sem uma tag de enum no wire (regra
/// já estabelecida pelo resto do protocolo) é espiar o JSON bruto antes de decidir o tipo
/// concreto. Só os dois pontos de leitura inicial em `exchange_manifests` usam isso: depois da
/// primeira mensagem, uma sessão aceita nunca mais manda `SessionBusy`.
async fn read_manifest_or_busy(reader: &mut Recv) -> Result<FileManifest, P2pError> {
    let value: serde_json::Value = read_json(reader).await?;

    let is_busy = value.get("error").and_then(|tag| tag.as_str()) == Some(SESSION_BUSY_TAG);
    if is_busy {
        let reason = value
            .get("reason")
            .and_then(|r| r.as_str())
            .unwrap_or("peer is busy");
        return Err(P2pError::StreamFailed(format!("peer busy: {reason}")));
    }

    serde_json::from_value(value)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode file sync message: {err}")))
}

// --- Fronteira FFI (Kotlin): uma função por método de `FileSyncProvider`, cada uma só
// clonando o `Arc` e delegando pro pool de blocking threads via `run_blocking` — nunca
// direto na worker thread do runtime async. Existem só pra `send_one_chapter`/
// `receive_one_chapter` não repetirem o bloco `{ Arc::clone + run_blocking }` a cada chamada.

async fn open_chapter_for_read(
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: String,
    chapter: String,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.open_chapter_for_read(comic_name, chapter)).await
}

async fn read_chapter_chunk(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<Vec<u8>, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.read_chapter_chunk(handle, CHUNK_SIZE as u32)).await
}

async fn close_read_handle(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<(), P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.close_read_handle(handle)).await
}

async fn begin_chapter_write(
    provider: &Arc<dyn FileSyncProvider>,
    header: &FileHeader,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    let (comic_name, chapter, file_name, checksum, size) = (
        header.comic_name.clone(),
        header.chapter.clone(),
        header.file_name.clone(),
        header.checksum.clone().unwrap_or_default(),
        header.size,
    );

    // Log estratégico 3/4: último ponto do lado Rust antes do checksum atravessar a FFI
    // de volta pro Kotlin (`beginChapterWrite`), onde fica guardado em `WriteHandle` até
    // `finalizeChapterWrite` comparar contra o hash recém-calculado do arquivo recebido.
    // Se este valor bater com o log 2/4 (mesmo comic/chapter) mas o Kotlin ainda assim
    // reportar mismatch, a corrupção está inteiramente do lado Kotlin (escrita/releitura),
    // fora do alcance do que dá pra instrumentar aqui.
    tracing::debug!(
        comic = %comic_name,
        chapter = %chapter,
        checksum = %checksum,
        "sync-files: passando expected_checksum pra FFI Rust->Kotlin (begin_chapter_write)",
    );

    run_blocking(move || {
        provider.begin_chapter_write(comic_name, chapter, file_name, checksum, size)
    })
    .await
}

async fn write_chapter_chunk(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    chunk: Vec<u8>,
) -> Result<bool, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.write_chapter_chunk(handle, chunk)).await
}

async fn finalize_chapter_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<bool, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.finalize_chapter_write(handle)).await
}

async fn abort_chapter_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<(), P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.abort_chapter_write(handle)).await
}

async fn build_local_manifest(
    provider: &Arc<dyn FileSyncProvider>,
) -> Result<FileManifest, P2pError> {
    let entries = run_blocking({
        let provider = Arc::clone(provider);
        move || provider.get_file_manifest()
    })
    .await?;
    let mut manifest = build_manifest(entries);

    let extras = run_blocking({
        let provider = Arc::clone(provider);
        move || provider.get_extras_manifest()
    })
    .await?;
    merge_extras(&mut manifest, extras);

    Ok(manifest)
}

async fn open_extra_for_read(
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: String,
    kind: String,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.open_extra_for_read(comic_name, kind)).await
}

async fn begin_extra_write(
    provider: &Arc<dyn FileSyncProvider>,
    header: &FileExtraHeader,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    let (comic_name, kind, file_name, checksum, size) = (
        header.comic_name.clone(),
        header.kind.clone(),
        header.file_name.clone(),
        header.checksum.clone().unwrap_or_default(),
        header.size,
    );
    run_blocking(move || provider.begin_extra_write(comic_name, kind, file_name, checksum, size))
        .await
}

async fn finalize_extra_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<bool, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.finalize_extra_write(handle)).await
}

async fn abort_extra_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<(), P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.abort_extra_write(handle)).await
}

/// Retorna `(comic_name, chapter)` que `peer` oferece e que `local` não tem (ausente, ou
/// presente com checksum diferente — tratado como "faltando" também, sem tentar reconciliar).
fn missing_from(local: &FileManifest, peer: &FileManifest) -> Vec<(String, String)> {
    let local_checksums: HashMap<(&str, &str), Option<&str>> = local
        .comics
        .iter()
        .flat_map(|comic| {
            comic.chapters.iter().map(move |chapter| {
                (
                    (comic.comic_name.as_str(), chapter.chapter.as_str()),
                    chapter.checksum.as_deref(),
                )
            })
        })
        .collect();

    peer.comics
        .iter()
        .flat_map(|comic| {
            let local_checksums = &local_checksums;
            comic.chapters.iter().filter_map(move |chapter| {
                let key = (comic.comic_name.as_str(), chapter.chapter.as_str());
                let missing = match local_checksums.get(&key) {
                    Some(checksum) => *checksum != chapter.checksum.as_deref(),
                    None => true,
                };
                missing.then(|| (comic.comic_name.clone(), chapter.chapter.clone()))
            })
        })
        .collect()
}

fn manifest_index(
    manifest: &FileManifest,
) -> HashMap<(&str, &str), &super::model::FileChapterInfo> {
    manifest
        .comics
        .iter()
        .flat_map(|comic: &FileComicInfo| {
            comic.chapters.iter().map(move |chapter| {
                (
                    (comic.comic_name.as_str(), chapter.chapter.as_str()),
                    chapter,
                )
            })
        })
        .collect()
}

/// Retorna `(comic_name, kind)` de itens extra (capa/banner/`ComicInfo.xml`) que `peer` oferece
/// e que `local` não tem (ausente, ou com checksum diferente) — mesma lógica de `missing_from`,
/// aplicada aos três campos opcionais de `FileComicInfo` em vez da lista de capítulos.
fn missing_extras_from(local: &FileManifest, peer: &FileManifest) -> Vec<(String, String)> {
    let local_by_key = extras_index(local);

    peer.comics
        .iter()
        .flat_map(|comic| {
            let local_by_key = &local_by_key;
            EXTRA_KINDS.iter().filter_map(move |kind| {
                let peer_extra = extra_for_kind(comic, kind)?;
                let key = (comic.comic_name.as_str(), *kind);
                let missing = match local_by_key.get(&key) {
                    Some(local_extra) => local_extra.checksum != peer_extra.checksum,
                    None => true,
                };
                missing.then(|| (comic.comic_name.clone(), kind.to_string()))
            })
        })
        .collect()
}

const EXTRA_KINDS: [&str; 3] = [EXTRA_KIND_COVER, EXTRA_KIND_BANNER, EXTRA_KIND_COMIC_INFO];

fn extra_for_kind<'a>(
    comic: &'a FileComicInfo,
    kind: &str,
) -> Option<&'a super::model::FileExtraInfo> {
    match kind {
        EXTRA_KIND_COVER => comic.cover.as_ref(),
        EXTRA_KIND_BANNER => comic.banner.as_ref(),
        EXTRA_KIND_COMIC_INFO => comic.comic_info.as_ref(),
        _ => None,
    }
}

fn extras_index(manifest: &FileManifest) -> HashMap<(&str, &str), &super::model::FileExtraInfo> {
    manifest
        .comics
        .iter()
        .flat_map(|comic| {
            EXTRA_KINDS.iter().filter_map(move |kind| {
                extra_for_kind(comic, kind).map(|extra| ((comic.comic_name.as_str(), *kind), extra))
            })
        })
        .collect()
}

async fn write_missing_header(
    writer: &mut Writer,
    comic_name: &str,
    chapter: &str,
) -> Result<(), P2pError> {
    write_json(
        writer,
        &FileHeader {
            comic_name: comic_name.to_string(),
            chapter: chapter.to_string(),
            file_name: String::new(),
            size: 0,
            checksum: None,
            blob_hash: None,
        },
    )
    .await
}

async fn write_missing_extra_header(
    writer: &mut Writer,
    comic_name: &str,
    kind: &str,
) -> Result<(), P2pError> {
    write_json(
        writer,
        &FileExtraHeader {
            comic_name: comic_name.to_string(),
            kind: kind.to_string(),
            file_name: String::new(),
            size: 0,
            checksum: None,
            blob_hash: None,
        },
    )
    .await
}

/// Envia um único item extra pedido — mesma mecânica de `send_one_chapter`
/// (abre via `open_extra_for_read`, publica no blob store via `publish_chapter_to_blob`, que já
/// é agnóstico a chapter vs. extra, fecha via `close_read_handle` reaproveitado), com
/// `FileExtraHeader` no lugar de `FileHeader`.
async fn send_one_extra(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: &str,
    kind: &str,
    entry: &super::model::FileExtraInfo,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<bool, P2pError> {
    let handle = open_extra_for_read(provider, comic_name.to_string(), kind.to_string()).await?;
    if handle < 0 {
        write_missing_extra_header(writer, comic_name, kind).await?;
        return Ok(false);
    }

    let publish_result = publish_chapter_to_blob(
        provider, handle, entry.size, emit, peer, comic_name, kind, transfer,
    )
    .await;
    close_read_handle(provider, handle).await?;
    let blob_hash = publish_result?;

    write_json(
        writer,
        &FileExtraHeader {
            comic_name: comic_name.to_string(),
            kind: kind.to_string(),
            file_name: entry.file_name.clone(),
            size: entry.size,
            checksum: Some(entry.checksum.clone()),
            blob_hash: Some(blob_hash),
        },
    )
    .await?;

    Ok(true)
}

/// Envia os itens extra pedidos — mesma lógica de `send_files`, olhando `cover`/`banner`/
/// `comic_info` de `local_manifest` em vez da lista de capítulos.
async fn send_extras(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: &FileManifest,
    requested: &[(String, String)],
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let by_comic: HashMap<&str, &FileComicInfo> = local_manifest
        .comics
        .iter()
        .map(|comic| (comic.comic_name.as_str(), comic))
        .collect();

    for (comic_name, kind) in requested {
        let entry = by_comic
            .get(comic_name.as_str())
            .and_then(|comic| extra_for_kind(comic, kind));
        let Some(entry) = entry else {
            write_missing_extra_header(writer, comic_name, kind).await?;
            continue;
        };

        if send_one_extra(
            writer, provider, comic_name, kind, entry, emit, peer, transfer,
        )
        .await?
        {
            stats.sent_count += 1;
        }
    }

    Ok(())
}

/// Busca o blob de um item extra e grava chunk a chunk — mesma mecânica de
/// `receive_chapter_from_blob`, com `FileExtraHeader` no lugar de `FileHeader`.
async fn receive_extra_from_blob(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    header: &FileExtraHeader,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<bool, P2pError> {
    let Some(blob_hash) = header.blob_hash.as_deref() else {
        return Ok(true);
    };

    let mut blob_reader = match transfer.fetch_reader(blob_hash, peer).await {
        Ok(reader) => reader,
        Err(err) => {
            tracing::warn!(
                comic = %header.comic_name,
                kind = %header.kind,
                blob_hash,
                error = %err,
                "sync-files: falha ao buscar blob de extra do peer",
            );
            return Ok(true);
        }
    };

    let mut received_bytes: u64 = 0;
    let mut bytes_since_progress: u64 = 0;
    let mut write_failed = handle < 0;
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n = blob_reader.read(&mut buf).await.map_err(|err| {
            P2pError::StreamFailed(format!("failed to read fetched extra blob: {err}"))
        })?;
        if n == 0 {
            break;
        }

        received_bytes += n as u64;
        bytes_since_progress += n as u64;

        if !write_failed && !write_chapter_chunk(provider, handle, buf[..n].to_vec()).await? {
            write_failed = true;
        }

        if bytes_since_progress >= PROGRESS_EVERY_BYTES {
            bytes_since_progress = 0;
            emit_progress(
                emit,
                peer,
                &header.comic_name,
                &header.kind,
                received_bytes,
                header.size,
            );
        }
    }

    Ok(write_failed)
}

fn extra_complete_payload(peer: &PeerIdentity, header: &FileExtraHeader) -> String {
    serde_json::json!({ "peerId": peer.id, "comicName": header.comic_name, "kind": header.kind })
        .to_string()
}

fn extra_failed_payload(peer: &PeerIdentity, header: &FileExtraHeader) -> String {
    serde_json::json!({
        "peerId": peer.id,
        "comicName": header.comic_name,
        "kind": header.kind,
        "reason": "checksum or I/O failure",
    })
    .to_string()
}

async fn receive_one_extra(
    provider: &Arc<dyn FileSyncProvider>,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    header: &FileExtraHeader,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let handle = begin_extra_write(provider, header).await?;
    let write_failed =
        receive_extra_from_blob(provider, handle, header, emit, peer, transfer).await?;

    let succeeded = if !write_failed && handle >= 0 {
        finalize_extra_write(provider, handle).await?
    } else {
        false
    };

    if succeeded {
        stats.received_count += 1;
        emit(
            "sync:files:extra_complete",
            extra_complete_payload(peer, header),
        );
    } else {
        if handle >= 0 {
            abort_extra_write(provider, handle).await?;
        }
        stats.failed_count += 1;
        emit(
            "sync:files:extra_failed",
            extra_failed_payload(peer, header),
        );
    }

    Ok(())
}

/// Recebe exatamente `expected_count` itens extra — mesma lógica de `receive_files`, com
/// `FileExtraHeader` no lugar de `FileHeader`.
async fn receive_extras(
    reader: &mut Recv,
    expected_count: usize,
    provider: &Arc<dyn FileSyncProvider>,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    for _ in 0..expected_count {
        let header: FileExtraHeader = read_json(reader).await?;

        if header.size == 0 {
            continue;
        }

        receive_one_extra(provider, emit, peer, &header, stats, transfer).await?;
    }

    Ok(())
}

/// Lê o capítulo já aberto (`handle` válido) um chunk de cada vez e publica os bytes
/// acumulados no blob store local (`ChapterTransfer::publish`) — devolve o hash de blob (hex)
/// que vai no `FileHeader`. A leitura em si continua chunk a chunk (nunca destrava o loop de
/// progresso existente), só o destino dos bytes muda: iam pro wire, agora vão pro `put()`.
async fn publish_chapter_to_blob(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    total_size: u64,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    comic_name: &str,
    chapter: &str,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<String, P2pError> {
    let mut bytes = Vec::new();
    let mut read_bytes_total: u64 = 0;
    let mut bytes_since_progress: u64 = 0;

    while read_bytes_total < total_size {
        let chunk = read_chapter_chunk(provider, handle).await?;
        if chunk.is_empty() {
            break;
        }

        read_bytes_total += chunk.len() as u64;
        bytes_since_progress += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);

        if bytes_since_progress >= PROGRESS_EVERY_BYTES {
            bytes_since_progress = 0;
            emit_progress(
                emit,
                peer,
                comic_name,
                chapter,
                read_bytes_total,
                total_size,
            );
        }
    }

    transfer.publish(bytes).await
}

/// Envia um único capítulo pedido: abre, publica os bytes no blob store local, escreve o
/// header com o hash resultante, fecha. `Ok(false)` sinaliza "não pôde atender" (item sumiu ou
/// handle inválido) — header vazio já escrito, sem publicação.
async fn send_one_chapter(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: &str,
    chapter: &str,
    entry: &super::model::FileChapterInfo,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<bool, P2pError> {
    let handle =
        open_chapter_for_read(provider, comic_name.to_string(), chapter.to_string()).await?;
    if handle < 0 {
        write_missing_header(writer, comic_name, chapter).await?;
        return Ok(false);
    }

    let publish_result = publish_chapter_to_blob(
        provider, handle, entry.size, emit, peer, comic_name, chapter, transfer,
    )
    .await;
    close_read_handle(provider, handle).await?;
    let blob_hash = publish_result?;

    tracing::debug!(
        comic = %comic_name,
        chapter = %chapter,
        blob_hash = %blob_hash,
        size = entry.size,
        "sync-files: publicado no blob store local, enviando header",
    );

    write_json(
        writer,
        &FileHeader {
            comic_name: comic_name.to_string(),
            chapter: chapter.to_string(),
            file_name: entry.file_name.clone(),
            size: entry.size,
            checksum: entry.checksum.clone(),
            blob_hash: Some(blob_hash),
        },
    )
    .await?;

    Ok(true)
}

/// Envia os capítulos pedidos, na ordem pedida. Um item que não existe mais localmente
/// (corrida com deleção concorrente) manda um `FileHeader` com `size: 0` em vez de ser
/// simplesmente pulado — o outro lado conta exatamente `requested.len()` headers, então
/// a contagem precisa ficar determinística mesmo quando um item não pode ser atendido.
async fn send_files(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: &FileManifest,
    requested: &[(String, String)],
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let by_key = manifest_index(local_manifest);

    for (comic_name, chapter) in requested {
        let Some(entry) = by_key.get(&(comic_name.as_str(), chapter.as_str())) else {
            write_missing_header(writer, comic_name, chapter).await?;
            continue;
        };

        if send_one_chapter(
            writer, provider, comic_name, chapter, entry, emit, peer, transfer,
        )
        .await?
        {
            stats.sent_count += 1;
        }
    }

    Ok(())
}

/// Recebe exatamente `expected_count` capítulos (a mesma contagem que este lado pediu em
/// `FileWantList`), cada um como um `FileHeader` seguido de `ChapterTransfer::fetch_reader` +
/// escrita chunk a chunk. `size: 0` sinaliza "peer não tem mais esse capítulo" — sem blob a
/// buscar.
async fn receive_files(
    reader: &mut Recv,
    expected_count: usize,
    provider: &Arc<dyn FileSyncProvider>,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    for _ in 0..expected_count {
        let header: FileHeader = read_json(reader).await?;

        if header.size == 0 {
            continue;
        }

        tracing::debug!(
            comic = %header.comic_name,
            chapter = %header.chapter,
            blob_hash = %header.blob_hash.as_deref().unwrap_or("<none>"),
            size = header.size,
            "sync-files: header recebido do wire",
        );

        receive_one_chapter(provider, emit, peer, &header, stats, transfer).await?;
    }

    Ok(())
}

/// Busca o blob referenciado por `header.blob_hash` no peer (`ChapterTransfer::fetch_reader`) e
/// grava o resultado chunk a chunk via `FileSyncProvider::write_chapter_chunk`, preservando os
/// mesmos eventos de progresso de antes — só a origem dos bytes mudou (era o wire cru, agora é
/// o `AsyncRead` devolvido pelo blob store após `fetch`). Retorna `true` se a busca ou alguma
/// escrita falhou (`handle` inválido conta como falha imediata).
async fn receive_chapter_from_blob(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    header: &FileHeader,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<bool, P2pError> {
    let Some(blob_hash) = header.blob_hash.as_deref() else {
        return Ok(true);
    };

    let mut blob_reader = match transfer.fetch_reader(blob_hash, peer).await {
        Ok(reader) => reader,
        Err(err) => {
            tracing::warn!(
                comic = %header.comic_name,
                chapter = %header.chapter,
                blob_hash,
                error = %err,
                "sync-files: falha ao buscar blob do peer",
            );
            return Ok(true);
        }
    };

    let mut received_bytes: u64 = 0;
    let mut bytes_since_progress: u64 = 0;
    let mut write_failed = handle < 0;
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n = blob_reader
            .read(&mut buf)
            .await
            .map_err(|err| P2pError::StreamFailed(format!("failed to read fetched blob: {err}")))?;
        if n == 0 {
            break;
        }

        received_bytes += n as u64;
        bytes_since_progress += n as u64;

        if !write_failed && !write_chapter_chunk(provider, handle, buf[..n].to_vec()).await? {
            write_failed = true;
        }

        if bytes_since_progress >= PROGRESS_EVERY_BYTES {
            bytes_since_progress = 0;
            emit_progress(
                emit,
                peer,
                &header.comic_name,
                &header.chapter,
                received_bytes,
                header.size,
            );
        }
    }

    Ok(write_failed)
}

fn chapter_complete_payload(peer: &PeerIdentity, header: &FileHeader) -> String {
    serde_json::json!({ "peerId": peer.id, "comicName": header.comic_name, "chapter": header.chapter }).to_string()
}

fn chapter_failed_payload(peer: &PeerIdentity, header: &FileHeader) -> String {
    serde_json::json!({
        "peerId": peer.id,
        "comicName": header.comic_name,
        "chapter": header.chapter,
        "reason": "checksum or I/O failure",
    })
    .to_string()
}

async fn receive_one_chapter(
    provider: &Arc<dyn FileSyncProvider>,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    header: &FileHeader,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let handle = begin_chapter_write(provider, header).await?;
    let write_failed =
        receive_chapter_from_blob(provider, handle, header, emit, peer, transfer).await?;

    let succeeded = if !write_failed && handle >= 0 {
        finalize_chapter_write(provider, handle).await?
    } else {
        false
    };

    // Log estratégico 4/4: resultado final da verificação de checksum, que acontece
    // inteiramente dentro do Kotlin (`finalizeChapterWrite` relê o arquivo escrito e
    // compara com `expectedChecksum`). O Rust só vê o booleano de volta — não o hash
    // real calculado do lado Kotlin — então este log mostra o `expected_checksum` que
    // tínhamos (log 3/4) ao lado do resultado, mas não prova sozinho onde a diferença
    // está; serve pra confirmar QUAL capítulo falhou e correlacionar com os logs 1-3.
    tracing::debug!(
        comic = %header.comic_name,
        chapter = %header.chapter,
        expected_checksum = %header.checksum.as_deref().unwrap_or("<none>"),
        write_failed,
        succeeded,
        "sync-files: resultado da finalização (checksum verificado do lado Kotlin)",
    );

    if succeeded {
        stats.received_count += 1;
        emit(
            "sync:files:chapter_complete",
            chapter_complete_payload(peer, header),
        );
    } else {
        if handle >= 0 {
            abort_chapter_write(provider, handle).await?;
        }
        stats.failed_count += 1;
        emit(
            "sync:files:chapter_failed",
            chapter_failed_payload(peer, header),
        );
    }

    Ok(())
}

fn emit_progress(
    emit: &EventEmitter,
    peer: &PeerIdentity,
    comic_name: &str,
    chapter: &str,
    transferred: u64,
    total: u64,
) {
    emit(
        "sync:files:progress",
        serde_json::json!({
            "peerId": peer.id,
            "comicName": comic_name,
            "chapter": chapter,
            "bytesTransferred": transferred,
            "totalBytes": total,
        })
        .to_string(),
    );
}

fn manifest_exchanged_payload(
    peer: &PeerIdentity,
    missing_count: usize,
    offering_count: usize,
) -> String {
    serde_json::json!({ "peerId": peer.id, "missingCount": missing_count, "offeringCount": offering_count }).to_string()
}

/// Fase 1: troca de manifesto (regra 4) — outbound escreve primeiro, inbound lê primeiro.
async fn exchange_manifests(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    local_manifest: &FileManifest,
) -> Result<FileManifest, P2pError> {
    if outbound_role {
        write_json(writer, local_manifest).await?;
        read_manifest_or_busy(reader).await
    } else {
        let peer_manifest = read_manifest_or_busy(reader).await?;
        write_json(writer, local_manifest).await?;
        Ok(peer_manifest)
    }
}

/// Fase 2: troca de want-list, mesma ordem espelhada da fase 1. Retorna
/// `(wanted, wanted_extras)` do peer.
async fn exchange_want_lists(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    wanted_locally: &[(String, String)],
    wanted_extras_locally: &[(String, String)],
) -> Result<(Vec<(String, String)>, Vec<(String, String)>), P2pError> {
    let mine = FileWantList {
        wanted: wanted_locally.to_vec(),
        wanted_extras: wanted_extras_locally.to_vec(),
    };

    if outbound_role {
        write_json(writer, &mine).await?;
        let theirs: FileWantList = read_json(reader).await?;
        Ok((theirs.wanted, theirs.wanted_extras))
    } else {
        let theirs: FileWantList = read_json(reader).await?;
        write_json(writer, &mine).await?;
        Ok((theirs.wanted, theirs.wanted_extras))
    }
}

/// Fase 3: cada lado envia o que o outro pediu e recebe o que pediu, na ordem espelhada —
/// outbound sempre escreve primeiro em cada etapa, inbound sempre lê primeiro. Capítulos
/// primeiro, depois extras, nos dois sentidos — mesma ordem usada pelo Desktop
/// (`infra/sync/protocol/file_handler.rs`), já que os dois lados leem/escrevem o wire na mesma
/// sequência sem tag de enum que distinga qual mensagem é qual.
#[allow(clippy::too_many_arguments)]
async fn transfer_files(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: &FileManifest,
    wanted_locally: &[(String, String)],
    wanted_extras_locally: &[(String, String)],
    their_wanted: &[(String, String)],
    their_wanted_extras: &[(String, String)],
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    if outbound_role {
        receive_files(
            reader,
            wanted_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        receive_extras(
            reader,
            wanted_extras_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        send_files(
            writer,
            provider,
            local_manifest,
            their_wanted,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        send_extras(
            writer,
            provider,
            local_manifest,
            their_wanted_extras,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
    } else {
        send_files(
            writer,
            provider,
            local_manifest,
            their_wanted,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        send_extras(
            writer,
            provider,
            local_manifest,
            their_wanted_extras,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        receive_files(
            reader,
            wanted_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        receive_extras(
            reader,
            wanted_extras_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
    }

    Ok(())
}

/// Cauda comum às fases 1-3 (troca de manifesto -> want-list -> transferência), compartilhada
/// entre `run_exchange` (biblioteca inteira) e `run_exchange_scoped` (um único quadrinho,
/// protocolo `acerola/sync-comic/1`) — a diferença entre os dois é como `local_manifest` foi
/// montado/filtrado antes de chegar aqui, mais `force_empty_wanted`: `run_exchange` sempre passa
/// `false` (sync de biblioteca é sempre bidirecional); `run_exchange_scoped` passa `true` quando
/// este lado é o "sender" da direção explícita escolhida (ver `SyncDirection` e
/// `run_exchange_scoped`) — nesse caso a want-list local é zerada mesmo que o diff mostre algo
/// faltando, porque este lado só deve entregar, nunca pedir de volta.
async fn run_exchange_with_manifest(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: FileManifest,
    writer: &mut Writer,
    reader: &mut Recv,
    transfer: &Arc<dyn ChapterTransfer>,
    force_empty_wanted: bool,
) -> Result<FileSyncStats, P2pError> {
    let mut stats = FileSyncStats::default();

    let peer_manifest = exchange_manifests(outbound_role, writer, reader, &local_manifest).await?;

    let (wanted_locally, wanted_extras_locally) = if force_empty_wanted {
        (Vec::new(), Vec::new())
    } else {
        (
            missing_from(&local_manifest, &peer_manifest),
            missing_extras_from(&local_manifest, &peer_manifest),
        )
    };
    let wanted_by_peer = missing_from(&peer_manifest, &local_manifest);
    emit(
        "sync:files:manifest_exchanged",
        manifest_exchanged_payload(peer, wanted_locally.len(), wanted_by_peer.len()),
    );

    let (their_wanted, their_wanted_extras) = exchange_want_lists(
        outbound_role,
        writer,
        reader,
        &wanted_locally,
        &wanted_extras_locally,
    )
    .await?;

    transfer_files(
        outbound_role,
        writer,
        reader,
        provider,
        &local_manifest,
        &wanted_locally,
        &wanted_extras_locally,
        &their_wanted,
        &their_wanted_extras,
        emit,
        peer,
        &mut stats,
        transfer,
    )
    .await?;

    Ok(stats)
}

/// Executa a sessão inteira do protocolo `acerola/sync-files/1` pra um dos dois lados.
/// Schema e sequência de wire idênticos ao Desktop (`infra/sync/protocol/file_handler.rs`):
/// (1) troca de manifesto; (2) troca de want-list; (3) transferência de arquivos.
pub(super) async fn run_exchange(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    transfer: &Arc<dyn ChapterTransfer>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<FileSyncStats, P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    let local_manifest = build_local_manifest(provider).await?;
    run_exchange_with_manifest(
        outbound_role,
        peer,
        emit,
        provider,
        local_manifest,
        &mut writer,
        &mut reader,
        transfer,
        false,
    )
    .await
}

/// Fase 0 exclusiva do `acerola/sync-comic/1`: outbound manda `ComicSyncScope` (o quadrinho e a
/// direção escolhidos pelo usuário, vindo de `P2PNode::sync_comic`), inbound só lê — nenhum dos
/// dois lados decide o alvo localmente, só quem discou sabe. Retorna `(comic_name, direction)`
/// acordados: `comic_name` filtra os manifestos, `direction` decide qual lado é o "sender" (ver
/// `run_exchange_scoped`).
async fn exchange_comic_scope(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    comic_name_outbound: Option<(&str, SyncDirection)>,
) -> Result<(String, SyncDirection), P2pError> {
    if outbound_role {
        let (comic_name, direction) = comic_name_outbound.ok_or_else(|| {
            P2pError::StreamFailed(
                "sync-comic: missing target comic_name for outbound session".into(),
            )
        })?;
        let comic_name = comic_name.to_string();
        write_json(
            writer,
            &ComicSyncScope {
                comic_name: comic_name.clone(),
                direction,
            },
        )
        .await?;
        Ok((comic_name, direction))
    } else {
        let scope: ComicSyncScope = read_json(reader).await?;
        Ok((scope.comic_name, scope.direction))
    }
}

/// Mantém só as entradas do quadrinho alvo — o resto do manifesto nunca é construído em vão
/// (a filtragem acontece depois de `get_file_manifest()`, que já é sobre a biblioteca inteira;
/// não há como pedir pro `FileSyncProvider` só um quadrinho sem mudar a trait FFI).
fn filter_manifest_to_comic(manifest: FileManifest, comic_name: &str) -> FileManifest {
    FileManifest {
        comics: manifest
            .comics
            .into_iter()
            .filter(|c| c.comic_name == comic_name)
            .collect(),
    }
}

/// Executa a sessão inteira do protocolo `acerola/sync-comic/1` (sincronização de um único
/// quadrinho) pra um dos dois lados. Mesma sequência de wire do `sync-files` normal, só que
/// precedida pela fase 0 (`ComicSyncScope`) e com `local_manifest` filtrado ao quadrinho
/// acordado. A direção explícita (`SyncDirection`, ponto de vista sempre do outbound) decide
/// qual lado é o "sender" nesta sessão — o papel de cada lado é sempre o oposto do outro:
/// `outbound_role=true` + `Push` ou `outbound_role=false` + `Pull` → este lado é sender (want
/// list forçada vazia em `run_exchange_with_manifest`); os outros dois casos → receiver (pede
/// normalmente). Isso cobre push e pull com o mesmo pipeline de diff/transferência, sem mudar a
/// troca de manifestos em si.
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_exchange_scoped(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    transfer: &Arc<dyn ChapterTransfer>,
    comic_name_outbound: Option<(String, SyncDirection)>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<FileSyncStats, P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    let comic_name_outbound_ref = comic_name_outbound
        .as_ref()
        .map(|(name, direction)| (name.as_str(), *direction));
    let (comic_name, direction) = exchange_comic_scope(
        outbound_role,
        &mut writer,
        &mut reader,
        comic_name_outbound_ref,
    )
    .await?;
    let is_sender = (direction == SyncDirection::Push) == outbound_role;

    let local_manifest = build_local_manifest(provider).await?;
    let local_manifest = filter_manifest_to_comic(local_manifest, &comic_name);

    run_exchange_with_manifest(
        outbound_role,
        peer,
        emit,
        provider,
        local_manifest,
        &mut writer,
        &mut reader,
        transfer,
        is_sender,
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicI64, AtomicU32, AtomicUsize, Ordering},
            Mutex,
        },
        time::Duration,
    };

    use super::*;
    use crate::callbacks::FfiFileManifestEntry;
    use crate::protocol::files::transfer::InMemoryChapterTransfer;

    /// Um par de transports em memória que compartilham o mesmo hash->bytes — simula os dois
    /// lados de uma conexão real publicando/buscando no mesmo "store de rede" content-addressed,
    /// sem precisar de um `Endpoint` iroh de verdade (só `BlobChapterTransfer`, usado em
    /// produção, precisa disso).
    fn test_transfer_pair() -> (Arc<dyn ChapterTransfer>, Arc<dyn ChapterTransfer>) {
        InMemoryChapterTransfer::shared_pair()
    }

    /// Provider que nunca é usado pra transferir nada — todos os métodos de dados retornam
    /// "vazio"/"-1"/"false". Só `get_file_manifest` faz algo (definido por closure em cada
    /// mock específico via composição).
    struct CountingSlowProvider {
        manifest_calls: Arc<AtomicUsize>,
        sleep_ms: u64,
    }

    impl FileSyncProvider for CountingSlowProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            self.manifest_calls.fetch_add(1, Ordering::SeqCst);
            if self.sleep_ms > 0 {
                std::thread::sleep(Duration::from_millis(self.sleep_ms));
            }
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

    fn make_peer(id: &str) -> PeerIdentity {
        PeerIdentity {
            id: id.to_string(),
            device_id: None,
        }
    }

    fn no_op_emitter() -> EventEmitter {
        Arc::new(|_event: &str, _payload: String| {})
    }

    /// Prova, no nível de `build_local_manifest` (já usando `run_blocking`), que uma chamada
    /// FFI lenta não trava outras tasks async concorrentes rodando no mesmo runtime — a
    /// assinatura exata do bug relatado (sync-files travava o app inteiro por minutos).
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn build_local_manifest_does_not_block_runtime() {
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingSlowProvider {
            manifest_calls: Arc::new(AtomicUsize::new(0)),
            sleep_ms: 200,
        });

        let ticks = Arc::new(AtomicU32::new(0));
        let ticks_clone = Arc::clone(&ticks);
        let ticker = tokio::spawn(async move {
            for _ in 0..15 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                ticks_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        let manifest = build_local_manifest(&provider).await.unwrap();
        let _ = tokio::join!(ticker);

        assert!(manifest.comics.is_empty());
        assert!(
            ticks.load(Ordering::SeqCst) >= 8,
            "runtime ficou travado durante get_file_manifest"
        );
    }

    /// Prova que `run_exchange` monta o manifesto local exatamente 1 vez por sessão — antes
    /// do fix, `send_files` reconstruía do zero internamente, dobrando o trabalho.
    #[tokio::test]
    async fn run_exchange_builds_local_manifest_exactly_once_per_side() {
        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let outbound_calls = Arc::new(AtomicUsize::new(0));
        let inbound_calls = Arc::new(AtomicUsize::new(0));
        let outbound_provider: Arc<dyn FileSyncProvider> = Arc::new(CountingSlowProvider {
            manifest_calls: Arc::clone(&outbound_calls),
            sleep_ms: 0,
        });
        let inbound_provider: Arc<dyn FileSyncProvider> = Arc::new(CountingSlowProvider {
            manifest_calls: Arc::clone(&inbound_calls),
            sleep_ms: 0,
        });

        let peer = make_peer("peer-x");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_provider,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_provider,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.unwrap();
        inbound_result.unwrap();

        assert_eq!(outbound_calls.load(Ordering::SeqCst), 1);
        assert_eq!(inbound_calls.load(Ordering::SeqCst), 1);
    }

    // --- Testes de integridade de transferência ---
    //
    // Os mocks acima (`CountingSlowProvider`) nunca tocam bytes reais — `read_chapter_chunk`
    // sempre devolve vazio, `write_chapter_chunk` sempre falha. Isso deixa uma lacuna: nenhum
    // teste existente exercita o caminho real de envio -> framing -> recepção -> verificação
    // de checksum, que é exatamente onde suspeitamos um bug de "checksum mismatch" em produção
    // (já provado, com hash real de um arquivo real, que o checksum ORIGEM está correto — o
    // problema está em algum ponto entre a origem e a verificação final).
    //
    // `InMemoryFileSyncProvider` guarda bytes de verdade nos dois sentidos e usa blake3 (só
    // como checksum de teste — não é o SHA-256 real de produção, que é calculado no Kotlin)
    // pra provar que o que sai de um lado chega intacto no outro, incluindo o próprio valor
    // do checksum atravessando `FileHeader` pelo wire. Mesmo espírito dos testes de stress do
    // `acerola-p2p` (`transport_stress.rs`: payload real + hash real + comparação), só que
    // exercitando o protocolo `sync-files` inteiro (manifesto -> want-list -> transferência)
    // em vez do transporte cru.

    /// `read_chapter_chunk`/`write_chapter_chunk`/`close_read_handle` são reaproveitados sem
    /// mudança tanto pra capítulo quanto pra extra (ver doc da trait em `callbacks.rs`) — então
    /// os dois tipos de handle têm que viver no MESMO espaço de handles/mapas aqui, exatamente
    /// como a implementação Kotlin real (`FileSyncProviderImpl`) vai precisar fazer. Só o que
    /// muda entre os dois é o que `WriteTarget` guarda e pra qual lista (`finalized` vs.
    /// `finalized_extras`) o resultado vai.
    enum WriteTarget {
        Chapter { comic_name: String, chapter: String },
        Extra { comic_name: String, kind: String },
    }

    struct WriteState {
        target: WriteTarget,
        expected_checksum: String,
        buffer: Vec<u8>,
    }

    /// Um capítulo que já foi finalizado do lado que recebeu — guarda tanto o checksum
    /// declarado pelo remetente (`expected_checksum`, o que veio no `FileHeader`) quanto o
    /// que este provider recalculou dos bytes efetivamente recebidos (`actual_checksum`),
    /// além dos bytes crus — pra poder comparar tanto o hash quanto o conteúdo byte-a-byte.
    struct FinalizedChapter {
        comic_name: String,
        chapter: String,
        expected_checksum: String,
        actual_checksum: String,
        bytes: Vec<u8>,
    }

    /// Mesmo par (checksum declarado x recalculado) de `FinalizedChapter`, pra um item extra
    /// (capa/banner/`ComicInfo.xml`) em vez de capítulo.
    struct FinalizedExtra {
        comic_name: String,
        kind: String,
        expected_checksum: String,
        actual_checksum: String,
        bytes: Vec<u8>,
    }

    struct InMemoryFileSyncProvider {
        readable: HashMap<(String, String), (String, Vec<u8>)>,
        read_handles: Mutex<HashMap<i64, (Vec<u8>, usize)>>,
        write_handles: Mutex<HashMap<i64, WriteState>>,
        finalized: Mutex<Vec<FinalizedChapter>>,
        readable_extras: HashMap<(String, String), (String, Vec<u8>)>,
        finalized_extras: Mutex<Vec<FinalizedExtra>>,
        next_handle: AtomicI64,
    }

    impl InMemoryFileSyncProvider {
        fn with_readable(entries: Vec<(String, String, String, Vec<u8>)>) -> Self {
            Self::with_readable_and_extras(entries, vec![])
        }

        /// `extras`: `(comic_name, kind, file_name, bytes)`.
        fn with_readable_and_extras(
            entries: Vec<(String, String, String, Vec<u8>)>,
            extras: Vec<(String, String, String, Vec<u8>)>,
        ) -> Self {
            let readable = entries
                .into_iter()
                .map(|(comic_name, chapter, file_name, bytes)| {
                    ((comic_name, chapter), (file_name, bytes))
                })
                .collect();
            let readable_extras = extras
                .into_iter()
                .map(|(comic_name, kind, file_name, bytes)| {
                    ((comic_name, kind), (file_name, bytes))
                })
                .collect();
            Self {
                readable,
                read_handles: Mutex::new(HashMap::new()),
                write_handles: Mutex::new(HashMap::new()),
                finalized: Mutex::new(Vec::new()),
                readable_extras,
                finalized_extras: Mutex::new(Vec::new()),
                next_handle: AtomicI64::new(1),
            }
        }
    }

    impl FileSyncProvider for InMemoryFileSyncProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            self.readable
                .iter()
                .map(
                    |((comic_name, chapter), (file_name, bytes))| FfiFileManifestEntry {
                        comic_name: comic_name.clone(),
                        chapter: chapter.clone(),
                        file_name: file_name.clone(),
                        checksum: blake3::hash(bytes).to_hex().to_string(),
                        size_bytes: bytes.len() as u64,
                    },
                )
                .collect()
        }

        fn get_library_summary(&self) -> Vec<crate::callbacks::FfiComicSummaryEntry> {
            vec![]
        }

        fn open_chapter_for_read(&self, comic_name: String, chapter: String) -> i64 {
            let Some((_, bytes)) = self.readable.get(&(comic_name, chapter)) else {
                return -1;
            };
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.read_handles
                .lock()
                .unwrap()
                .insert(handle, (bytes.clone(), 0));
            handle
        }

        fn read_chapter_chunk(&self, handle: i64, chunk_size: u32) -> Vec<u8> {
            let mut handles = self.read_handles.lock().unwrap();
            let Some((bytes, pos)) = handles.get_mut(&handle) else {
                return vec![];
            };
            let end = (*pos + chunk_size as usize).min(bytes.len());
            if *pos >= end {
                return vec![];
            }
            let chunk = bytes[*pos..end].to_vec();
            *pos = end;
            chunk
        }

        fn close_read_handle(&self, handle: i64) {
            self.read_handles.lock().unwrap().remove(&handle);
        }

        fn begin_chapter_write(
            &self,
            comic_name: String,
            chapter: String,
            _file_name: String,
            expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.write_handles.lock().unwrap().insert(
                handle,
                WriteState {
                    target: WriteTarget::Chapter {
                        comic_name,
                        chapter,
                    },
                    expected_checksum,
                    buffer: Vec::new(),
                },
            );
            handle
        }

        fn write_chapter_chunk(&self, handle: i64, bytes: Vec<u8>) -> bool {
            let mut handles = self.write_handles.lock().unwrap();
            let Some(state) = handles.get_mut(&handle) else {
                return false;
            };
            state.buffer.extend_from_slice(&bytes);
            true
        }

        fn finalize_chapter_write(&self, handle: i64) -> bool {
            let Some(state) = self.write_handles.lock().unwrap().remove(&handle) else {
                return false;
            };
            let WriteTarget::Chapter {
                comic_name,
                chapter,
            } = state.target
            else {
                return false;
            };
            let actual_checksum = blake3::hash(&state.buffer).to_hex().to_string();
            let succeeded = actual_checksum == state.expected_checksum;
            self.finalized.lock().unwrap().push(FinalizedChapter {
                comic_name,
                chapter,
                expected_checksum: state.expected_checksum,
                actual_checksum,
                bytes: state.buffer,
            });
            succeeded
        }

        fn abort_chapter_write(&self, handle: i64) {
            self.write_handles.lock().unwrap().remove(&handle);
        }

        fn get_extras_manifest(&self) -> Vec<crate::callbacks::FfiExtraManifestEntry> {
            self.readable_extras
                .iter()
                .map(|((comic_name, kind), (file_name, bytes))| {
                    crate::callbacks::FfiExtraManifestEntry {
                        comic_name: comic_name.clone(),
                        kind: kind.clone(),
                        file_name: file_name.clone(),
                        checksum: blake3::hash(bytes).to_hex().to_string(),
                        size_bytes: bytes.len() as u64,
                    }
                })
                .collect()
        }

        // Reaproveita `read_handles` (o MESMO mapa usado por `open_chapter_for_read`) — não
        // existe `open_extra_for_read` separado no mapa, só na trait: o handle devolvido aqui
        // é consumido depois pela mesma `read_chapter_chunk`/`close_read_handle` de sempre.
        fn open_extra_for_read(&self, comic_name: String, kind: String) -> i64 {
            let Some((_, bytes)) = self.readable_extras.get(&(comic_name, kind)) else {
                return -1;
            };
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.read_handles
                .lock()
                .unwrap()
                .insert(handle, (bytes.clone(), 0));
            handle
        }

        // Mesma ideia de `open_extra_for_read`: insere em `write_handles`, o mapa compartilhado
        // com capítulo — `write_chapter_chunk` (reaproveitado) grava aqui sem saber a diferença.
        fn begin_extra_write(
            &self,
            comic_name: String,
            kind: String,
            _file_name: String,
            expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.write_handles.lock().unwrap().insert(
                handle,
                WriteState {
                    target: WriteTarget::Extra { comic_name, kind },
                    expected_checksum,
                    buffer: Vec::new(),
                },
            );
            handle
        }

        fn finalize_extra_write(&self, handle: i64) -> bool {
            let Some(state) = self.write_handles.lock().unwrap().remove(&handle) else {
                return false;
            };
            let WriteTarget::Extra { comic_name, kind } = state.target else {
                return false;
            };
            let actual_checksum = blake3::hash(&state.buffer).to_hex().to_string();
            let succeeded = actual_checksum == state.expected_checksum;
            self.finalized_extras.lock().unwrap().push(FinalizedExtra {
                comic_name,
                kind,
                expected_checksum: state.expected_checksum,
                actual_checksum,
                bytes: state.buffer,
            });
            succeeded
        }

        fn abort_extra_write(&self, handle: i64) {
            self.write_handles.lock().unwrap().remove(&handle);
        }
    }

    /// Gerador determinístico de payload — varia com `seed` pra capítulos diferentes não
    /// terem o mesmo conteúdo (evita mascarar bug de troca/mistura entre capítulos).
    fn make_payload(size: usize, seed: u8) -> Vec<u8> {
        (0..size)
            .map(|i| ((i as u64).wrapping_mul(31).wrapping_add(seed as u64) % 251) as u8)
            .collect()
    }

    /// Round-trip com bytes de verdade nos dois sentidos, tamanhos propositalmente NÃO
    /// múltiplos de `CHUNK_SIZE` (64KB) pra exercitar o último chunk parcial do loop em
    /// `stream_chapter_out`/`receive_chapter_bytes`. Verifica três coisas independentes:
    /// (1) o checksum declarado pelo remetente sobrevive ao wire; (2) o checksum recalculado
    /// do lado que recebe bate com o declarado; (3) os bytes recebidos são idênticos,
    /// byte-a-byte, ao que foi enviado — não só o hash batendo (hash batendo não prova
    /// ausência de bug se o mock também estivesse errado do mesmo jeito dos dois lados).
    #[tokio::test]
    async fn run_exchange_transfers_real_bytes_with_correct_checksum_round_trip() {
        let outbound_payload = make_payload(1_500_037, 7);
        let inbound_payload = make_payload(900_321, 42);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Comic A".to_string(),
            "Ch. 1".to_string(),
            "ch1.cbz".to_string(),
            outbound_payload.clone(),
        )]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Comic B".to_string(),
            "Ch. 1".to_string(),
            "ch1.cbz".to_string(),
            inbound_payload.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-x");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound deveria completar sem erro");
        inbound_result.expect("inbound deveria completar sem erro");

        let outbound_received = outbound_provider.finalized.lock().unwrap();
        assert_eq!(
            outbound_received.len(),
            1,
            "outbound deveria ter recebido exatamente 1 capítulo de B"
        );
        let received_from_b = &outbound_received[0];
        assert_eq!(received_from_b.comic_name, "Comic B");
        assert_eq!(
            received_from_b.expected_checksum, received_from_b.actual_checksum,
            "checksum declarado por B não bate com o recalculado por A após receber"
        );
        assert_eq!(
            received_from_b.bytes, inbound_payload,
            "bytes recebidos por A não batem byte-a-byte com o que B enviou"
        );

        let inbound_received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            inbound_received.len(),
            1,
            "inbound deveria ter recebido exatamente 1 capítulo de A"
        );
        let received_from_a = &inbound_received[0];
        assert_eq!(received_from_a.comic_name, "Comic A");
        assert_eq!(
            received_from_a.expected_checksum, received_from_a.actual_checksum,
            "checksum declarado por A não bate com o recalculado por B após receber"
        );
        assert_eq!(
            received_from_a.bytes, outbound_payload,
            "bytes recebidos por B não batem byte-a-byte com o que A enviou"
        );
    }

    /// Mesmo espírito do round-trip de capítulo acima, só que pros itens extra (capa/banner/
    /// `ComicInfo.xml`) — prova que `send_extras`/`receive_extras` (que reaproveitam os mesmos
    /// handles de leitura/escrita usados por capítulo) entregam os bytes certos pro comic certo,
    /// com o `kind` certo, sem se misturar com a transferência de capítulos que roda na mesma
    /// sessão.
    #[tokio::test]
    async fn run_exchange_transfers_extras_round_trip() {
        let cover_bytes = make_payload(2048, 11);
        let comic_info_bytes = b"<ComicInfo><Title>Berserk</Title></ComicInfo>".to_vec();

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable_and_extras(
            vec![],
            vec![(
                "Comic A".to_string(),
                EXTRA_KIND_COVER.to_string(),
                "cover.jpg".to_string(),
                cover_bytes.clone(),
            )],
        ));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable_and_extras(
            vec![],
            vec![(
                "Comic A".to_string(),
                EXTRA_KIND_COMIC_INFO.to_string(),
                "ComicInfo.xml".to_string(),
                comic_info_bytes.clone(),
            )],
        ));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-extras");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound deveria completar sem erro");
        inbound_result.expect("inbound deveria completar sem erro");

        let outbound_received = outbound_provider.finalized_extras.lock().unwrap();
        assert_eq!(
            outbound_received.len(),
            1,
            "outbound deveria ter recebido o ComicInfo.xml de B"
        );
        assert_eq!(outbound_received[0].comic_name, "Comic A");
        assert_eq!(outbound_received[0].kind, EXTRA_KIND_COMIC_INFO);
        assert_eq!(outbound_received[0].bytes, comic_info_bytes);
        assert_eq!(
            outbound_received[0].expected_checksum,
            outbound_received[0].actual_checksum
        );

        let inbound_received = inbound_provider.finalized_extras.lock().unwrap();
        assert_eq!(
            inbound_received.len(),
            1,
            "inbound deveria ter recebido a capa de A"
        );
        assert_eq!(inbound_received[0].comic_name, "Comic A");
        assert_eq!(inbound_received[0].kind, EXTRA_KIND_COVER);
        assert_eq!(inbound_received[0].bytes, cover_bytes);
        assert_eq!(
            inbound_received[0].expected_checksum,
            inbound_received[0].actual_checksum
        );
    }

    /// Stress: muitos capítulos de tamanhos variados transferidos SEQUENCIALMENTE numa única
    /// sessão (mesmo `send_files`/`receive_files` loop). Alvo específico: o bug real em
    /// produção falha capítulos DIFERENTES a cada execução, espalhados por vários quadrinhos
    /// — a assinatura clássica de um handle sendo reaproveitado incorretamente ou de estado
    /// (buffer, posição de leitura) vazando entre capítulos consecutivos, não de um bug
    /// determinístico de cálculo. Se esse tipo de vazamento existisse no lado Rust, um
    /// arquivo receberia bytes de outro e o checksum bateria errado só nesse — este teste
    /// pega isso comparando cada capítulo recebido, individualmente, contra seu payload
    /// original exato.
    #[tokio::test]
    async fn run_exchange_stress_many_sequential_chapters_preserve_checksum_integrity() {
        const CHAPTER_COUNT: usize = 40;

        let mut outbound_entries = Vec::with_capacity(CHAPTER_COUNT);
        for i in 0..CHAPTER_COUNT {
            let size = 10_000 + (i * 37_123) % 500_000;
            outbound_entries.push((
                "Stress Comic".to_string(),
                format!("Ch. {i}"),
                format!("ch{i}.cbz"),
                make_payload(size, i as u8),
            ));
        }

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(
            outbound_entries.clone(),
        ));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));

        let (client_io, server_io) = tokio::io::duplex(1024 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-stress");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound deveria completar sem erro");
        inbound_result.expect("inbound deveria completar sem erro");

        let received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            CHAPTER_COUNT,
            "nem todos os capítulos chegaram"
        );

        let by_chapter: HashMap<&str, &FinalizedChapter> =
            received.iter().map(|f| (f.chapter.as_str(), f)).collect();

        for (comic_name, chapter, _file_name, expected_bytes) in &outbound_entries {
            let got = by_chapter
                .get(chapter.as_str())
                .unwrap_or_else(|| panic!("capítulo {chapter} nunca chegou do lado que recebeu"));
            assert_eq!(
                &got.comic_name, comic_name,
                "capítulo {chapter}: comic_name divergiu"
            );
            assert_eq!(
                got.expected_checksum, got.actual_checksum,
                "capítulo {chapter}: checksum declarado não bate com o recalculado"
            );
            assert_eq!(
                &got.bytes, expected_bytes,
                "capítulo {chapter}: bytes recebidos divergem do original (possível vazamento de estado entre capítulos sequenciais)"
            );
        }
    }

    // --- Testes de `run_exchange_scoped` (protocolo `acerola/sync-comic/1`) ---
    //
    // Reaproveitam o mesmo `InMemoryFileSyncProvider` acima: a única coisa que muda é que o
    // outbound tem MAIS DE UM quadrinho no manifesto local, e o teste prova que só o
    // `comic_name` pedido atravessa — não a biblioteca inteira (o que já é coberto pelos testes
    // de `run_exchange` acima).

    /// Push: outbound tem dois quadrinhos, mas só escopa um. Prova que o `ComicSyncScope`
    /// (fase 0) realmente filtra o manifesto local antes da troca — sem a filtragem, o inbound
    /// receberia os dois quadrinhos, não só o pedido.
    #[tokio::test]
    async fn run_exchange_scoped_transfers_only_the_target_comic() {
        let payload_a = make_payload(50_000, 3);
        let payload_b = make_payload(70_000, 9);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![
            (
                "Comic A".to_string(),
                "Ch. 1".to_string(),
                "ch1.cbz".to_string(),
                payload_a.clone(),
            ),
            (
                "Comic B".to_string(),
                "Ch. 1".to_string(),
                "ch1.cbz".to_string(),
                payload_b.clone(),
            ),
        ]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-scoped");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Comic A".to_string(), SyncDirection::Push)),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound scoped exchange deveria completar sem erro");
        inbound_result.expect("inbound scoped exchange deveria completar sem erro");

        let received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "só o quadrinho escopado deveria ter sido transferido, não a biblioteca inteira"
        );
        assert_eq!(received[0].comic_name, "Comic A");
        assert_eq!(received[0].bytes, payload_a);
    }

    /// Pull: o quadrinho pedido só existe no peer (inbound), o outbound não tem nada dele
    /// localmente. Prova que a mesma troca simétrica que cobre push também cobre pull sem
    /// nenhum código extra — o manifesto local filtrado do outbound fica vazio, então
    /// `missing_from` já resolve "quero tudo que o peer tem desse quadrinho" sozinho.
    #[tokio::test]
    async fn run_exchange_scoped_pulls_a_comic_that_only_exists_on_the_peer() {
        let payload = make_payload(30_000, 5);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Comic Only On Peer".to_string(),
            "Ch. 1".to_string(),
            "ch1.cbz".to_string(),
            payload.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(128 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-pull");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Comic Only On Peer".to_string(), SyncDirection::Pull)),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("pull deveria completar sem erro");
        inbound_result.expect("lado que oferece deveria completar sem erro");

        let received = outbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "quem pediu deveria ter recebido o capítulo que só existia no peer"
        );
        assert_eq!(received[0].comic_name, "Comic Only On Peer");
        assert_eq!(received[0].bytes, payload);
    }

    /// Prova que `SyncDirection::Push` é respeitada mesmo quando o diff normal mostraria o
    /// outbound como "faltando" o capítulo do peer: outbound entrega "Cap A" (que só ele tem)
    /// mas NUNCA pede "Cap B" de volta, mesmo estando genuinamente sem esse capítulo. Ao
    /// contrário dos dois testes acima (onde um dos lados não tem NADA do quadrinho, então a
    /// direção nunca chega a ser testada de verdade — o diff já dava zero), aqui os dois lados
    /// têm conteúdo real e diferente do MESMO quadrinho.
    #[tokio::test]
    async fn run_exchange_scoped_push_never_requests_what_it_is_missing() {
        let payload_a = make_payload(40_000, 3);
        let payload_b = make_payload(60_000, 11);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap A".to_string(),
            "capA.cbz".to_string(),
            payload_a.clone(),
        )]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap B".to_string(),
            "capB.cbz".to_string(),
            payload_b.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-push");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Shared Comic".to_string(), SyncDirection::Push)),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("push não deveria falhar");
        inbound_result.expect("push não deveria falhar");

        assert!(
            outbound_provider.finalized.lock().unwrap().is_empty(),
            "outbound (sender) nunca deveria ter pedido 'Cap B' de volta"
        );
        let received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "inbound (receiver) deveria ter recebido 'Cap A'"
        );
        assert_eq!(received[0].comic_name, "Shared Comic");
        assert_eq!(received[0].chapter, "Cap A");
        assert_eq!(received[0].bytes, payload_a);
    }

    /// Espelho do teste acima pra `SyncDirection::Pull` — outbound puxa "Cap B" do peer sem
    /// nunca mandar "Cap A", mesmo o inbound estando genuinamente sem esse capítulo.
    #[tokio::test]
    async fn run_exchange_scoped_pull_never_sends_what_the_peer_is_missing() {
        let payload_a = make_payload(40_000, 3);
        let payload_b = make_payload(60_000, 11);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap A".to_string(),
            "capA.cbz".to_string(),
            payload_a.clone(),
        )]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap B".to_string(),
            "capB.cbz".to_string(),
            payload_b.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-pull");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Shared Comic".to_string(), SyncDirection::Pull)),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("pull não deveria falhar");
        inbound_result.expect("pull não deveria falhar");

        assert!(
            inbound_provider.finalized.lock().unwrap().is_empty(),
            "inbound (sender) nunca deveria ter pedido 'Cap A' de volta"
        );
        let received = outbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "outbound (receiver) deveria ter puxado 'Cap B'"
        );
        assert_eq!(received[0].comic_name, "Shared Comic");
        assert_eq!(received[0].chapter, "Cap B");
        assert_eq!(received[0].bytes, payload_b);
    }

    /// Sem `comic_name` do lado outbound (bug de wiring — `P2PNode::sync_comic` sempre grava o
    /// pending scope antes de conectar, então isso não deveria acontecer em produção), a sessão
    /// falha imediatamente em vez de travar esperando o peer mandar algo que nunca vai vir.
    #[tokio::test]
    async fn run_exchange_scoped_outbound_without_comic_name_fails_fast() {
        let provider: Arc<dyn FileSyncProvider> =
            Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));
        let (client_io, _server_io) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(client_io);
        let peer = make_peer("peer-no-scope");
        let emit = no_op_emitter();

        let result = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &provider,
            &test_transfer_pair().0,
            None,
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(
            result.is_err(),
            "outbound sem comic_name deveria falhar sem travar esperando o peer"
        );
    }
}
