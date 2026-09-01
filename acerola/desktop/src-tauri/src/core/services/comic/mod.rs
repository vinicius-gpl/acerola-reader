use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use crate::{
    core::services::archive::{comic_scanner_engine::ComicScannerService, cover_extractor},
    data::{
        models::archive::{comic_directory::ComicDirectory, volume_archive::VolumeArchive},
        repositories::{
            archive::{
                chapter_archive_repo::{ChapterRepository, ChapterSortCriteria},
                comic_directory_repo::ComicRepository,
                volume_archive_repo::VolumeRepository,
            },
            category::CategoryRepository,
            history::{chapter_read_repo::ChapterReadRepository, reading_history_repo::ReadingHistoryRepository},
            metadata::MetadataRepository,
        },
    },
    infra::error::ComicError,
};

/// Serviço para operações em massa e individuais de quadrinhos.
pub struct ComicService {
    repo: ComicRepository,
    chapter_repo: ChapterRepository,
    volume_repo: VolumeRepository,
    metadata_repo: MetadataRepository,
    category_repo: CategoryRepository,
    reading_history_repo: ReadingHistoryRepository,
    chapter_read_repo: ChapterReadRepository,
    pool: SqlitePool,
}

impl ComicService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: ComicRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool.clone()),
            volume_repo: VolumeRepository::new(pool.clone()),
            metadata_repo: MetadataRepository::new(pool.clone()),
            category_repo: CategoryRepository::new(pool.clone()),
            reading_history_repo: ReadingHistoryRepository::new(pool.clone()),
            chapter_read_repo: ChapterReadRepository::new(pool.clone()),
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

    /// Deleta múltiplos quadrinhos: a pasta física no disco (recursivamente) e todas as
    /// linhas relacionadas no banco, incluindo o registro em `comic_directory` em si.
    ///
    /// A remoção em disco é melhor esforço, por quadrinho — uma pasta já ausente ou
    /// bloqueada não impede a exclusão dos demais nem do registro no banco (mesma postura
    /// de `MetadataService::clear_comic_metadata`).
    ///
    /// A limpeza das tabelas filhas é manual, na ordem de dependência (filhas antes das
    /// pais), porque o cascade de FK declarado nas migrations não roda em produção — o pool
    /// nunca habilita `PRAGMA foreign_keys` (mesmo motivo documentado em
    /// `VolumeRepository::delete_by_comic`). Sem essa limpeza, as linhas órfãs ficam presas
    /// ao `id` do quadrinho excluído — e como esse `id` é hash do path
    /// (`path_hash`), um quadrinho novo criado depois no mesmo caminho herdaria em
    /// silêncio o lixo (capítulos, histórico, metadata) do quadrinho antigo.
    pub async fn delete_batch(&self, ids: &[i64]) -> Result<usize, ComicError> {
        if ids.is_empty() {
            return Ok(0);
        }

        for &id in ids {
            self.remove_comic_folder_best_effort(id).await;

            self.reading_history_repo.delete_by_comic_id(id).await.ok();
            self.chapter_read_repo.delete_by_comic(id).await.ok();
            self.metadata_repo.delete_by_comic_id(id).await.ok();
            self.chapter_repo.delete_by_comic(id).await.ok();
            self.volume_repo.delete_by_comic(id).await.ok();
            self.category_repo.remove_category_from_comic(id).await.ok();
        }

        self.repo.delete_batch(ids).await.map_err(ComicError::from)
    }

    /// Remove a pasta física de um quadrinho do disco — melhor esforço: um `id` sem
    /// registro correspondente (não deveria acontecer) ou uma pasta que não pode ser
    /// removida (já ausente, permissão negada) não impede a exclusão do restante em
    /// `delete_batch`, só loga e segue. Todo caminho de saída loga algo — nenhuma falha
    /// fica silenciosa (era exatamente esse o sintoma reportado: pasta não sumia e não
    /// aparecia nenhum log explicando por quê).
    async fn remove_comic_folder_best_effort(&self, id: i64) {
        let comic = match self.repo.find_by_id(id).await {
            Ok(Some(comic)) => comic,
            Ok(None) => {
                tracing::warn!(comic_id = id, "Comic not found in database while deleting its folder");
                return;
            },
            Err(error) => {
                tracing::warn!(comic_id = id, error = %error, "Failed to look up comic before deleting its folder");
                return;
            },
        };

        tracing::info!(comic_id = id, path = %comic.path, "Removing comic folder from disk");

        match tokio::fs::remove_dir_all(&comic.path).await {
            Ok(()) => {
                tracing::info!(comic_id = id, path = %comic.path, "Comic folder removed from disk");
            },
            Err(error) => {
                tracing::warn!(
                    comic_id = id,
                    path = %comic.path,
                    error = %error,
                    "Failed to remove comic folder from disk while deleting comic"
                );
            },
        }
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

#[cfg(test)]
mod tests {
    use super::ComicService;
    use crate::tests::utils::setup_test_db::{
        insert_chapter_archive, insert_chapter_read, insert_comic_directory, insert_reading_history,
        setup_test_db,
    };

    /// Prova o próprio motivo desta mudança: excluir um quadrinho tinha que apagar a pasta
    /// física, não só a linha no banco (era só `DELETE FROM comic_directory`, arquivo em
    /// disco ficava intocado).
    #[tokio::test]
    async fn delete_batch_removes_the_physical_folder_from_disk() {
        let pool = setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let comic_dir = temp_dir.path().join("Quadrinho Para Excluir");
        tokio::fs::create_dir_all(&comic_dir).await.unwrap();
        tokio::fs::write(comic_dir.join("Cap 1.cbz"), b"fake cbz").await.unwrap();

        insert_comic_directory(&pool, 1, "Quadrinho Para Excluir", &comic_dir.to_string_lossy()).await;

        let service = ComicService::new(pool);
        let deleted = service.delete_batch(&[1]).await.unwrap();

        assert_eq!(deleted, 1);
        assert!(!comic_dir.exists(), "pasta física deveria ter sido removida");
    }

    /// Uma pasta já ausente (usuário apagou manualmente, ou path desatualizado) não pode
    /// impedir a exclusão do registro no banco — mesma postura best-effort de
    /// `MetadataService::clear_comic_metadata`.
    #[tokio::test]
    async fn delete_batch_still_removes_db_row_when_folder_is_already_gone() {
        let pool = setup_test_db().await;
        insert_comic_directory(&pool, 1, "Quadrinho Fantasma", "/caminho/que/nao/existe").await;

        let service = ComicService::new(pool.clone());
        let deleted = service.delete_batch(&[1]).await.unwrap();

        assert_eq!(deleted, 1);
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comic_directory").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 0);
    }

    /// Regressão do bug de linha órfã: como o cascade de FK não roda em produção, excluir um
    /// quadrinho sem limpar as tabelas filhas manualmente deixava capítulo/histórico/leitura
    /// presos ao `id` antigo — e como esse `id` é hash do path, um quadrinho novo criado
    /// depois no mesmo caminho herdaria esse lixo em silêncio.
    #[tokio::test]
    async fn delete_batch_cleans_up_orphaned_child_rows() {
        let pool = setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let comic_dir = temp_dir.path().join("Quadrinho Com Historico");
        tokio::fs::create_dir_all(&comic_dir).await.unwrap();

        insert_comic_directory(&pool, 1, "Quadrinho Com Historico", &comic_dir.to_string_lossy()).await;
        insert_chapter_archive(&pool, 1, 1).await;
        insert_reading_history(&pool, 1, 1, 5).await;
        insert_chapter_read(&pool, 1, 1).await;

        let service = ComicService::new(pool.clone());
        service.delete_batch(&[1]).await.unwrap();

        let chapters: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chapter_archive").fetch_one(&pool).await.unwrap();
        let history: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM reading_history").fetch_one(&pool).await.unwrap();
        let read: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chapter_read").fetch_one(&pool).await.unwrap();

        assert_eq!(chapters.0, 0, "capítulo órfão não foi limpo");
        assert_eq!(history.0, 0, "histórico de leitura órfão não foi limpo");
        assert_eq!(read.0, 0, "marcador de lido órfão não foi limpo");
    }
}
