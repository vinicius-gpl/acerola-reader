use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::infra::sync::messages::SyncDirection;

/// `Handler::handle()` (trait do `acerola-p2p`) é chamado sem nenhum parâmetro específico da
/// invocação — os handlers `acerola/sync-comic/1` são singletons registrados uma única vez no
/// boot (`bios::network::setup_network`). Pra o lado que INICIA um sync individual (o comando
/// Tauri `sync_comic`) conseguir dizer ao `ComicSyncOutbound` qual `comic_name`/`direction` usar
/// nessa chamada específica de `connect()`, esse registro funciona como um side-channel: o
/// comando grava a tripla (peer_id, comic_name, direction) antes de chamar `connect()`, e o
/// handler consome (`take`) esse valor assim que a sessão começa. Só o lado outbound precisa
/// disso — o lado inbound recebe `comic_name`/`direction` pelo próprio `ComicSyncRequest` no
/// wire.
#[derive(Default)]
pub struct PendingComicSyncRegistry {
    pending: Mutex<HashMap<String, (String, SyncDirection)>>,
}

impl PendingComicSyncRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn set(&self, peer_id: String, comic_name: String, direction: SyncDirection) {
        self.pending
            .lock()
            .expect("pending comic sync registry mutex poisoned")
            .insert(peer_id, (comic_name, direction));
    }

    /// Consome (remove) o `(comic_name, direction)` pendente pro peer — cada chamada de
    /// `connect()` só serve pra uma sessão, então não faz sentido deixar o valor lá depois de
    /// lido.
    pub fn take(&self, peer_id: &str) -> Option<(String, SyncDirection)> {
        self.pending.lock().expect("pending comic sync registry mutex poisoned").remove(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_removes_the_pending_entry() {
        let registry = PendingComicSyncRegistry::new();
        registry.set("peer-1".to_string(), "Berserk".to_string(), SyncDirection::Push);

        assert_eq!(registry.take("peer-1"), Some(("Berserk".to_string(), SyncDirection::Push)));
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingComicSyncRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }
}
