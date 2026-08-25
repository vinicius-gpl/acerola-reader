use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use crate::{
    core::services::archive::{comic_scanner_engine::ComicScannerService, cover_extractor},
    data::{
        models::archive::{comic_directory::ComicDirectory, volume_archive::VolumeArchive},
        repositories::archive::{
            chapter_archive_repo::{ChapterRepository, ChapterSortCriteria},
            comic_directory_repo::ComicRepository,
            volume_archive_repo::VolumeRepository,
        },
    },
    infra::error::ComicError,
};

/// Serviço para operações em massa e individuais de quadrinhos.
pub struct ComicService {
    repo: ComicRepository,
    chapter_repo: ChapterRepository,
    volume_repo: VolumeRepository,
    pool: SqlitePool,
}

impl ComicService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: ComicRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool.clone()),
            volume_repo: VolumeRepository::new(pool.clone()),
            pool,
        }
    }

    /// Reescaneia pontualmente um único quadrinho a partir do seu path atual no disco.
    pub async fn rescan_comic(&self, id: i64) -> Result<(), ComicError> {
        let comic = self.repo.find_by_id(id).await.map_err(ComicError::from)?.ok_or(ComicError::NotFound)?;
        let scanner = ComicScannerService::new(PathBuf::from(&comic.path), self.pool.clone());

        scanner.rescan_comic(comic, |_| {}, |_| {}).await
    }

    /// Invalida e reescaneia um único quadrinho do zero (capítulos e volumes inclusos).
    pub async fn deep_rescan_comic(&self, id: i64) -> Result<(), ComicError> {
        let comic = self.repo.find_by_id(id).await.map_err(ComicError::from)?.ok_or(ComicError::NotFound)?;
        let scanner = ComicScannerService::new(PathBuf::from(&comic.path), self.pool.clone());

        scanner.deep_rescan_comic(comic, |_| {}, |_| {}).await
    }

    /// Atualiza o status de visibilidade de um quadrinho específico.
    pub async fn update_hidden_status(
        &self, id: i64, hidden: bool,
    ) -> Result<ComicDirectory, ComicError> {
        self.repo.update_hidden_status(id, hidden).await.map_err(ComicError::from)
    }

    /// Atualiza o status de sincronizacao externa de um quadrinho específico.
    pub async fn update_external_sync_enabled(
        &self, id: i64, external_sync_enabled: bool,
    ) -> Result<ComicDirectory, ComicError> {
        self.repo
            .update_external_sync_enabled(id, external_sync_enabled)
            .await
            .map_err(ComicError::from)
    }

    /// Atualiza o status de visibilidade de multiplos quadrinhos em batch.
    pub async fn update_hidden_status_batch(
        &self, ids: &[i64], hidden: bool,
    ) -> Result<usize, ComicError> {
        self.repo.update_hidden_status_batch(ids, hidden).await.map_err(ComicError::from)
    }

    /// Deleta multiplos quadrinhos do banco de dados.
    pub async fn delete_batch(&self, ids: &[i64]) -> Result<usize, ComicError> {
        self.repo.delete_batch(ids).await.map_err(ComicError::from)
    }

    /// Gera a capa de um quadrinho a partir da primeira página do seu primeiro capítulo
    /// (considerando a ordem de leitura, inclusive dentro de volumes), substituindo
    /// qualquer capa já definida.
    pub async fn regenerate_cover(&self, id: i64) -> Result<ComicDirectory, ComicError> {
        let comic = self.repo.find_by_id(id).await.map_err(ComicError::from)?.ok_or(ComicError::NotFound)?;

        let chapters = self
            .chapter_repo
            .get_chapters_by_directory(id, 1, 0, ChapterSortCriteria::NumberAsc)
            .await
            .map_err(ComicError::from)?;

        let first_chapter = chapters.into_iter().next().ok_or_else(|| {
            ComicError::SystemFailure("Comic has no chapters to generate a cover from.".to_string())
        })?;

        let cover_path =
            extract_and_persist_cover(PathBuf::from(first_chapter.path), Path::new(&comic.path))
                .await?;

        self.repo.update_cover(id, &cover_path).await.map_err(ComicError::from)
    }

    /// Gera a capa de cada volume do quadrinho a partir da primeira página do seu primeiro
    /// capítulo, substituindo qualquer capa já definida. Volumes sem capítulos são ignorados.
    pub async fn regenerate_volume_covers(&self, id: i64) -> Result<Vec<VolumeArchive>, ComicError> {
        self.repo.find_by_id(id).await.map_err(ComicError::from)?.ok_or(ComicError::NotFound)?;

        let volumes = self.volume_repo.find_by_comic(id).await.map_err(ComicError::from)?;

        let mut updated = Vec::new();
        for volume in volumes {
            let chapters = self
                .chapter_repo
                .get_chapters_by_volume(id, volume.id, 1, 0, ChapterSortCriteria::NumberAsc)
                .await
                .map_err(ComicError::from)?;

            let Some(first_chapter) = chapters.into_iter().next() else {
                continue;
            };

            let cover_path = extract_and_persist_cover(
                PathBuf::from(first_chapter.path),
                Path::new(&volume.path),
            )
            .await?;

            let saved = self.volume_repo.update_cover(volume.id, &cover_path).await.map_err(ComicError::from)?;
            updated.push(saved);
        }

        Ok(updated)
    }
}

/// Extrai a primeira página do arquivo de capítulo informado (em uma thread bloqueante) e
/// persiste como `cover.*` dentro de `target_dir`.
async fn extract_and_persist_cover(
    chapter_file: PathBuf, target_dir: &Path,
) -> Result<String, ComicError> {
    let (bytes, format) = tokio::task::spawn_blocking(move || cover_extractor::extract_first_page(&chapter_file))
        .await
        .map_err(|error| ComicError::SystemFailure(error.to_string()))??;

    cover_extractor::persist_cover(target_dir, bytes, format).await
}
