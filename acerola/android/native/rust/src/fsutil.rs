use std::{io, path::Path};

/// Lê `path` inteiro, retornando `None` se o arquivo simplesmente não existe. Só usado hoje
/// pra migração dos arquivos legados em texto puro (`identity.seed`/`peers.json`/
/// `trusted.txt`/`blocked.txt`) pro `SecureBlobStore` — ver `storage.rs`/`trust_store.rs`.
pub(crate) fn read_optional(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}
