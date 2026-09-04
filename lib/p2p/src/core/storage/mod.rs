//! Abstrações e implementações de persistência para o acerola-p2p (P2PStorage).
//!
//! Separação de responsabilidades:
//! - Vault: Persistência de segredos e chaves de identidade.
//! - Cache: Persistência de metadados e endereços conhecidos de peers para reconexão/bootstrapping.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    data::identity::device_info::DeviceInfo,
    infra::{
        error::ConnectionError,
        peer::{PeerAddr, PeerId},
    },
};

/// Contrato principal para persistência de identidade e peers descobertos.
#[async_trait]
pub trait P2PStorage: Send + Sync {
    /// Salva a chave/seed de identidade do nó local no Vault seguro.
    async fn save_identity(&self, secret: &[u8]) -> Result<(), ConnectionError>;

    /// Carrega a chave/seed de identidade salva anteriormente no Vault.
    async fn load_identity(&self) -> Result<Option<Vec<u8>>, ConnectionError>;

    /// Salva um peer e seus dados de discagem no Cache de peers.
    async fn save_peer(&self, peer: &PeerAddr) -> Result<(), ConnectionError>;

    /// Carrega todos os peers previamente conhecidos do Cache.
    async fn load_peers(&self) -> Result<Vec<PeerAddr>, ConnectionError>;

    /// Salva as informações de dispositivo (nome, SO, versão) trocadas no handshake com um
    /// peer — sem isso, o nome de um peer só existe em memória (`NetworkState`) e some ao
    /// reiniciar o app até o próximo handshake com aquele peer específico.
    async fn save_device_info(
        &self, peer: &PeerId, info: &DeviceInfo,
    ) -> Result<(), ConnectionError>;

    /// Carrega todas as informações de dispositivo persistidas anteriormente.
    async fn load_device_info(&self) -> Result<Vec<(PeerId, DeviceInfo)>, ConnectionError>;
}

/// Implementação padrão em memória do `P2PStorage` (ideal para testes ou ambientes sem disco).
#[derive(Default, Clone)]
pub struct InMemoryStorage {
    identity: Arc<RwLock<Option<Vec<u8>>>>,
    peers: Arc<RwLock<HashMap<PeerId, PeerAddr>>>,
    device_info: Arc<RwLock<HashMap<PeerId, DeviceInfo>>>,
}

impl InMemoryStorage {
    /// Instancia um novo storage em memória limpo.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl P2PStorage for InMemoryStorage {
    async fn save_identity(&self, secret: &[u8]) -> Result<(), ConnectionError> {
        *self.identity.write().await = Some(secret.to_vec());
        Ok(())
    }

    async fn load_identity(&self) -> Result<Option<Vec<u8>>, ConnectionError> {
        Ok(self.identity.read().await.clone())
    }

    async fn save_peer(&self, peer: &PeerAddr) -> Result<(), ConnectionError> {
        self.peers.write().await.insert(peer.id.clone(), peer.clone());
        Ok(())
    }

    async fn load_peers(&self) -> Result<Vec<PeerAddr>, ConnectionError> {
        Ok(self.peers.read().await.values().cloned().collect())
    }

    async fn save_device_info(
        &self, peer: &PeerId, info: &DeviceInfo,
    ) -> Result<(), ConnectionError> {
        self.device_info.write().await.insert(peer.clone(), info.clone());
        Ok(())
    }

    async fn load_device_info(&self) -> Result<Vec<(PeerId, DeviceInfo)>, ConnectionError> {
        Ok(self
            .device_info
            .read()
            .await
            .iter()
            .map(|(peer, info)| (peer.clone(), info.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer(id: &str) -> PeerId {
        PeerId { id: id.to_string(), device_id: None }
    }

    fn make_addr(id: &str) -> PeerAddr {
        PeerAddr { id: make_peer(id), addrs: vec![1, 2, 3] }
    }

    #[tokio::test]
    async fn in_memory_storage_identity_roundtrip() {
        let storage = InMemoryStorage::new();
        assert!(storage.load_identity().await.unwrap().is_none());

        let secret = [0x42u8; 32];
        storage.save_identity(&secret).await.unwrap();

        let loaded = storage.load_identity().await.unwrap();
        assert_eq!(loaded, Some(secret.to_vec()));
    }

    #[tokio::test]
    async fn in_memory_storage_peers_roundtrip() {
        let storage = InMemoryStorage::new();
        assert!(storage.load_peers().await.unwrap().is_empty());

        let addr1 = make_addr("node-1");
        let addr2 = make_addr("node-2");

        storage.save_peer(&addr1).await.unwrap();
        storage.save_peer(&addr2).await.unwrap();

        let loaded = storage.load_peers().await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.contains(&addr1));
        assert!(loaded.contains(&addr2));
    }

    fn make_device_info(name: &str) -> DeviceInfo {
        DeviceInfo { name: name.to_string(), os: "linux".to_string(), version: "0.0.1".to_string() }
    }

    #[tokio::test]
    async fn in_memory_storage_device_info_roundtrip() {
        let storage = InMemoryStorage::new();
        assert!(storage.load_device_info().await.unwrap().is_empty());

        let peer = make_peer("node-1");
        storage.save_device_info(&peer, &make_device_info("my-pc")).await.unwrap();

        let loaded = storage.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, peer);
        assert_eq!(loaded[0].1.name, "my-pc");
    }

    #[tokio::test]
    async fn in_memory_storage_save_device_info_overwrites_existing() {
        let storage = InMemoryStorage::new();
        let peer = make_peer("node-1");

        storage.save_device_info(&peer, &make_device_info("old-name")).await.unwrap();
        storage.save_device_info(&peer, &make_device_info("new-name")).await.unwrap();

        let loaded = storage.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1.name, "new-name");
    }
}
