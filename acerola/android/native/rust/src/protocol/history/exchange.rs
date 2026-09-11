use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity, protocol::EventEmitter};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::model::{HistoryManifest, HistorySyncStats};
use crate::{
    callbacks::{FfiReadingProgressEntry, HistorySyncProvider},
    protocol::ffi_blocking::run_blocking,
};

const MANIFEST_READ_TIMEOUT: Duration = Duration::from_secs(15);
const ENTRY_READ_TIMEOUT: Duration = Duration::from_secs(15);

pub(super) use crate::protocol::framing::{Recv, Writer};

/// Fina camada sobre `framing::write_json`/`read_json` — ver comentário equivalente em
/// `protocol/files/exchange.rs`.
pub(super) async fn write_manifest(
    send: &mut Writer,
    manifest: &HistoryManifest,
) -> Result<(), P2pError> {
    crate::protocol::framing::write_json(send, manifest).await
}

pub(super) async fn read_manifest(recv: &mut Recv) -> Result<HistoryManifest, P2pError> {
    crate::protocol::framing::read_json(recv, MANIFEST_READ_TIMEOUT).await
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

/// Executa o push unidirecional de UMA entrada de progresso (`acerola/sync-history-entry/1`).
/// `outbound_role`: o lado que inicia busca a entrada via `provider` (escopada por
/// `comic_name`, já resolvido do `PendingHistoryEntryScope` por quem chama) e escreve; o lado
/// que responde lê, aplica (se houver entrada — `None` quando o quadrinho nunca teve progresso
/// no outro lado) e confirma com um ack vazio antes de fechar, pra o outbound não considerar a
/// sessão concluída antes do inbound terminar de aplicar.
pub(super) async fn run_entry_exchange(
    outbound_role: bool,
    comic_name: Option<String>,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn HistorySyncProvider>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    emit("sync:history-entry:started", started_payload(peer));

    if outbound_role {
        let comic_name = comic_name.ok_or_else(|| {
            P2pError::StreamFailed("no pending history entry scope for this peer".into())
        })?;
        let provider_clone = Arc::clone(provider);
        let entry = run_blocking(move || provider_clone.get_reading_progress_for_comic(comic_name))
            .await?;

        crate::protocol::framing::write_json(&mut writer, &entry).await?;
        let _ack: serde_json::Value =
            crate::protocol::framing::read_json(&mut reader, ENTRY_READ_TIMEOUT).await?;
    } else {
        let entry: Option<FfiReadingProgressEntry> =
            crate::protocol::framing::read_json(&mut reader, ENTRY_READ_TIMEOUT).await?;

        if let Some(entry) = entry {
            let provider_clone = Arc::clone(provider);
            run_blocking(move || provider_clone.apply_reading_progress(entry)).await?;
        }

        crate::protocol::framing::write_json(&mut writer, &serde_json::json!({})).await?;
    }

    emit("sync:history-entry:complete", started_payload(peer));

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
        fn get_reading_progress_for_comic(&self, _comic_name: String) -> Option<FfiReadingProgressEntry> {
            None
        }
    }

    struct InMemoryProvider {
        progress: std::sync::Mutex<HashMap<String, FfiReadingProgressEntry>>,
    }

    impl InMemoryProvider {
        fn new() -> Self {
            Self { progress: std::sync::Mutex::new(HashMap::new()) }
        }

        fn with_entry(entry: FfiReadingProgressEntry) -> Self {
            let provider = Self::new();
            provider
                .progress
                .lock()
                .unwrap()
                .insert(entry.comic_name.clone(), entry);
            provider
        }
    }

    impl HistorySyncProvider for InMemoryProvider {
        fn get_reading_progress(&self) -> Vec<FfiReadingProgressEntry> {
            self.progress.lock().unwrap().values().cloned().collect()
        }
        fn get_chapters_read(&self) -> Vec<FfiChapterReadEntry> {
            vec![]
        }
        fn apply_reading_progress(&self, entry: FfiReadingProgressEntry) -> bool {
            self.progress.lock().unwrap().insert(entry.comic_name.clone(), entry);
            true
        }
        fn apply_chapter_read(&self, _entry: FfiChapterReadEntry) -> bool {
            true
        }
        fn get_reading_progress_for_comic(&self, comic_name: String) -> Option<FfiReadingProgressEntry> {
            self.progress.lock().unwrap().get(&comic_name).cloned()
        }
    }

    fn test_peer() -> PeerIdentity {
        PeerIdentity { id: "peer-a".to_string(), device_id: None }
    }

    fn noop_emitter() -> EventEmitter {
        Arc::new(|_, _| {})
    }

    /// O escopo (`comic_name`) só existe do lado outbound (vem do `PendingHistoryEntryScope`,
    /// resolvido por quem chama antes de invocar `run_entry_exchange`) — a entrada aplicada do
    /// lado inbound tem que ser exatamente a do quadrinho escopado, não a biblioteca inteira.
    #[tokio::test]
    async fn entry_exchange_pushes_only_the_scoped_comic() {
        let outbound_provider: Arc<dyn HistorySyncProvider> =
            Arc::new(InMemoryProvider::with_entry(FfiReadingProgressEntry {
                comic_name: "Berserk".to_string(),
                chapter_sort: "12".to_string(),
                last_page: 7,
                is_completed: false,
                updated_at: 5000,
            }));
        let inbound_provider: Arc<dyn HistorySyncProvider> = Arc::new(InMemoryProvider::new());

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);
        let peer = test_peer();
        let emit = noop_emitter();

        let outbound_fut = run_entry_exchange(
            true,
            Some("Berserk".to_string()),
            &peer,
            &emit,
            &outbound_provider,
            Box::new(client_send),
            Box::new(client_recv),
        );
        let inbound_fut = run_entry_exchange(
            false,
            None,
            &peer,
            &emit,
            &inbound_provider,
            Box::new(server_send),
            Box::new(server_recv),
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound deveria completar sem erro");
        inbound_result.expect("inbound deveria completar sem erro");

        let applied = inbound_provider.get_reading_progress_for_comic("Berserk".to_string());
        assert_eq!(applied.map(|entry| entry.last_page), Some(7));
    }

    /// Sem escopo registrado (`comic_name: None` do lado outbound), a sessão falha rápido em
    /// vez de tentar adivinhar qual quadrinho mandar.
    #[tokio::test]
    async fn entry_exchange_fails_fast_without_a_scoped_comic() {
        let provider: Arc<dyn HistorySyncProvider> = Arc::new(InMemoryProvider::new());
        let (client_io, _server_io) = tokio::io::duplex(1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let peer = test_peer();
        let emit = noop_emitter();

        let result = run_entry_exchange(
            true,
            None,
            &peer,
            &emit,
            &provider,
            Box::new(client_send),
            Box::new(client_recv),
        )
        .await;

        assert!(result.is_err());
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
