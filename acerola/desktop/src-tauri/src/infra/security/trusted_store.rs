use std::{collections::HashSet, path::PathBuf};

use acerola_p2p::api::guard::TrustedPeerStore;
use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{decrypt, encrypt};

/// Implementação de `TrustedPeerStore` (lista TOFU de confiança) com criptografia em
/// repouso — ver `infra::security` pro esquema. `trusted.enc` guarda os ids
/// confiáveis; `blocked.enc` os bloqueados (carregado uma vez, `TrustedPeerStore` não
/// expõe uma forma de escrever nele — mesma limitação que a implementação anterior).
pub struct SecureTrustedStore {
    trusted_path: PathBuf,
    master_key: [u8; 32],
    trusted: RwLock<HashSet<String>>,
    blocked: HashSet<String>,
}

impl SecureTrustedStore {
    pub fn open(base_dir: impl Into<PathBuf>, master_key: [u8; 32]) -> std::io::Result<Self> {
        let base_dir = base_dir.into();
        std::fs::create_dir_all(&base_dir)?;

        let trusted_path = base_dir.join("trusted.enc");
        let blocked_path = base_dir.join("blocked.enc");

        Ok(Self {
            trusted: RwLock::new(Self::read_set(&trusted_path, &master_key)?),
            blocked: Self::read_set(&blocked_path, &master_key)?,
            trusted_path,
            master_key,
        })
    }

    /// Diferente da versão anterior, decrypt/parse falhando vira `Err` — não pode virar
    /// "conjunto vazio" em silêncio, porque `insert` reescreve o arquivo inteiro com o que
    /// estiver em memória: um cache vazio por engano apaga toda a lista de confiança (TOFU)
    /// anterior assim que o próximo peer conhecido reconectar.
    fn read_set(path: &std::path::Path, master_key: &[u8; 32]) -> std::io::Result<HashSet<String>> {
        match std::fs::read(path) {
            Ok(encrypted) => {
                let json = decrypt(master_key, &encrypted).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to decrypt {} (wrong master key or corrupted file): {err}", path.display()),
                    )
                })?;
                serde_json::from_slice(&json).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to parse {}: {err}", path.display()),
                    )
                })
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(HashSet::new()),
            Err(err) => Err(err),
        }
    }

    fn write_set(&self, path: &std::path::Path, set: &HashSet<String>) -> std::io::Result<()> {
        let json = serde_json::to_vec(set).expect("HashSet<String> serialization cannot fail");
        std::fs::write(path, encrypt(&self.master_key, &json))
    }

    /// Remove um peer da lista de confiança (TOFU) — usado por "desparear". Idempotente: id
    /// desconhecido não é erro, só não reescreve o arquivo à toa.
    pub async fn remove(&self, id: &str) -> std::io::Result<()> {
        let mut trusted = self.trusted.write().await;
        if !trusted.remove(id) {
            return Ok(());
        }
        self.write_set(&self.trusted_path, &trusted)
    }
}

#[async_trait]
impl TrustedPeerStore for SecureTrustedStore {
    async fn insert(&self, id: &str) {
        let mut trusted = self.trusted.write().await;
        if trusted.contains(id) {
            return;
        }

        trusted.insert(id.to_string());
        if let Err(err) = self.write_set(&self.trusted_path, &trusted) {
            tracing::error!(layer = "security", error = ?err, "failed to persist trusted peer");
        }
    }

    async fn contains(&self, id: &str) -> bool {
        self.trusted.read().await.contains(id)
    }

    async fn is_blocked(&self, id: &str) -> bool {
        self.blocked.contains(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn insert_persists_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [4u8; 32];

        {
            let store = SecureTrustedStore::open(dir.path(), key).unwrap();
            assert!(!store.contains("peer-1").await);
            store.insert("peer-1").await;
            assert!(store.contains("peer-1").await);
        }

        let reopened = SecureTrustedStore::open(dir.path(), key).unwrap();
        assert!(reopened.contains("peer-1").await);

        let raw = std::fs::read(dir.path().join("trusted.enc")).unwrap();
        assert!(!raw.windows(b"peer-1".len()).any(|window| window == b"peer-1"));
    }

    #[tokio::test]
    async fn remove_persists_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [8u8; 32];

        {
            let store = SecureTrustedStore::open(dir.path(), key).unwrap();
            store.insert("peer-1").await;
            store.remove("peer-1").await.unwrap();
            assert!(!store.contains("peer-1").await);
        }

        let reopened = SecureTrustedStore::open(dir.path(), key).unwrap();
        assert!(!reopened.contains("peer-1").await);
    }

    #[tokio::test]
    async fn remove_of_unknown_peer_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let store = SecureTrustedStore::open(dir.path(), [3u8; 32]).unwrap();
        store.insert("peer-1").await;

        store.remove("never-trusted-peer").await.unwrap();

        assert!(store.contains("peer-1").await);
    }

    #[tokio::test]
    async fn insert_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let store = SecureTrustedStore::open(dir.path(), [6u8; 32]).unwrap();
        store.insert("peer-x").await;
        store.insert("peer-x").await;
        assert!(store.contains("peer-x").await);
    }

    #[tokio::test]
    async fn reopen_with_wrong_key_fails_loudly_instead_of_silently_dropping_trust() {
        let dir = tempfile::tempdir().unwrap();
        let store = SecureTrustedStore::open(dir.path(), [1u8; 32]).unwrap();
        store.insert("trusted-peer").await;

        // Regressão: chave errada (keyring resolveu outra chave nessa execução) tinha que
        // falhar aqui, não abrir com uma lista de confiança vazia como se ninguém nunca
        // tivesse sido aprovado — TOFU voltaria a "confiar automaticamente" em qualquer um
        // de novo, e o próximo `insert` apagaria a lista antiga do disco.
        let reopen_result = SecureTrustedStore::open(dir.path(), [2u8; 32]);
        assert!(reopen_result.is_err(), "abrir com a chave errada deveria falhar, não degradar em silêncio");

        let reopened_with_correct_key = SecureTrustedStore::open(dir.path(), [1u8; 32]).unwrap();
        assert!(reopened_with_correct_key.contains("trusted-peer").await);
    }
}
