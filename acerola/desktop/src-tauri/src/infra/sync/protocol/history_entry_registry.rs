use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Mesmo side-channel de `PendingComicSyncRegistry`, só que pro push individual de UMA entrada
/// de histórico (`acerola/sync-history-entry/1`): o comando Tauri `sync_history_entry` grava o
/// `comic_name` antes de chamar `connect()`, e `HistoryEntrySyncOutbound` consome (`take`) esse
/// valor assim que a sessão começa.
#[derive(Default)]
pub struct PendingHistoryEntryRegistry {
    pending: Mutex<HashMap<String, String>>,
}

impl PendingHistoryEntryRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn set(&self, peer_id: String, comic_name: String) {
        self.pending
            .lock()
            .expect("pending history entry registry mutex poisoned")
            .insert(peer_id, comic_name);
    }

    /// Consome (remove) o `comic_name` pendente pro peer — cada chamada de `connect()` só serve
    /// pra uma sessão.
    pub fn take(&self, peer_id: &str) -> Option<String> {
        self.pending.lock().expect("pending history entry registry mutex poisoned").remove(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_removes_the_pending_entry() {
        let registry = PendingHistoryEntryRegistry::new();
        registry.set("peer-1".to_string(), "Berserk".to_string());

        assert_eq!(registry.take("peer-1"), Some("Berserk".to_string()));
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingHistoryEntryRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }
}
