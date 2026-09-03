use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use acerola_p2p::api::{
    error::P2pError,
    identity::DeviceInfo,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
};
use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{decrypt, encrypt};

/// Colapsa entradas com o mesmo `id.id`, mantendo a última (mesma regra de "mais recente
/// vence" do upsert em `save_peer`). Migração pra arquivos `peers.enc` gravados antes da
/// correção do upsert (que comparava `PeerId` inteira, então `device_id: None` vs
/// `device_id: Some(..)` pro mesmo peer virava duas entradas) — sem isso, quem já tem uma
/// duplicata persistida continuaria vendo `pairedPeers` quebrar `{#each ... (peer.peerId)}`
/// no frontend mesmo depois de atualizar.
fn dedupe_by_id(peers: Vec<PeerAddr>) -> Vec<PeerAddr> {
    let mut deduped: Vec<PeerAddr> = Vec::with_capacity(peers.len());
    for peer in peers {
        match deduped.iter_mut().find(|cached| cached.id.id == peer.id.id) {
            Some(cached) => *cached = peer,
            None => deduped.push(peer),
        }
    }
    deduped
}

/// Implementação de `P2PStorage` (identidade do nó + cache de peers pareados) com
/// criptografia em repouso — ver `infra::security` pro esquema (chave mestra no
/// keyring do SO, AES-256-GCM pro resto). Layout dentro de `base_dir`:
/// - `identity.enc` — seed de identidade do nó, criptografada.
/// - `peers.enc`    — lista de `PeerAddr` conhecidos (JSON antes de criptografar).
pub struct SecureP2pStorage {
    base_dir: PathBuf,
    master_key: [u8; 32],
    peers_cache: RwLock<Vec<PeerAddr>>,
    device_info_cache: RwLock<Vec<(PeerIdentity, DeviceInfo)>>,
}

impl SecureP2pStorage {
    pub fn open(base_dir: impl Into<PathBuf>, master_key: [u8; 32]) -> std::io::Result<Self> {
        let base_dir = base_dir.into();
        std::fs::create_dir_all(&base_dir)?;

        let peers_cache = Self::read_peers(&base_dir, &master_key)?;
        let device_info_cache = Self::read_device_info(&base_dir, &master_key)?;

        Ok(Self {
            base_dir,
            master_key,
            peers_cache: RwLock::new(peers_cache),
            device_info_cache: RwLock::new(device_info_cache),
        })
    }

    fn identity_path(base_dir: &Path) -> PathBuf {
        base_dir.join("identity.enc")
    }

    fn peers_path(base_dir: &Path) -> PathBuf {
        base_dir.join("peers.enc")
    }

    fn device_info_path(base_dir: &Path) -> PathBuf {
        base_dir.join("device_info.enc")
    }

    /// Lê e descriptografa `peers.enc`. Diferente da versão anterior, uma falha real de
    /// descriptografia (chave mestra errada/corrompida) vira `Err` daqui — não pode
    /// silenciosamente virar "lista vazia", porque `save_peer` reescreve o arquivo inteiro
    /// com o que estiver em memória: um cache vazio por engano apaga todo peer pareado
    /// anterior assim que a próxima sincronização salvar algo.
    fn read_peers(base_dir: &Path, master_key: &[u8; 32]) -> std::io::Result<Vec<PeerAddr>> {
        let peers = match std::fs::read(Self::peers_path(base_dir)) {
            Ok(encrypted) => {
                let json = decrypt(master_key, &encrypted).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to decrypt peers.enc (wrong master key or corrupted file): {err}"),
                    )
                })?;
                serde_json::from_slice(&json).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to parse peers.enc contents: {err}"),
                    )
                })?
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(err) => return Err(err),
        };

        Ok(dedupe_by_id(peers))
    }

    /// Lê e descriptografa `device_info.enc` — mesma lógica de `read_peers`, mas pro nome do
    /// dispositivo trocado no handshake. Antes desta persistência, esse dado só existia em
    /// memória (`NetworkState`) e sumia da UI (caindo pro ID cru) até o peer reconectar e
    /// refazer o handshake nesta sessão.
    fn read_device_info(
        base_dir: &Path, master_key: &[u8; 32],
    ) -> std::io::Result<Vec<(PeerIdentity, DeviceInfo)>> {
        match std::fs::read(Self::device_info_path(base_dir)) {
            Ok(encrypted) => {
                let json = decrypt(master_key, &encrypted).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "failed to decrypt device_info.enc (wrong master key or corrupted file): {err}"
                        ),
                    )
                })?;
                serde_json::from_slice(&json).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to parse device_info.enc contents: {err}"),
                    )
                })
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl P2PStorage for SecureP2pStorage {
    async fn save_identity(&self, secret: &[u8]) -> Result<(), P2pError> {
        let encrypted = encrypt(&self.master_key, secret);
        std::fs::write(Self::identity_path(&self.base_dir), encrypted)
            .map_err(|err| P2pError::StreamFailed(format!("failed to save identity: {err}")))
    }

    async fn load_identity(&self) -> Result<Option<Vec<u8>>, P2pError> {
        let encrypted = match std::fs::read(Self::identity_path(&self.base_dir)) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(P2pError::StreamFailed(format!("failed to load identity: {err}"))),
        };

        decrypt(&self.master_key, &encrypted)
            .map(Some)
            .map_err(|err| P2pError::StreamFailed(format!("failed to decrypt identity: {err}")))
    }

    async fn save_peer(&self, peer: &PeerAddr) -> Result<(), P2pError> {
        let mut peers = self.peers_cache.write().await;
        // Compara só `id.id` (a string de identidade), não a `PeerId` inteira — `PeerId` deriva
        // `PartialEq` incluindo `device_id`, que nem sempre é conhecido em quem chama (ex:
        // comandos do frontend montam `PeerIdentity { id, device_id: None }` porque só têm a
        // string). Comparar a struct inteira faz uma reconexão sem `device_id` duplicar o peer
        // já pareado (que tem `device_id: Some(..)` do handshake original) em vez de atualizá-lo
        // — e a lista de pareados vira o `pairedPeers` que a UI usa como chave de `{#each}`, então
        // a duplicata quebra a renderização (`each_key_duplicate`) em vez de só ser um dado feio.
        match peers.iter_mut().find(|cached| cached.id.id == peer.id.id) {
            Some(cached) => *cached = peer.clone(),
            None => peers.push(peer.clone()),
        }

        let json = serde_json::to_vec(&*peers)
            .map_err(|err| P2pError::StreamFailed(format!("failed to encode peer cache: {err}")))?;
        let encrypted = encrypt(&self.master_key, &json);
        std::fs::write(Self::peers_path(&self.base_dir), encrypted)
            .map_err(|err| P2pError::StreamFailed(format!("failed to save peer cache: {err}")))
    }

    async fn load_peers(&self) -> Result<Vec<PeerAddr>, P2pError> {
        Ok(self.peers_cache.read().await.clone())
    }

    async fn save_device_info(&self, peer: &PeerIdentity, info: &DeviceInfo) -> Result<(), P2pError> {
        let mut device_info = self.device_info_cache.write().await;
        match device_info.iter_mut().find(|(cached_id, _)| cached_id.id == peer.id) {
            Some(entry) => entry.1 = info.clone(),
            None => device_info.push((peer.clone(), info.clone())),
        }

        let json = serde_json::to_vec(&*device_info)
            .map_err(|err| P2pError::StreamFailed(format!("failed to encode device info cache: {err}")))?;
        let encrypted = encrypt(&self.master_key, &json);
        std::fs::write(Self::device_info_path(&self.base_dir), encrypted)
            .map_err(|err| P2pError::StreamFailed(format!("failed to save device info cache: {err}")))
    }

    async fn load_device_info(&self) -> Result<Vec<(PeerIdentity, DeviceInfo)>, P2pError> {
        Ok(self.device_info_cache.read().await.clone())
    }
}

impl SecureP2pStorage {
    /// Remove um peer da lista de pareados — usado por "desparear". Idempotente: id
    /// desconhecido não é erro, só não reescreve o arquivo à toa. Não faz parte de
    /// `P2PStorage` (trait compartilhada com o lado Android) de propósito — a lib nunca
    /// precisa remover um peer sozinha, só a UI de "desparear" chama isso.
    pub async fn remove_peer(&self, id: &str) -> std::io::Result<()> {
        let mut peers = self.peers_cache.write().await;
        let before = peers.len();
        peers.retain(|peer| peer.id.id != id);
        if peers.len() != before {
            let json = serde_json::to_vec(&*peers).expect("Vec<PeerAddr> serialization cannot fail");
            std::fs::write(Self::peers_path(&self.base_dir), encrypt(&self.master_key, &json))?;
        }

        // Limpa também o nome persistido — sem isso, reparear o mesmo `id` de volta faria o
        // nome antigo "ressuscitar" antes do próximo handshake atualizar de verdade.
        let mut device_info = self.device_info_cache.write().await;
        let before = device_info.len();
        device_info.retain(|(cached_id, _)| cached_id.id != id);
        if device_info.len() != before {
            let json =
                serde_json::to_vec(&*device_info).expect("Vec<(PeerIdentity, DeviceInfo)> serialization cannot fail");
            std::fs::write(Self::device_info_path(&self.base_dir), encrypt(&self.master_key, &json))?;
        }

        Ok(())
    }
}

/// Delega pra um `Arc<SecureP2pStorage>` — deixa o mesmo storage passado ao builder
/// (`AcerolaP2p::builder().storage(...)`, que exige posse por valor) também acessível fora
/// dele. Sem isso, `paired_peers()` (`NetworkServiceApi`) não teria como ler os peers
/// persistidos: a lib não devolve o storage de volta depois do `build()`.
pub struct SharedP2pStorage(pub Arc<SecureP2pStorage>);

#[async_trait]
impl P2PStorage for SharedP2pStorage {
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

    async fn save_device_info(&self, peer: &PeerIdentity, info: &DeviceInfo) -> Result<(), P2pError> {
        self.0.save_device_info(peer, info).await
    }

    async fn load_device_info(&self) -> Result<Vec<(PeerIdentity, DeviceInfo)>, P2pError> {
        self.0.load_device_info().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acerola_p2p::api::peer::PeerIdentity;

    fn peer(id: &str) -> PeerAddr {
        PeerAddr { id: PeerIdentity { id: id.to_string(), device_id: None }, addrs: vec![1, 2, 3] }
    }

    #[tokio::test]
    async fn identity_roundtrip_through_encrypted_file() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [5u8; 32]).unwrap();

        assert_eq!(storage.load_identity().await.unwrap(), None);

        storage.save_identity(b"my secret seed").await.unwrap();
        assert_eq!(storage.load_identity().await.unwrap(), Some(b"my secret seed".to_vec()));

        // O arquivo no disco não pode conter o segredo em texto puro.
        let raw = std::fs::read(dir.path().join("identity.enc")).unwrap();
        assert!(!raw.windows(b"my secret seed".len()).any(|window| window == b"my secret seed"));
    }

    #[tokio::test]
    async fn save_peer_upserts_by_id_and_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [9u8; 32];

        {
            let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
            storage.save_peer(&peer("peer-a")).await.unwrap();
            storage.save_peer(&peer("peer-a")).await.unwrap(); // upsert, não duplica
            storage.save_peer(&peer("peer-b")).await.unwrap();
            assert_eq!(storage.load_peers().await.unwrap().len(), 2);
        }

        // Reabre (simula restart do app) — o cache tem que vir do arquivo criptografado.
        let reopened = SecureP2pStorage::open(dir.path(), key).unwrap();
        let peers = reopened.load_peers().await.unwrap();
        assert_eq!(peers.len(), 2);
        assert!(peers.iter().any(|p| p.id.id == "peer-a"));
        assert!(peers.iter().any(|p| p.id.id == "peer-b"));
    }

    /// Regressão: `sync_comic`/`query_remote_library`/etc. montam `PeerIdentity { id, device_id:
    /// None }` porque o frontend só manda a string do peer — mas o pareamento inicial (handshake)
    /// salva com `device_id: Some(uuid)`. Se o upsert comparasse `PeerId` inteira (como fazia
    /// antes), essas duas chamadas para o MESMO peer virariam duas entradas na lista, e a UI
    /// (`{#each pairedPeers as peer (peer.peerId)}`) quebra com chave duplicada.
    #[tokio::test]
    async fn save_peer_upserts_by_id_string_even_when_device_id_differs() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [7u8; 32]).unwrap();

        let paired_via_handshake = PeerAddr {
            id: PeerIdentity { id: "peer-a".to_string(), device_id: Some("uuid-1".to_string()) },
            addrs: vec![1, 2, 3],
        };
        let reconnected_via_action = PeerAddr {
            id: PeerIdentity { id: "peer-a".to_string(), device_id: None },
            addrs: vec![4, 5, 6],
        };

        storage.save_peer(&paired_via_handshake).await.unwrap();
        storage.save_peer(&reconnected_via_action).await.unwrap();

        let peers = storage.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1, "mesmo peer.id.id deveria atualizar, não duplicar");
        assert_eq!(peers[0].addrs, vec![4, 5, 6]);
    }

    /// Migração: um `peers.enc` já gravado ANTES da correção do upsert pode ter duplicatas
    /// persistidas no disco (mesmo `id.id`, `device_id` diferente). `open()` tem que limpar
    /// isso na leitura — sem isso, quem já pegou o bug continuaria vendo `pairedPeers` quebrar
    /// a UI mesmo depois de atualizar o app, porque a duplicata já estava salva antes do fix.
    #[tokio::test]
    async fn open_deduplicates_peers_already_persisted_before_the_fix() {
        let dir = tempfile::tempdir().unwrap();
        let key = [11u8; 32];

        let duplicated_on_disk = vec![
            PeerAddr {
                id: PeerIdentity { id: "peer-a".to_string(), device_id: Some("uuid-1".to_string()) },
                addrs: vec![1, 2, 3],
            },
            PeerAddr {
                id: PeerIdentity { id: "peer-a".to_string(), device_id: None },
                addrs: vec![4, 5, 6],
            },
        ];
        let json = serde_json::to_vec(&duplicated_on_disk).unwrap();
        std::fs::write(dir.path().join("peers.enc"), encrypt(&key, &json)).unwrap();

        let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
        let peers = storage.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1, "duplicata pré-existente deveria ser colapsada na leitura");
        assert_eq!(peers[0].addrs, vec![4, 5, 6], "mantém a última entrada, mesma regra do upsert");
    }

    #[tokio::test]
    async fn remove_peer_persists_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [12u8; 32];

        {
            let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
            storage.save_peer(&peer("peer-a")).await.unwrap();
            storage.save_peer(&peer("peer-b")).await.unwrap();

            storage.remove_peer("peer-a").await.unwrap();

            let peers = storage.load_peers().await.unwrap();
            assert_eq!(peers.len(), 1);
            assert_eq!(peers[0].id.id, "peer-b");
        }

        let reopened = SecureP2pStorage::open(dir.path(), key).unwrap();
        let peers = reopened.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id.id, "peer-b");
    }

    fn device_info(name: &str) -> DeviceInfo {
        DeviceInfo { name: name.to_string(), os: "windows".to_string(), version: "1.0.0".to_string() }
    }

    #[tokio::test]
    async fn device_info_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [21u8; 32];

        {
            let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
            storage
                .save_device_info(&peer("peer-a").id, &device_info("PC do Vinicius"))
                .await
                .unwrap();
        }

        // Reabre (simula restart do app) — o nome tem que vir do arquivo criptografado, não
        // só existir enquanto o processo estava de pé.
        let reopened = SecureP2pStorage::open(dir.path(), key).unwrap();
        let loaded = reopened.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0.id, "peer-a");
        assert_eq!(loaded[0].1.name, "PC do Vinicius");
    }

    #[tokio::test]
    async fn save_device_info_upserts_by_id_instead_of_duplicating() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [22u8; 32]).unwrap();

        storage.save_device_info(&peer("peer-a").id, &device_info("old-name")).await.unwrap();
        storage.save_device_info(&peer("peer-a").id, &device_info("new-name")).await.unwrap();

        let loaded = storage.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1.name, "new-name");
    }

    #[tokio::test]
    async fn remove_peer_also_forgets_its_persisted_device_name() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [23u8; 32]).unwrap();

        storage.save_peer(&peer("peer-a")).await.unwrap();
        storage.save_device_info(&peer("peer-a").id, &device_info("PC do Vinicius")).await.unwrap();

        storage.remove_peer("peer-a").await.unwrap();

        assert!(storage.load_device_info().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn remove_peer_of_unknown_id_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [13u8; 32]).unwrap();
        storage.save_peer(&peer("peer-a")).await.unwrap();

        storage.remove_peer("never-paired-peer").await.unwrap();

        assert_eq!(storage.load_peers().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn reopen_with_wrong_key_fails_loudly_instead_of_silently_dropping_peers() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [1u8; 32]).unwrap();
        storage.save_peer(&peer("secret-peer")).await.unwrap();

        // Chave errada (ex: keyring resolveu outra chave nessa execução) — decrypt falha.
        // Regressão: antes isso degradava pra uma lista vazia em silêncio, e a próxima
        // `save_peer` reescrevia `peers.enc` do zero, apagando pra sempre o que estava
        // salvo com a chave antiga. Agora tem que falhar alto no `open()`, não abrir com
        // um cache vazio como se nada tivesse acontecido.
        let reopen_result = SecureP2pStorage::open(dir.path(), [2u8; 32]);
        assert!(reopen_result.is_err(), "abrir com a chave errada deveria falhar, não degradar em silêncio");

        // E o arquivo original continua intacto no disco — nada foi sobrescrito.
        let reopened_with_correct_key = SecureP2pStorage::open(dir.path(), [1u8; 32]).unwrap();
        assert_eq!(reopened_with_correct_key.load_peers().await.unwrap().len(), 1);
    }
}
