use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity, protocol::EventEmitter};
use futures::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::model::{HistoryManifest, HistorySyncStats};
use crate::{callbacks::HistorySyncProvider, protocol::ffi_blocking::run_blocking};

const MANIFEST_READ_TIMEOUT: Duration = Duration::from_secs(15);

pub(super) type Recv = FramedRead<Box<dyn AsyncRead + Send + Unpin>, LengthDelimitedCodec>;
pub(super) type Writer = FramedWrite<Box<dyn AsyncWrite + Send + Unpin>, LengthDelimitedCodec>;

pub(super) async fn write_manifest(
    send: &mut Writer,
    manifest: &HistoryManifest,
) -> Result<(), P2pError> {
    let bytes = serde_json::to_vec(manifest).map_err(|err| {
        P2pError::StreamFailed(format!("failed to encode history manifest: {err}"))
    })?;
    send.send(bytes.into())
        .await
        .map_err(|err| P2pError::StreamFailed(format!("failed to write history manifest: {err}")))
}

pub(super) async fn read_manifest(recv: &mut Recv) -> Result<HistoryManifest, P2pError> {
    let frame = tokio::time::timeout(MANIFEST_READ_TIMEOUT, recv.next())
        .await
        .map_err(|_| P2pError::StreamFailed("timed out reading history manifest".into()))?
        .ok_or_else(|| P2pError::StreamFailed("stream closed before history manifest".into()))?
        .map_err(|err| P2pError::StreamFailed(format!("failed to read history manifest: {err}")))?;

    serde_json::from_slice(&frame)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode history manifest: {err}")))
}

/// Monta o manifesto local a partir do `provider` (Room, via Kotlin). As duas chamadas FFI
/// cabem no mesmo `spawn_blocking` — uma única troca de thread por sessão, não duas.
pub(super) async fn build_local_manifest(
    provider: &Arc<dyn HistorySyncProvider>,
) -> Result<HistoryManifest, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || HistoryManifest {
        reading_progress: provider.get_reading_progress(),
        chapters_read: provider.get_chapters_read(),
    })
    .await
}

/// Progresso de leitura do peer que vence o local — last-write-wins por `updated_at`.
/// Puro, sem FFI: só decide o que deveria ser aplicado, não aplica.
fn progress_entries_to_apply(
    local: &HistoryManifest,
    peer_progress: Vec<crate::callbacks::FfiReadingProgressEntry>,
) -> Vec<crate::callbacks::FfiReadingProgressEntry> {
    let local_by_key: HashMap<(&str, &str), &crate::callbacks::FfiReadingProgressEntry> = local
        .reading_progress
        .iter()
        .map(|entry| {
            (
                (entry.comic_name.as_str(), entry.chapter_sort.as_str()),
                entry,
            )
        })
        .collect();

    peer_progress
        .into_iter()
        .filter(|peer_entry| {
            let key = (
                peer_entry.comic_name.as_str(),
                peer_entry.chapter_sort.as_str(),
            );
            match local_by_key.get(&key) {
                Some(local_entry) => peer_entry.updated_at > local_entry.updated_at,
                None => true,
            }
        })
        .collect()
}

/// Capítulos lidos que o peer tem e o local ainda não — união idempotente, sem noção de
/// "mais recente" (diferente de progresso). Puro, sem FFI.
fn chapters_read_entries_to_apply(
    local: &HistoryManifest,
    peer_chapters_read: Vec<crate::callbacks::FfiChapterReadEntry>,
) -> Vec<crate::callbacks::FfiChapterReadEntry> {
    let local_keys: HashSet<(&str, &str)> = local
        .chapters_read
        .iter()
        .map(|entry| (entry.comic_name.as_str(), entry.chapter_sort.as_str()))
        .collect();

    peer_chapters_read
        .into_iter()
        .filter(|peer_entry| {
            let key = (
                peer_entry.comic_name.as_str(),
                peer_entry.chapter_sort.as_str(),
            );
            !local_keys.contains(&key)
        })
        .collect()
}

/// Aplica as entradas já decididas via `provider` — única parte que toca FFI, roda inteira
/// num único `spawn_blocking` (não um por entrada).
async fn apply_entries(
    provider: &Arc<dyn HistorySyncProvider>,
    progress: Vec<crate::callbacks::FfiReadingProgressEntry>,
    chapters_read: Vec<crate::callbacks::FfiChapterReadEntry>,
) -> Result<HistorySyncStats, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || {
        let mut stats = HistorySyncStats::default();

        for entry in progress {
            if provider.apply_reading_progress(entry) {
                stats.progress_applied += 1;
            } else {
                stats.progress_skipped += 1;
            }
        }

        for entry in chapters_read {
            if provider.apply_chapter_read(entry) {
                stats.chapters_read_applied += 1;
            } else {
                stats.chapters_read_skipped += 1;
            }
        }

        stats
    })
    .await
}

/// Calcula o diff do manifesto do peer contra o manifesto local e aplica via `provider`:
/// last-write-wins (por `updated_at`) pra progresso de leitura, união idempotente pra
/// capítulos lidos. Cada lado roda essa função com seu próprio manifesto local — não há
/// mais nenhuma troca de rede depois disso.
pub(super) async fn diff_and_apply(
    local: &HistoryManifest,
    peer: HistoryManifest,
    provider: &Arc<dyn HistorySyncProvider>,
) -> Result<HistorySyncStats, P2pError> {
    let progress = progress_entries_to_apply(local, peer.reading_progress);
    let chapters_read = chapters_read_entries_to_apply(local, peer.chapters_read);
    apply_entries(provider, progress, chapters_read).await
}

/// Executa a troca de manifesto de um lado da conexão e aplica o diff.
///
/// `outbound_role` decide a ordem de leitura/escrita (regra 4): o lado outbound sempre
/// escreve primeiro em cada etapa, o inbound sempre lê primeiro — nunca "quem chegar primeiro
/// escreve". Compartilhada pelos dois handlers já que a única diferença entre eles é essa ordem.
pub(super) async fn run_exchange(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn HistorySyncProvider>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    emit("sync:history:started", started_payload(peer));

    let local_manifest = build_local_manifest(provider).await?;

    let peer_manifest = if outbound_role {
        write_manifest(&mut writer, &local_manifest).await?;
        read_manifest(&mut reader).await?
    } else {
        let peer_manifest = read_manifest(&mut reader).await?;
        write_manifest(&mut writer, &local_manifest).await?;
        peer_manifest
    };

    let stats = diff_and_apply(&local_manifest, peer_manifest, provider).await?;
    emit("sync:history:complete", complete_payload(peer, &stats));

    Ok(())
}

fn started_payload(peer: &PeerIdentity) -> String {
    serde_json::json!({ "peerId": peer.id }).to_string()
}

fn complete_payload(peer: &PeerIdentity, stats: &HistorySyncStats) -> String {
    serde_json::json!({
        "peerId": peer.id,
        "progressApplied": stats.progress_applied,
        "progressSkipped": stats.progress_skipped,
        "chaptersReadApplied": stats.chapters_read_applied,
        "chaptersReadSkipped": stats.chapters_read_skipped,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU32, Ordering},
        time::Duration,
    };

    use super::*;
    use crate::callbacks::{FfiChapterReadEntry, FfiReadingProgressEntry};

    struct SlowProvider {
        sleep_ms: u64,
    }

    impl HistorySyncProvider for SlowProvider {
        fn get_reading_progress(&self) -> Vec<FfiReadingProgressEntry> {
            if self.sleep_ms > 0 {
                std::thread::sleep(Duration::from_millis(self.sleep_ms));
            }
            vec![]
        }
        fn get_chapters_read(&self) -> Vec<FfiChapterReadEntry> {
            vec![]
        }
        fn apply_reading_progress(&self, _entry: FfiReadingProgressEntry) -> bool {
            true
        }
        fn apply_chapter_read(&self, _entry: FfiChapterReadEntry) -> bool {
            true
        }
    }

    /// Mesma prova que em `protocol/files/exchange.rs`: uma chamada FFI lenta não deve travar
    /// outra task async concorrente no mesmo runtime.
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn build_local_manifest_does_not_block_runtime() {
        let provider: Arc<dyn HistorySyncProvider> = Arc::new(SlowProvider { sleep_ms: 150 });

        let ticks = Arc::new(AtomicU32::new(0));
        let ticks_clone = Arc::clone(&ticks);
        let ticker = tokio::spawn(async move {
            for _ in 0..12 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                ticks_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        let manifest = build_local_manifest(&provider).await.unwrap();
        let _ = tokio::join!(ticker);

        assert!(manifest.reading_progress.is_empty());
        assert!(
            ticks.load(Ordering::SeqCst) >= 6,
            "runtime ficou travado durante get_reading_progress"
        );
    }
}
