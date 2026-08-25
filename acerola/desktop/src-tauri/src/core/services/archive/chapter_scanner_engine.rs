use std::path::Path;

use tokio::fs;

use crate::{
    core::services::archive::path_guard::path_hash,
    data::{
        models::archive::chapter_archive::{is_special_name, ChapterArchive},
        repositories::archive::chapter_archive_repo::ChapterRepository,
    },
    infra::{
        error::{ComicError, DbError},
        pattern::{template::extract_chapter_parts, template_validator::validate_chapter_template},
    },
};

/// Responsável por indexar capítulos individuais no banco de dados.
///
/// Recebe um arquivo de capítulo, extrai seus metadados e o persiste.
/// Chamado pelo [`super::comic_scanner_engine::ComicScannerService`] durante qualquer
/// forma de scan de biblioteca.
pub struct ChapterScannerService {
    chapter_repo: ChapterRepository,
}

impl ChapterScannerService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { chapter_repo: ChapterRepository::new(pool) }
    }

    /// Remove todos os capítulos indexados de um quadrinho. Usado pelo rescan profundo
    /// para invalidar o estado atual antes de re-escanear a pasta do zero.
    pub async fn delete_by_comic(&self, comic_id: i64) -> Result<(), ComicError> {
        self.chapter_repo.delete_by_comic(comic_id).await?;
        Ok(())
    }

    /// Indexa um único arquivo de capítulo no banco de dados.
    ///
    /// Extrai nome, hash rápido (`nome|tamanho|modificado`) e `chapter_sort` a partir do
    /// template detectado. Se nenhum template for fornecido, usa [`ChapterArchive::fallback_sort`].
    ///
    /// O checksum só é (re)calculado no scan — nunca no sync — e só quando necessário:
    /// capítulo novo, capítulo com `last_modified` alterado, ou capítulo já indexado mas
    /// ainda sem checksum (backfill de bibliotecas escaneadas antes dessa lógica existir).
    /// Um capítulo inalterado com checksum já em cache nem toca o banco de novo.
    #[rustfmt::skip]
    pub async fn scan_chapter(
        &self,
        file: &Path,
        index: usize,
        comic_id: i64,
        volume_id: Option<i64>,
        template: Option<&str>,
    ) -> Result<(), ComicError> {
        let meta = fs::metadata(file).await?;

        let file_name = file
            .file_name()
            .and_then(|it| it.to_str())
            .ok_or_else(|| ComicError::SystemFailure("File name is invalid".into()))?;

        let file_modified = modified_secs(&meta);
        let id = path_hash(file);

        let existing = self.chapter_repo.find_by_id(id).await?;

        if let Some(row) = &existing {
            if row.last_modified == file_modified && row.checksum.is_some() {
                return Ok(());
            }
        }

        let chapter_name = file.file_stem().and_then(|it| it.to_str()).unwrap_or("unknown").to_string();

        let chapter_sort = template
            .and_then(|template| {
                extract_chapter_parts(file_name, template, validate_chapter_template)
            })
            .map(|(chapter, decimal)| ChapterArchive::format_sort(chapter, decimal))
            .unwrap_or_else(|| ChapterArchive::fallback_sort(&chapter_name, index));

        let checksum = compute_checksum(file).await.ok();

        let chapter = ChapterArchive {
            id,
            chapter: chapter_name.clone(),
            path: file.to_string_lossy().to_string(),
            chapter_sort,
            is_special: is_special_name(&chapter_name),
            checksum,
            comic_directory_fk: comic_id,
            volume_fk: volume_id,
            last_modified: file_modified,
        };

        if existing.is_some() {
            self.chapter_repo.base.update(&chapter).await?;
            return Ok(());
        }

        match self.chapter_repo.base.insert(&chapter).await {
            Ok(_) => {}
            Err(DbError::UniqueViolation) => {
                log::debug!(
                    "[Scanner] Chapter '{}' already indexed, skipping.",
                    chapter.chapter
                );
            }
            Err(err) => return Err(err.into()),
        }

        Ok(())
    }
}

#[rustfmt::skip]
fn modified_secs(meta: &std::fs::Metadata) -> i64 {
    meta.modified().map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64).unwrap_or(0)
}

/// Calcula o SHA-256 do arquivo em streaming, em buffer de 8KB (sem carregar o cbz/cbr
/// inteiro em memória), rodando em `spawn_blocking` pois é I/O síncrono de CPU-bound hashing.
/// Mesmo algoritmo e formato de saída do Android (`FileHash.kt::sha256()`), pra bater
/// string-a-string entre os dois lados do protocolo P2P: hex minúsculo, sem separador.
///
/// Usado tanto pra verificar integridade após transferências P2P quanto pra dedup best-effort.
async fn compute_checksum(path: &Path) -> Result<String, ComicError> {
    use std::io::Read;

    use sha2::{Digest, Sha256};

    let owned_path = path.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut file = std::fs::File::open(&owned_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }

        Ok::<String, std::io::Error>(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|join_error| ComicError::SystemFailure(format!("Checksum task panicked: {join_error}")))?
    .map_err(ComicError::Io)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;
    use tokio::fs;

    use super::ChapterScannerService;
    use crate::{
        data::repositories::archive::chapter_archive_repo::ChapterRepository,
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    async fn setup() -> (ChapterScannerService, sqlx::SqlitePool, TempDir) {
        let pool = setup_test_db_with_comic().await;
        let service = ChapterScannerService::new(pool.clone());
        let dir = tempfile::tempdir().unwrap();
        (service, pool, dir)
    }

    async fn create_file(dir: &TempDir, name: &str) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, b"fake cbz content").await.unwrap();
        path
    }

    fn chapter_repo(pool: &sqlx::SqlitePool) -> ChapterRepository {
        ChapterRepository::new(pool.clone())
    }

    #[tokio::test]
    async fn scan_chapter_inserts_into_database() {
        let (service, pool, dir) = setup().await;
        let file = create_file(&dir, "Ch. 1.cbz").await;
        service.scan_chapter(&file, 0, 1, None, None).await.unwrap();
        let all = chapter_repo(&pool).base.find_all().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].chapter, "Ch. 1");
    }

    #[tokio::test]
    async fn scan_chapter_with_template_generates_correct_sort() {
        let (service, pool, dir) = setup().await;
        let file = create_file(&dir, "Ch. 10.cbz").await;
        service
            .scan_chapter(&file, 0, 1, None, Some("Ch. {chapter}{decimal}.*.{extension}"))
            .await
            .unwrap();
        let all = chapter_repo(&pool).base.find_all().await.unwrap();
        assert_eq!(all[0].chapter_sort, "10");
    }

    #[tokio::test]
    async fn scan_chapter_duplicate_is_ignored() {
        let (service, pool, dir) = setup().await;
        let file = create_file(&dir, "Ch. 1.cbz").await;
        service.scan_chapter(&file, 0, 1, None, None).await.unwrap();
        service.scan_chapter(&file, 0, 1, None, None).await.unwrap();
        assert_eq!(chapter_repo(&pool).base.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn scan_chapter_nonexistent_file_returns_error() {
        let (service, _, _) = setup().await;
        let fake = PathBuf::from("/nao/existe/Ch. 1.cbz");
        assert!(service.scan_chapter(&fake, 0, 1, None, None).await.is_err());
    }
}
