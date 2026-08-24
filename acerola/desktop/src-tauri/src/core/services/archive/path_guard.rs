use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use crate::infra::error::PathError;

/// Gera um ID numérico determinístico de 64 bits a partir de um caminho de arquivo.
///
/// ### Racional de Design
/// O Acerola Desktop utiliza hashing de caminho em vez de IDs auto-incrementais do banco por três motivos:
/// 1. **Determinismo:** O ID de um quadrinho ou capítulo permanece o mesmo entre scans, mesmo se o banco for deletado.
/// 2. **Performance de Busca:** O SQLite realiza buscas por índices `INTEGER PRIMARY KEY` de forma muito mais rápida do que comparando strings de caminhos longos.
/// 3. **Desacoplamento e Paralelismo:** O scanner pode gerar IDs para entidades filhas (volumes/capítulos) sem precisar esperar a inserção do pai no banco para obter o ID gerado pelo SQLite.
pub fn path_hash(path: &Path) -> i64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);

    (hasher.finish() & 0x7fff_ffff_ffff_ffff) as i64
}

pub struct PathGuard {
    allowed_root: PathBuf,
}

impl PathGuard {
    pub fn new(root: PathBuf) -> Self {
        Self { allowed_root: root.canonicalize().unwrap_or(root) }
    }

    fn validate(&self, path: &Path) -> Result<(), PathError> {
        let canonical = path.canonicalize().map_err(|_| PathError::not_found(path))?;

        if !canonical.starts_with(&self.allowed_root) {
            return Err(PathError::access_denied(&canonical, &self.allowed_root));
        }

        Ok(())
    }

    pub fn execute<F, R, E>(&self, path: &Path, action: F) -> Result<R, PathError>
    where
        F: FnOnce(&Path) -> Result<R, E>,
        E: std::fmt::Display,
    {
        self.validate(path)?;
        action(path).map_err(|err: E| PathError::action_failed(path, err))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{path_hash, PathGuard};
    use crate::infra::error::PathError;

    #[test]
    fn test_path_hash_is_always_non_negative() {
        for i in 0..64 {
            let path = std::path::PathBuf::from(format!("/library/comic-{i}/chapter-{i}.cbz"));
            assert!(path_hash(&path) >= 0);
        }
    }

    #[test]
    fn test_path_hash_is_deterministic_and_distinguishes_paths() {
        let path = std::path::Path::new("/library/berserk/chapter-01.cbz");
        assert_eq!(path_hash(path), path_hash(path));
        assert_ne!(
            path_hash(path),
            path_hash(std::path::Path::new("/library/berserk/chapter-02.cbz"))
        );
    }

    #[test]
    fn test_valid_path_inside_root() {
        let root = tempdir().unwrap();
        let guard = PathGuard::new(root.path().to_path_buf());
        let file = root.path().join("arquivo.cbz");
        fs::write(&file, b"").unwrap();
        let result = guard.execute(&file, |_| -> Result<(), String> { Ok(()) });
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_outside_root_is_denied() {
        let root = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let guard = PathGuard::new(root.path().to_path_buf());
        let file = outside.path().join("arquivo.cbz");
        fs::write(&file, b"").unwrap();
        let result = guard.execute(&file, |_| -> Result<(), String> { Ok(()) });
        assert!(matches!(result, Err(PathError::AccessDenied)));
    }

    #[test]
    fn test_nonexistent_path_is_denied() {
        let root = tempdir().unwrap();
        let guard = PathGuard::new(root.path().to_path_buf());
        let fake = root.path().join("nao_existe.cbz");
        let result = guard.execute(&fake, |_| -> Result<(), String> { Ok(()) });
        assert!(matches!(result, Err(PathError::NotFound(_))));
    }

    #[test]
    fn test_path_traversal_is_denied() {
        let root = tempdir().unwrap();
        let guard = PathGuard::new(root.path().to_path_buf());
        let traversal = root.path().join("../arquivo_malicioso.cbz");
        let result = guard.execute(&traversal, |_| -> Result<(), String> { Ok(()) });
        assert!(matches!(result, Err(PathError::AccessDenied) | Err(PathError::NotFound(_))));
    }

    #[test]
    fn test_action_failure_is_propagated() {
        let root = tempdir().unwrap();
        let guard = PathGuard::new(root.path().to_path_buf());
        let file = root.path().join("arquivo.cbz");
        fs::write(&file, b"").unwrap();
        let result =
            guard.execute(&file, |_| -> Result<(), String> { Err("falha simulada".to_string()) });
        assert!(matches!(result, Err(PathError::ActionFailed(_))));
    }
}
