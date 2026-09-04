//! Criptografia em repouso pra tudo que o `acerola-p2p` persiste (identidade do nó,
//! cache de peers, lista de confiança TOFU) — ver [`p2p_storage`] e [`trusted_store`].
//!
//! Design: o keystore do SO (Credential Manager no Windows, Secret Service no Linux,
//! via crate `keyring`) protege só uma chave mestra AES-256 pequena, gerada uma vez.
//! Essa chave mestra criptografa (AES-256-GCM) todo o resto antes de tocar o disco —
//! não dá pra guardar listas que crescem direto no keyring do SO (não é feito pra
//! blobs grandes), mas dá pra guardar uma chave de 32 bytes tranquilamente.
//!
//! **Fallback sem keyring**: em Linux sem um provider de Secret Service rodando (ex:
//! Hyprland/sway sem gnome-keyring/kwallet configurado), a chave mestra cai pra um
//! arquivo local sem a proteção do SO — funciona, mas com segurança reduzida. Isso
//! NUNCA falha silenciosamente: [`get_or_create_master_key`] retorna de qual fonte a
//! chave veio, e o chamador (`bios::network::setup_network`) emite um evento pro
//! frontend avisar o usuário — mesmo princípio do VS Code quando não acha um keyring.

pub mod p2p_storage;
pub mod trusted_store;

use std::path::Path;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Key,
};
use rand::RngCore;

use crate::infra::error::ComicError;

const KEYRING_SERVICE: &str = "br.acerola.comic";
const KEYRING_USERNAME: &str = "p2p-master-key";
const MASTER_KEY_LEN: usize = 32;
/// Nome propositalmente explícito — se alguém ver esse arquivo no disco, o nome já
/// avisa que não está protegido pelo keyring do sistema.
const FALLBACK_KEY_FILE_NAME: &str = "master.key.insecure";

/// De onde a chave mestra efetivamente veio — o chamador usa isso pra decidir se
/// precisa avisar o usuário.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterKeySource {
    /// Protegida pelo keystore do SO (Credential Manager / Secret Service).
    Keyring,
    /// O SO não tem um backend de keyring utilizável agora — chave caiu pra um
    /// arquivo local sem essa camada extra de proteção.
    FallbackFile,
}

/// Obtém a chave mestra, tentando primeiro o keyring do SO e caindo pra um arquivo
/// local (`master.key.insecure` em `fallback_dir`) só se o keyring realmente não
/// estiver disponível — não confunde "ainda não existe entrada" (primeiro uso normal)
/// com "não tem backend nenhum" (aí sim precisa avisar o usuário).
pub fn get_or_create_master_key(
    fallback_dir: &Path,
) -> Result<([u8; MASTER_KEY_LEN], MasterKeySource), ComicError> {
    match try_keyring_master_key() {
        Ok(key) => Ok((key, MasterKeySource::Keyring)),
        Err(err) => {
            tracing::warn!(
                error = %err,
                "OS keyring unavailable, falling back to unencrypted local key file \
                 (identity/peers/trust will still be AES-encrypted with it, just without \
                 OS-level protection on the key itself)"
            );
            let key = get_or_create_fallback_master_key(fallback_dir)?;
            Ok((key, MasterKeySource::FallbackFile))
        },
    }
}

/// Só retorna `Ok` se a chave veio de verdade do keyring do SO. Qualquer falha real de
/// backend (não simplesmente "entrada não existe ainda") vira `Err`, sinalizando pro
/// chamador cair no fallback.
fn try_keyring_master_key() -> keyring::Result<[u8; MASTER_KEY_LEN]> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)?;

    match entry.get_secret() {
        Ok(bytes) if bytes.len() == MASTER_KEY_LEN => {
            let mut key = [0u8; MASTER_KEY_LEN];
            key.copy_from_slice(&bytes);
            Ok(key)
        },
        // Ausente (primeiro uso) ou corrompida/tamanho errado — gera e salva uma nova.
        // `set_secret` falhando aqui É um sinal real de backend indisponível.
        Ok(_) | Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; MASTER_KEY_LEN];
            rand::thread_rng().fill_bytes(&mut key);
            entry.set_secret(&key)?;
            Ok(key)
        },
        Err(other) => Err(other),
    }
}

fn get_or_create_fallback_master_key(dir: &Path) -> Result<[u8; MASTER_KEY_LEN], ComicError> {
    std::fs::create_dir_all(dir)
        .map_err(|err| ComicError::SystemFailure(format!("failed to create data dir: {err}")))?;
    let path = dir.join(FALLBACK_KEY_FILE_NAME);

    match std::fs::read(&path) {
        Ok(bytes) if bytes.len() == MASTER_KEY_LEN => {
            let mut key = [0u8; MASTER_KEY_LEN];
            key.copy_from_slice(&bytes);
            Ok(key)
        },
        _ => {
            let mut key = [0u8; MASTER_KEY_LEN];
            rand::thread_rng().fill_bytes(&mut key);
            std::fs::write(&path, key).map_err(|err| {
                ComicError::SystemFailure(format!("failed to save fallback master key: {err}"))
            })?;
            Ok(key)
        },
    }
}

/// Criptografa `plaintext` com AES-256-GCM. O nonce (96 bits, aleatório por chamada) é
/// prefixado no blob retornado — não precisa ser guardado à parte pra decriptar depois.
pub fn encrypt(key: &[u8; MASTER_KEY_LEN], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    // Só falha se o buffer for absurdamente grande (>2^36 bytes) — não é um caso real aqui.
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("AES-GCM encryption should not fail for small blobs");

    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    out
}

/// Decriptografa um blob produzido por [`encrypt`] (nonce prefixado + ciphertext).
pub fn decrypt(key: &[u8; MASTER_KEY_LEN], blob: &[u8]) -> Result<Vec<u8>, ComicError> {
    const NONCE_LEN: usize = 12;

    if blob.len() < NONCE_LEN {
        return Err(ComicError::SystemFailure(
            "encrypted blob too short to contain a nonce".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    cipher.decrypt(nonce_bytes.into(), ciphertext).map_err(|_| {
        ComicError::SystemFailure("failed to decrypt blob: wrong key or corrupted data".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip_returns_original_plaintext() {
        let key = [7u8; MASTER_KEY_LEN];
        let plaintext = b"acerola p2p identity seed material";

        let ciphertext = encrypt(&key, plaintext);
        assert_ne!(ciphertext, plaintext, "ciphertext should not equal plaintext");

        let decrypted = decrypt(&key, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_is_non_deterministic_due_to_random_nonce() {
        let key = [3u8; MASTER_KEY_LEN];
        let plaintext = b"same input, different ciphertext each time";

        let first = encrypt(&key, plaintext);
        let second = encrypt(&key, plaintext);

        assert_ne!(
            first, second,
            "two encryptions of the same plaintext must differ (random nonce)"
        );
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key_a = [1u8; MASTER_KEY_LEN];
        let key_b = [2u8; MASTER_KEY_LEN];
        let ciphertext = encrypt(&key_a, b"secret");

        assert!(decrypt(&key_b, &ciphertext).is_err());
    }

    #[test]
    fn decrypt_rejects_truncated_blob() {
        let key = [9u8; MASTER_KEY_LEN];
        assert!(decrypt(&key, b"short").is_err());
    }

    #[test]
    fn fallback_master_key_is_stable_across_calls() {
        let dir = tempfile::tempdir().unwrap();
        let first = get_or_create_fallback_master_key(dir.path()).unwrap();
        let second = get_or_create_fallback_master_key(dir.path()).unwrap();
        assert_eq!(first, second, "re-reading the fallback key file must return the same key");
    }
}
