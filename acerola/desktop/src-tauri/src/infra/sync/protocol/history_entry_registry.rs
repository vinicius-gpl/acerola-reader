use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Escopo de UM push de histórico: o quadrinho e os capítulos (`chapter_archive_id`, mesma
/// unidade usada pelo resto da UI local, ex.: `markChaptersReadBatch`) selecionados pelo
/// usuário pra enviar — o manifesto trocado (`acerola/sync-history-entry/1`) fica restrito a
/// isso, nunca a biblioteca inteira. `HistorySyncService::build_manifest_for_chapters` resolve
/// os IDs pra `chapter_sort` (a chave comparável entre devices) antes de montar o manifesto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntryScope {
    pub comic_name: String,
    pub chapter_ids: Vec<i64>,
}

/// Mesmo side-channel de `PendingComicSyncRegistry`, só que pro push de histórico restrito a
/// capítulo(s) selecionado(s) (`acerola/sync-history-entry/1`): o comando Tauri
/// `sync_history_entry` grava o [`HistoryEntryScope`] antes de chamar `connect()`, e
/// `HistoryEntrySyncOutbound` consome (`take`) esse valor assim que a sessão começa.
#[derive(Default)]
pub struct PendingHistoryEntryRegistry {
    pending: Mutex<HashMap<String, HistoryEntryScope>>,
}

impl PendingHistoryEntryRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn set(&self, peer_id: String, scope: HistoryEntryScope) {
        self.pending
            .lock()
            .expect("pending history entry registry mutex poisoned")
            .insert(peer_id, scope);
    }

    /// Consome (remove) o escopo pendente pro peer — cada chamada de `connect()` só serve
    /// pra uma sessão.
    pub fn take(&self, peer_id: &str) -> Option<HistoryEntryScope> {
        self.pending.lock().expect("pending history entry registry mutex poisoned").remove(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(comic_name: &str, chapter_ids: &[i64]) -> HistoryEntryScope {
        HistoryEntryScope { comic_name: comic_name.to_string(), chapter_ids: chapter_ids.to_vec() }
    }

    #[test]
    fn take_removes_the_pending_entry() {
        let registry = PendingHistoryEntryRegistry::new();
        registry.set("peer-1".to_string(), scope("Berserk", &[1, 2]));

        assert_eq!(registry.take("peer-1"), Some(scope("Berserk", &[1, 2])));
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingHistoryEntryRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }
}
