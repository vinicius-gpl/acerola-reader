use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

/// O comando Tauri `query_remote_cover` grava `(comic_name, known_version)` pro peer ANTES de
/// chamar `connect()`, e `CoverBrowseOutbound::handle` consome (`take`) esse valor assim que a
/// sessão outbound começa — mesma ideia de `comic_sync_registry.rs::PendingComicSyncRegistry`,
/// mas com uma diferença importante: `use-remote-library.svelte.ts::fetchCoversFor` dispara UM
/// `query_remote_cover` por quadrinho da biblioteca remota, todos em paralelo (`Promise.all`),
/// pro MESMO peer. Um slot único por peer (versão anterior, com `insert`/`remove`) permitia que
/// o `push` de uma chamada sobrescrevesse o de outra antes do `CoverBrowseOutbound::handle`
/// correspondente rodar — a maioria das sessões achava o slot vazio e nunca chegava a escrever
/// o `CoverRequest` no stream, resultando em `StreamFailed("no pending cover request registered
/// for this peer")` no Desktop e `StreamFailed("stream closed before cover message")` do lado
/// que aceita (Android), só a última sobrevivendo. Reportado/reproduzido ao vivo em 22/08/2026.
///
/// Fila (FIFO) por peer em vez de slot único: `push` empilha, `take` desempilha o mais antigo.
/// Não faz diferença qual conexão física carrega qual pedido — só importa que cada `push` tenha
/// exatamente um `take` correspondente, sem overwrite e sem perda. `CoverBrowseOutbound` é um
/// singleton sem outra forma de saber `comic_name` por chamada de `connect()`, então não dá pra
/// indexar por `(peer_id, comic_name)` no lado do `take` — quem chama `take` só tem `peer_id`.
///
/// Só o lado outbound precisa disso — o lado inbound recebe tudo pelo próprio `CoverRequest` no
/// wire.
type PendingCoverRequest = (String, Option<i64>);

#[derive(Default)]
pub struct PendingCoverRequestRegistry {
    pending: Mutex<HashMap<String, VecDeque<PendingCoverRequest>>>,
}

impl PendingCoverRequestRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Enfileira um pedido pendente pro peer — nunca sobrescreve um pedido já enfileirado.
    pub fn push(&self, peer_id: String, comic_name: String, known_version: Option<i64>) {
        self.pending
            .lock()
            .expect("pending cover request registry mutex poisoned")
            .entry(peer_id)
            .or_default()
            .push_back((comic_name, known_version));
    }

    /// Retira o pedido mais antigo ainda pendente pro peer (FIFO). `None` se não há nenhum.
    pub fn take(&self, peer_id: &str) -> Option<PendingCoverRequest> {
        let mut pending =
            self.pending.lock().expect("pending cover request registry mutex poisoned");
        let queue = pending.get_mut(peer_id)?;
        let next = queue.pop_front();
        if queue.is_empty() {
            pending.remove(peer_id);
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_removes_the_pending_entry() {
        let registry = PendingCoverRequestRegistry::new();
        registry.push("peer-1".to_string(), "Berserk".to_string(), Some(42));

        assert_eq!(registry.take("peer-1"), Some(("Berserk".to_string(), Some(42))));
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingCoverRequestRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }

    /// Regressão do bug reportado em 22/08/2026: `fetchCoversFor` dispara N `query_remote_cover`
    /// em paralelo pro mesmo peer — com o slot único antigo, o segundo `push` (na época `set`)
    /// apagava o primeiro antes de qualquer `take` rodar, e uma das duas sessões sempre achava
    /// o registry vazio. Com a fila, os dois pedidos sobrevivem e saem na ordem em que entraram.
    #[test]
    fn concurrent_pushes_for_the_same_peer_do_not_overwrite_each_other() {
        let registry = PendingCoverRequestRegistry::new();
        registry.push("peer-1".to_string(), "Berserk".to_string(), None);
        registry.push("peer-1".to_string(), "Vinland Saga".to_string(), Some(3));

        assert_eq!(registry.take("peer-1"), Some(("Berserk".to_string(), None)));
        assert_eq!(registry.take("peer-1"), Some(("Vinland Saga".to_string(), Some(3))));
        assert_eq!(registry.take("peer-1"), None);
    }

    /// Peers diferentes têm filas independentes — pedidos pendentes pro peer A não vazam pro
    /// `take` do peer B nem são consumidos por ele.
    #[test]
    fn different_peers_have_independent_queues() {
        let registry = PendingCoverRequestRegistry::new();
        registry.push("peer-a".to_string(), "A".to_string(), None);
        registry.push("peer-b".to_string(), "B".to_string(), None);

        assert_eq!(registry.take("peer-b"), Some(("B".to_string(), None)));
        assert_eq!(registry.take("peer-a"), Some(("A".to_string(), None)));
    }
}
