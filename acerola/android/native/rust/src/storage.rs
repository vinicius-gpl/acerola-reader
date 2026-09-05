use std::{path::Path, sync::Arc};

use acerola_p2p::api::{
    error::P2pError,
    identity::DeviceInfo,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
};
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::callbacks::SecureBlobStore;

const IDENTITY_KEY: &str = "identity";
const PEERS_KEY: &str = "peers";
const DEVICE_INFO_KEY: &str = "device_info";
const IROH_SERVICES_TICKET_KEY: &str = "iroh_services_ticket";

/// Implementação de `P2PStorage` que persiste via [`SecureBlobStore`] (Kotlin,
/// `EncryptedSharedPreferences` protegida pelo Android Keystore) em vez de arquivos em texto
/// puro — ver `callbacks::SecureBlobStore`. Chaves: `"identity"` (seed do nó) e `"peers"`
/// (cache de `PeerAddr` conhecidos, codificado como JSON antes de virar blob).
pub(crate) struct SecureP2pStorage {
    store: Arc<dyn SecureBlobStore>,
    peers_cache: RwLock<Vec<PeerAddr>>,
    device_info_cache: RwLock<Vec<(PeerIdentity, DeviceInfo)>>,
}

impl SecureP2pStorage {
    /// `legacy_dir`, se fornecido e ainda tiver `identity.seed`/`peers.json` (formato antigo,
    /// texto puro, de antes da migração pro `SecureBlobStore`), tem seu conteúdo migrado pro
    /// storage seguro numa única vez — sem isso, quem já tinha pareado dispositivos perderia a
    /// identidade P2P atual no primeiro update e precisaria re-parear tudo do zero.
    pub(crate) fn open(store: Arc<dyn SecureBlobStore>, legacy_dir: Option<&Path>) -> Self {
        if let Some(legacy_dir) = legacy_dir {
            Self::migrate_legacy_files(&store, legacy_dir);
        }

        // Falha real de backend na carga inicial do cache de peers não é fatal (o node ainda
        // sobe, só sem os endereços conhecidos) — mas precisa aparecer no log, não virar
        // silenciosamente "nenhum peer conhecido".
        let peers_cache = match store.load_blob(PEERS_KEY.to_string()) {
            Ok(Some(bytes)) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Ok(None) => Vec::new(),
            Err(err) => {
                tracing::error!(layer = "storage", error = %err, "failed to load cached peers, starting empty");
                Vec::new()
            }
        };

        let device_info_cache = match store.load_blob(DEVICE_INFO_KEY.to_string()) {
            Ok(Some(bytes)) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Ok(None) => Vec::new(),
            Err(err) => {
                tracing::error!(layer = "storage", error = %err, "failed to load cached device info, starting empty");
                Vec::new()
            }
        };

        Self {
            store,
            peers_cache: RwLock::new(peers_cache),
            device_info_cache: RwLock::new(device_info_cache),
        }
    }

    /// Remove um peer do cache de endereços conhecidos — usado quando o usuário desempareia
    /// um dispositivo pela UI. `get_paired_peers` (ver `api.rs`) lê exatamente desse cache,
    /// não do `trust_store`, então sem isso o peer sumiria da confiança mas continuaria
    /// aparecendo na lista de pareados.
    pub(crate) async fn remove_peer(&self, id: &str) -> Result<(), P2pError> {
        let mut peers = self.peers_cache.write().await;
        peers.retain(|cached| cached.id.id != id);

        let bytes = serde_json::to_vec(&*peers).map_err(|err| {
            P2pError::StartupFailed(format!("failed to encode peer cache: {err}"))
        })?;

        self.store
            .save_blob(PEERS_KEY.to_string(), bytes)
            .map_err(|err| P2pError::StartupFailed(format!("failed to save peer cache: {err}")))?;

        // Limpa também o nome persistido — sem isso, reparear o mesmo `id` de volta faria o
        // nome antigo "ressuscitar" antes do próximo handshake atualizar de verdade.
        let mut device_info = self.device_info_cache.write().await;
        device_info.retain(|(cached_id, _)| cached_id.id != id);

        let bytes = serde_json::to_vec(&*device_info).map_err(|err| {
            P2pError::StartupFailed(format!("failed to encode device info cache: {err}"))
        })?;

        self.store
            .save_blob(DEVICE_INFO_KEY.to_string(), bytes)
            .map_err(|err| {
                P2pError::StartupFailed(format!("failed to save device info cache: {err}"))
            })
    }

    /// Persiste o ticket da conta do usuário em `services.iroh.computer` (colado por ele na
    /// tela de configuração de rede) no mesmo cofre da identidade P2P — nunca em DataStore
    /// (que fica sem criptografia por padrão). Só tem efeito no próximo início do app, mesma
    /// regra das outras fontes de relay (a lib não suporta trocar relay em runtime).
    pub(crate) fn save_iroh_services_ticket(&self, ticket: &str) -> Result<(), P2pError> {
        self.store
            .save_blob(
                IROH_SERVICES_TICKET_KEY.to_string(),
                ticket.as_bytes().to_vec(),
            )
            .map_err(|err| {
                P2pError::StartupFailed(format!("failed to save Iroh Services ticket: {err}"))
            })
    }

    /// `None` quando o usuário nunca colou um ticket, ou quando o blob existe mas não decodifica
    /// como UTF-8 (corrompido) — trata como "não configurado" em vez de propagar erro, já que a
    /// rede pública do Iroh indisponível não deveria travar o resto da rede P2P.
    pub(crate) fn load_iroh_services_ticket(&self) -> Result<Option<String>, P2pError> {
        let bytes = self
            .store
            .load_blob(IROH_SERVICES_TICKET_KEY.to_string())
            .map_err(|err| {
                P2pError::StartupFailed(format!("failed to load Iroh Services ticket: {err}"))
            })?;

        Ok(bytes.and_then(|bytes| String::from_utf8(bytes).ok()))
    }

    /// Remove o ticket salvo — usado quando o usuário desliga a fonte ou substitui por um novo.
    pub(crate) fn clear_iroh_services_ticket(&self) -> Result<(), P2pError> {
        self.store
            .clear_blob(IROH_SERVICES_TICKET_KEY.to_string())
            .map_err(|err| {
                P2pError::StartupFailed(format!("failed to clear Iroh Services ticket: {err}"))
            })
    }

    fn migrate_legacy_files(store: &Arc<dyn SecureBlobStore>, legacy_dir: &Path) {
        Self::migrate_legacy_file(store, &legacy_dir.join("identity.seed"), IDENTITY_KEY);
        Self::migrate_legacy_file(store, &legacy_dir.join("peers.json"), PEERS_KEY);
    }

    /// Só migra se o storage seguro confirmar (`Ok(None)`) que a chave está genuinamente
    /// vazia — em caso de falha real de backend (`Err`), não migra nem apaga o arquivo
    /// legado, pra não arriscar perder o único dado bom que ainda existe.
    fn migrate_legacy_file(store: &Arc<dyn SecureBlobStore>, path: &Path, key: &str) {
        match store.load_blob(key.to_string()) {
            Ok(None) => {
                if let Ok(Some(bytes)) = crate::fsutil::read_optional(path) {
                    if let Err(err) = store.save_blob(key.to_string(), bytes) {
                        tracing::error!(layer = "storage", key, error = %err, "failed to migrate legacy file");
                        return;
                    }
                }
                std::fs::remove_file(path).ok();
            }
            Ok(Some(_)) => {
                // Já migrado numa execução anterior — só limpa o arquivo antigo.
                std::fs::remove_file(path).ok();
            }
            Err(err) => {
                tracing::error!(layer = "storage", key, error = %err, "secure store unavailable, skipping legacy migration");
            }
        }
    }
}

#[async_trait]
impl P2PStorage for SecureP2pStorage {
    async fn save_identity(&self, secret: &[u8]) -> Result<(), P2pError> {
        self.store
            .save_blob(IDENTITY_KEY.to_string(), secret.to_vec())
            .map_err(|err| P2pError::StartupFailed(format!("failed to save identity: {err}")))
    }

    async fn load_identity(&self) -> Result<Option<Vec<u8>>, P2pError> {
        self.store
            .load_blob(IDENTITY_KEY.to_string())
            .map_err(|err| P2pError::StartupFailed(format!("failed to load identity: {err}")))
    }

    async fn save_peer(&self, peer: &PeerAddr) -> Result<(), P2pError> {
        let mut peers = self.peers_cache.write().await;
        match peers.iter_mut().find(|cached| cached.id == peer.id) {
            Some(cached) => *cached = peer.clone(),
            None => peers.push(peer.clone()),
        }

        let bytes = serde_json::to_vec(&*peers).map_err(|err| {
            P2pError::StartupFailed(format!("failed to encode peer cache: {err}"))
        })?;

        self.store
            .save_blob(PEERS_KEY.to_string(), bytes)
            .map_err(|err| P2pError::StartupFailed(format!("failed to save peer cache: {err}")))
    }

    async fn load_peers(&self) -> Result<Vec<PeerAddr>, P2pError> {
        Ok(self.peers_cache.read().await.clone())
    }

    async fn save_device_info(
        &self,
        peer: &PeerIdentity,
        info: &DeviceInfo,
    ) -> Result<(), P2pError> {
        let mut device_info = self.device_info_cache.write().await;
        match device_info
            .iter_mut()
            .find(|(cached_id, _)| cached_id.id == peer.id)
        {
            Some(entry) => entry.1 = info.clone(),
            None => device_info.push((peer.clone(), info.clone())),
        }

        let bytes = serde_json::to_vec(&*device_info).map_err(|err| {
            P2pError::StartupFailed(format!("failed to encode device info cache: {err}"))
        })?;

        self.store
            .save_blob(DEVICE_INFO_KEY.to_string(), bytes)
            .map_err(|err| {
                P2pError::StartupFailed(format!("failed to save device info cache: {err}"))
            })
    }

    async fn load_device_info(&self) -> Result<Vec<(PeerIdentity, DeviceInfo)>, P2pError> {
        Ok(self.device_info_cache.read().await.clone())
    }
}

/// Newtype local em volta de `Arc<SecureP2pStorage>` — as regras de coerência do Rust não
/// deixam implementar uma trait estrangeira (`P2PStorage`, da `acerola-p2p`) direto pra
/// `Arc<SecureP2pStorage>`, já que `Arc` também é estrangeiro (não é um tipo "fundamental"
/// como `Box`). Com esse wrapper local, dá pra passar a MESMA instância pro builder (que
/// exige posse) e manter outra referência viva no `P2PNode` pra consultar depois — ex: pra
/// listar quem já foi pareado, já que a conexão do handshake em si é só um "toque" de ~2s
/// (troca PING/PONG/DeviceInfo e fecha), não uma sessão persistente.
pub(crate) struct SharedSecureP2pStorage(pub(crate) Arc<SecureP2pStorage>);

#[async_trait]
impl P2PStorage for SharedSecureP2pStorage {
    async fn save_identity(&self, secret: &[u8]) -> Result<(), P2pError> {
        self.0.save_identity(secret).await
    }

    async fn load_identity(&self) -> Result<Option<Vec<u8>>, P2pError> {
        self.0.load_identity().await
    }

    async fn save_peer(&self, peer: &PeerAddr) -> Result<(), P2pError> {
        self.0.save_peer(peer).await
    }

    async fn load_peers(&self) -> Result<Vec<PeerAddr>, P2pError> {
        self.0.load_peers().await
    }

    async fn save_device_info(
        &self,
        peer: &PeerIdentity,
        info: &DeviceInfo,
    ) -> Result<(), P2pError> {
        self.0.save_device_info(peer, info).await
    }

    async fn load_device_info(&self) -> Result<Vec<(PeerIdentity, DeviceInfo)>, P2pError> {
        self.0.load_device_info().await
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex as StdMutex};

    use acerola_p2p::api::peer::PeerIdentity;

    use super::*;
    use crate::callbacks::{SecureBlobStore, SecureBlobStoreError};

    struct FakeBlobStore {
        data: StdMutex<HashMap<String, Vec<u8>>>,
    }

    impl FakeBlobStore {
        fn new() -> Self {
            Self {
                data: StdMutex::new(HashMap::new()),
            }
        }
    }

    impl SecureBlobStore for FakeBlobStore {
        fn save_blob(&self, key: String, value: Vec<u8>) -> Result<(), SecureBlobStoreError> {
            self.data.lock().unwrap().insert(key, value);
            Ok(())
        }

        fn load_blob(&self, key: String) -> Result<Option<Vec<u8>>, SecureBlobStoreError> {
            Ok(self.data.lock().unwrap().get(&key).cloned())
        }

        fn clear_blob(&self, key: String) -> Result<(), SecureBlobStoreError> {
            self.data.lock().unwrap().remove(&key);
            Ok(())
        }
    }

    fn make_peer(id: &str) -> PeerAddr {
        PeerAddr {
            id: PeerIdentity {
                id: id.to_string(),
                device_id: None,
            },
            addrs: Vec::new(),
        }
    }

    #[tokio::test]
    async fn remove_peer_forgets_a_cached_address() {
        let storage = SecureP2pStorage::open(Arc::new(FakeBlobStore::new()), None);
        storage.save_peer(&make_peer("peer-a")).await.unwrap();

        storage.remove_peer("peer-a").await.unwrap();

        let remaining = storage.load_peers().await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn remove_peer_only_affects_the_given_peer() {
        let storage = SecureP2pStorage::open(Arc::new(FakeBlobStore::new()), None);
        storage.save_peer(&make_peer("peer-a")).await.unwrap();
        storage.save_peer(&make_peer("peer-b")).await.unwrap();

        storage.remove_peer("peer-a").await.unwrap();

        let remaining = storage.load_peers().await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id.id, "peer-b");
    }

    #[tokio::test]
    async fn remove_peer_persists_across_reopen() {
        let blob_store: Arc<dyn SecureBlobStore> = Arc::new(FakeBlobStore::new());

        let storage = SecureP2pStorage::open(Arc::clone(&blob_store), None);
        storage.save_peer(&make_peer("peer-a")).await.unwrap();
        storage.save_peer(&make_peer("peer-b")).await.unwrap();
        storage.remove_peer("peer-a").await.unwrap();

        let reopened = SecureP2pStorage::open(blob_store, None);
        let remaining = reopened.load_peers().await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id.id, "peer-b");
    }

    #[tokio::test]
    async fn removing_an_unknown_peer_is_a_no_op() {
        let storage = SecureP2pStorage::open(Arc::new(FakeBlobStore::new()), None);
        storage.save_peer(&make_peer("peer-a")).await.unwrap();

        storage.remove_peer("never-existed").await.unwrap();

        let remaining = storage.load_peers().await.unwrap();
        assert_eq!(remaining.len(), 1);
    }

    fn make_device_info(name: &str) -> DeviceInfo {
        DeviceInfo {
            name: name.to_string(),
            os: "android".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    #[tokio::test]
    async fn device_info_persists_across_reopen() {
        let blob_store: Arc<dyn SecureBlobStore> = Arc::new(FakeBlobStore::new());

        let storage = SecureP2pStorage::open(Arc::clone(&blob_store), None);
        storage
            .save_device_info(
                &make_peer("peer-a").id,
                &make_device_info("Pixel do Vinicius"),
            )
            .await
            .unwrap();

        let reopened = SecureP2pStorage::open(blob_store, None);
        let loaded = reopened.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0.id, "peer-a");
        assert_eq!(loaded[0].1.name, "Pixel do Vinicius");
    }

    #[tokio::test]
    async fn save_device_info_upserts_by_id_instead_of_duplicating() {
        let storage = SecureP2pStorage::open(Arc::new(FakeBlobStore::new()), None);

        storage
            .save_device_info(&make_peer("peer-a").id, &make_device_info("old-name"))
            .await
            .unwrap();
        storage
            .save_device_info(&make_peer("peer-a").id, &make_device_info("new-name"))
            .await
            .unwrap();

        let loaded = storage.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1.name, "new-name");
    }

    #[test]
    fn iroh_services_ticket_roundtrips_through_the_secure_store() {
        let storage = SecureP2pStorage::open(Arc::new(FakeBlobStore::new()), None);

        assert_eq!(storage.load_iroh_services_ticket().unwrap(), None);

        storage
            .save_iroh_services_ticket("services-fake-ticket")
            .unwrap();
        assert_eq!(
            storage.load_iroh_services_ticket().unwrap(),
            Some("services-fake-ticket".to_string())
        );
    }

    #[test]
    fn clear_iroh_services_ticket_removes_it_and_is_idempotent() {
        let storage = SecureP2pStorage::open(Arc::new(FakeBlobStore::new()), None);

        storage
            .save_iroh_services_ticket("services-fake-ticket")
            .unwrap();
        storage.clear_iroh_services_ticket().unwrap();
        assert_eq!(storage.load_iroh_services_ticket().unwrap(), None);

        // Chamar de novo sem nada salvo não é erro.
        storage.clear_iroh_services_ticket().unwrap();
    }

    #[tokio::test]
    async fn remove_peer_also_forgets_its_persisted_device_name() {
        let storage = SecureP2pStorage::open(Arc::new(FakeBlobStore::new()), None);
        storage.save_peer(&make_peer("peer-a")).await.unwrap();
        storage
            .save_device_info(
                &make_peer("peer-a").id,
                &make_device_info("Pixel do Vinicius"),
            )
            .await
            .unwrap();

        storage.remove_peer("peer-a").await.unwrap();

        assert!(storage.load_device_info().await.unwrap().is_empty());
    }
}
