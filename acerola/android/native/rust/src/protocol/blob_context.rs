//! Ponte entre os protocolos de aplicação (`sync-files`, `sync-comic`, futuramente
//! `browse-cover`) e a capacidade de blobs do node. O `Handler` da lib (`acerola-p2p`) só
//! entrega a `PeerIdentity` de quem chamou, sem endereço de rede — mas `P2pBlobStore::fetch`
//! precisa de um `PeerAddr` completo (abre uma conexão QUIC própria, independente do stream do
//! protocolo em si) e do próprio `Arc<dyn P2pBlobStore>`, os dois só disponíveis via métodos do
//! `AcerolaP2p` (`node.blobs()`, `node.known_peers()`).
//!
//! O node só existe DEPOIS que `AcerolaP2p::builder(...).build()` retorna, mas os handlers são
//! passados PRO builder antes disso — não dá pra injetar `Arc<AcerolaP2p>` direto na construção
//! deles. `BlobContext` resolve isso guardando um `Weak<AcerolaP2p>` (`set_node`), preenchido
//! logo depois de cada `.build()` em `api.rs` — tanto no boot quanto em todo `P2PNode::restart`
//! — antes de qualquer conexão poder disparar um handler. `RwLock` (não `OnceLock`) porque um
//! restart chama `set_node` de novo: com `OnceLock` a segunda chamada seria um no-op silencioso
//! e os handlers ficariam presos pro sempre no node antigo já desligado.

use std::sync::{Arc, RwLock, Weak};

use acerola_p2p::api::{
    blobs::P2pBlobStore,
    error::P2pError,
    peer::{PeerAddr, PeerIdentity},
    AcerolaP2p,
};

#[derive(Default)]
pub(crate) struct BlobContext {
    node: RwLock<Option<Weak<AcerolaP2p>>>,
}

impl BlobContext {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub(crate) fn set_node(&self, node: &Arc<AcerolaP2p>) {
        *self
            .node
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::downgrade(node));
    }

    fn node(&self) -> Result<Arc<AcerolaP2p>, P2pError> {
        self.node
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or_else(|| P2pError::StreamFailed("p2p node not initialized yet".into()))
    }

    pub(crate) async fn blob_store(&self) -> Result<Arc<dyn P2pBlobStore>, P2pError> {
        self.node()?
            .blobs()
            .await
            .ok_or_else(|| P2pError::StreamFailed("blob store not configured on this node".into()))
    }

    /// Resolve o `PeerAddr` mais recente conhecido para `peer` — cache alimentado pelo
    /// handshake (`acerola/handshake/1`) e por chamadas explícitas (`connect`/`sync_comic`).
    pub(crate) async fn resolve_addr(&self, peer: &PeerIdentity) -> Result<PeerAddr, P2pError> {
        self.node()?
            .known_peers()
            .await
            .into_iter()
            .find(|(id, _, _)| id == peer)
            .map(|(_, addr, _)| addr)
            .ok_or_else(|| P2pError::StreamFailed(format!("no known address for peer {}", peer.id)))
    }

    /// RTT medido com `peer` agora, usado só pra dimensionar paralelismo de fetch de blob (ver
    /// `exchange.rs::receive_files`) — nunca crítico o bastante pra virar `Err`. `None` tanto
    /// sem conexão viva quanto sem node (nesse caso o chamador já vai falhar logo depois em
    /// `resolve_addr`/`blob_store`, então não vale a pena duplicar o erro aqui). Espelha
    /// `BlobContext::latency` do Desktop (`infra/sync/blob_context.rs`) — mesmo contrato de
    /// concorrência, plataformas diferentes.
    pub(crate) async fn latency(&self, peer: &PeerIdentity) -> Option<std::time::Duration> {
        self.node().ok()?.latency(peer).await
    }
}
