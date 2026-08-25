use std::{collections::HashSet, sync::Arc};

use acerola_p2p::api::{
    guard::{InMemoryTrustedStore, TofuGuard, TrustedPeerStore},
    identity::DeviceInfo,
    network::NetworkMode,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
    AcerolaP2p,
};
use async_trait::async_trait;

use crate::infra::security::{p2p_storage::SecureP2pStorage, trusted_store::SecureTrustedStore};

pub type ConnectedPeerInfo = (PeerIdentity, HashSet<Vec<u8>>, Option<DeviceInfo>);

#[async_trait]
pub trait NetworkServiceApi: Send + Sync + 'static {
    fn local_id(&self) -> Result<String, String>;
    fn local_addr(&self) -> Result<PeerAddr, String>;
    fn local_device_info(&self) -> Result<DeviceInfo, String>;
    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String>;
    /// Todo peer já pareado (TOFU) alguma vez, com o último endereço conhecido — persiste
    /// entre reinícios e independe de conexão ativa agora, diferente de
    /// [`NetworkServiceApi::connected_peers_with_info`] (sessão de protocolo, dura só
    /// segundos). É essa lista que deve alimentar "disparar sync com X" na UI, já que o peer
    /// quase nunca está conectado no exato instante em que o usuário clica o botão.
    ///
    /// `DeviceInfo` vem de `AcerolaP2p::known_peers()` (mesmo mecanismo persistente,
    /// sobrevive ao handshake fechar) — não de `connected_peers_with_info`, que só tem dado
    /// nos poucos segundos em que a sessão de handshake está de fato aberta.
    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String>;
    /// Desempareia um peer: some da confiança (TOFU) e do cache de endereços conhecidos —
    /// mesmo par de fontes que [`NetworkServiceApi::paired_peers`] lê. Não derruba uma
    /// conexão ativa nem bloqueia o peer; se ele reconectar depois, passa pelo mesmo fluxo
    /// TOFU de um dispositivo nunca visto (mesmo comportamento do lado Android, ver
    /// `P2PNode::remove_paired_peer`).
    async fn remove_peer(&self, id: String) -> Result<(), String>;
    async fn switch_to_local(&self) -> Result<(), String>;
    async fn switch_to_relay(&self) -> Result<(), String>;
    async fn mode(&self) -> Result<NetworkMode, String>;
    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String>;
    async fn shutdown(&self) -> Result<(), String>;
}

pub struct NetworkService {
    node: Arc<AcerolaP2p>,
    /// Mesmo storage passado ao builder (`.storage(...)`) — clonado antes por
    /// `bios::network::setup_network` pra sobreviver aqui como fonte de "peers pareados"
    /// persistidos, já que a lib não devolve o storage de volta depois do `build()`. Tipo
    /// concreto (não `Arc<dyn P2PStorage>`) porque [`Self::remove_peer`] precisa de
    /// `SecureP2pStorage::remove_peer`, que é específico dessa implementação — não faz parte
    /// da trait compartilhada com o lado Android.
    storage: Arc<SecureP2pStorage>,
    /// Mesmo motivo do campo acima, pro lado da confiança (TOFU) — `SecureTrustedStore::remove`
    /// também não faz parte de `TrustedPeerStore`.
    trust_store: Arc<SecureTrustedStore>,
}

impl NetworkService {
    pub fn new(
        node: Arc<AcerolaP2p>, storage: Arc<SecureP2pStorage>, trust_store: Arc<SecureTrustedStore>,
    ) -> Self {
        Self { node, storage, trust_store }
    }
}

#[async_trait]
impl NetworkServiceApi for NetworkService {
    fn local_id(&self) -> Result<String, String> {
        Ok(self.node.local_id().to_string())
    }

    /// Endereço completo (id + bytes de endereçamento) usado pra gerar o código/QR de
    /// pareamento — é o que o outro dispositivo precisa pra nos alcançar via `connect()`.
    fn local_addr(&self) -> Result<PeerAddr, String> {
        Ok(self.node.local_addr().clone())
    }

    /// Nome/OS/versão deste dispositivo — usado na tela de Rede pra exibir algo mais
    /// legível que o peer id cru (ex: "Notebook do Vinicius" em vez de um hex de 64 chars).
    fn local_device_info(&self) -> Result<DeviceInfo, String> {
        Ok(self.node.local_device_info().clone())
    }

    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String> {
        Ok(self.node.connected_peers_with_info().await)
    }

    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String> {
        use std::collections::HashMap;

        let peers = self.storage.load_peers().await.map_err(|err| err.to_string())?;

        let device_info_by_peer: HashMap<String, DeviceInfo> = self
            .node
            .known_peers()
            .await
            .into_iter()
            .filter_map(|(peer, _, info)| info.map(|device| (peer.id, device)))
            .collect();

        Ok(peers
            .into_iter()
            .map(|addr| {
                let device = device_info_by_peer.get(&addr.id.id).cloned();
                (addr, device)
            })
            .collect())
    }

    async fn remove_peer(&self, id: String) -> Result<(), String> {
        self.trust_store.remove(&id).await.map_err(|err| err.to_string())?;
        self.storage.remove_peer(&id).await.map_err(|err| err.to_string())
    }

    async fn switch_to_local(&self) -> Result<(), String> {
        let store = Arc::new(InMemoryTrustedStore::new());
        let guard = TofuGuard::new(store as Arc<dyn TrustedPeerStore>).into_validator();
        self.node.switch_guard(guard, NetworkMode::Local).await.map_err(|err| err.to_string())
    }

    async fn switch_to_relay(&self) -> Result<(), String> {
        let store = Arc::new(InMemoryTrustedStore::new());
        let guard = TofuGuard::new(store as Arc<dyn TrustedPeerStore>).into_validator();
        self.node.switch_guard(guard, NetworkMode::Relay).await.map_err(|err| err.to_string())
    }

    async fn mode(&self) -> Result<NetworkMode, String> {
        Ok(self.node.mode().await)
    }

    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String> {
        self.node.connect(peer_addr, &alpn).await.map_err(|err| err.to_string())
    }

    async fn shutdown(&self) -> Result<(), String> {
        self.node.shutdown().await.map_err(|err| err.to_string())
    }
}
