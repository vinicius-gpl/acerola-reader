use std::collections::HashMap;

use sqlx::{Row, SqlitePool};

use crate::{
    data::{
        models::{
            archive::chapter_archive::ChapterArchive,
            relations::{
                chapter_with_comic::ChapterArchiveWithComic,
                chapter_with_volume::ChapterArchiveWithVolume,
                library_summary_row::LibrarySummaryRow,
            },
        },
        repositories::Repository,
    },
    infra::error::DbError,
};

#[derive(Debug, Clone, Copy, Default)]
pub enum ChapterSortCriteria {
    #[default]
    NumberAsc,
    NumberDesc,
    ModifiedAsc,
    ModifiedDesc,
}

#[derive(Clone)]
pub struct ChapterRepository {
    pub base: Repository<ChapterArchive>,
    pool: SqlitePool,
}

impl ChapterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    pub async fn get_all_counts(&self) -> Result<HashMap<i64, i64>, DbError> {
        let counts = sqlx::query("SELECT comic_directory_fk, COUNT(*) as count FROM chapter_archive GROUP BY comic_directory_fk")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| (row.get::<i64, _>("comic_directory_fk"), row.get::<i64, _>("count")))
            .collect();

        Ok(counts)
    }

    pub async fn count_by_directory_id(&self, comic_directory_fk: i64) -> Result<i64, DbError> {
        let result =
            sqlx::query("SELECT COUNT(*) FROM chapter_archive WHERE comic_directory_fk = ?")
                .bind(comic_directory_fk)
                .fetch_one(&self.pool)
                .await?;
        Ok(result.get(0))
    }

    pub async fn count_by_directory_id_with_search(
        &self, comic_directory_fk: i64, search_query: &str,
    ) -> Result<i64, DbError> {
        let search_pattern = format!("%{}%", search_query);
        let result = sqlx::query(
            "SELECT COUNT(*) FROM chapter_archive WHERE comic_directory_fk = ? AND chapter LIKE ?",
        )
        .bind(comic_directory_fk)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.get(0))
    }

    pub async fn count_root_chapters(&self, comic_directory_fk: i64) -> Result<i64, DbError> {
        let result = sqlx::query("SELECT COUNT(*) FROM chapter_archive WHERE comic_directory_fk = ? AND volume_fk IS NULL")
            .bind(comic_directory_fk)
            .fetch_one(&self.pool)
            .await?;
        Ok(result.get(0))
    }

    pub async fn get_total_count_by_volume(&self, volume_fk: i64) -> Result<i64, DbError> {
        let result = sqlx::query("SELECT COUNT(*) FROM chapter_archive WHERE volume_fk = ?")
            .bind(volume_fk)
            .fetch_one(&self.pool)
            .await?;
        Ok(result.get(0))
    }

    fn order_clause(criteria: ChapterSortCriteria) -> String {
        match criteria {
            ChapterSortCriteria::NumberAsc => "ORDER BY CAST(ca.chapter_sort AS INTEGER) ASC, CAST(CASE WHEN ca.chapter_sort LIKE '%.%' THEN SUBSTR(ca.chapter_sort, INSTR(ca.chapter_sort, '.') + 1) ELSE 0 END AS INTEGER) ASC, (ca.is_special OR COALESCE(va.is_special, 0)) ASC".to_string(),
            ChapterSortCriteria::NumberDesc => "ORDER BY CAST(ca.chapter_sort AS INTEGER) DESC, CAST(CASE WHEN ca.chapter_sort LIKE '%.%' THEN SUBSTR(ca.chapter_sort, INSTR(ca.chapter_sort, '.') + 1) ELSE 0 END AS INTEGER) DESC, (ca.is_special OR COALESCE(va.is_special, 0)) ASC".to_string(),
            ChapterSortCriteria::ModifiedAsc => "ORDER BY ca.last_modified ASC".to_string(),
            ChapterSortCriteria::ModifiedDesc => "ORDER BY ca.last_modified DESC".to_string(),
        }
    }

    pub async fn get_chapters_by_directory(
        &self, comic_directory_fk: i64, page_size: i64, offset: i64, criteria: ChapterSortCriteria,
    ) -> Result<Vec<ChapterArchiveWithVolume>, DbError> {
        let order = Self::order_clause(criteria);
        let sql = format!(
            "SELECT ca.*, va.name AS volume_name, va.volume_sort, va.is_special AS volume_is_special FROM chapter_archive ca LEFT JOIN volume_archive va ON ca.volume_fk = va.id WHERE ca.comic_directory_fk = ? {} LIMIT ? OFFSET ?",
            order
        );

        sqlx::query_as::<_, ChapterArchiveWithVolume>(&sql)
            .bind(comic_directory_fk)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn get_chapters_by_directory_with_search(
        &self, comic_directory_fk: i64, page_size: i64, offset: i64, search_query: &str,
        criteria: ChapterSortCriteria,
    ) -> Result<Vec<ChapterArchiveWithVolume>, DbError> {
        let search_pattern = format!("%{}%", search_query);
        let order = Self::order_clause(criteria);
        let sql = format!(
            "SELECT ca.*, va.name AS volume_name, va.volume_sort, va.is_special AS volume_is_special FROM chapter_archive ca LEFT JOIN volume_archive va ON ca.volume_fk = va.id WHERE ca.comic_directory_fk = ? AND ca.chapter LIKE ? {} LIMIT ? OFFSET ?",
            order
        );

        sqlx::query_as::<_, ChapterArchiveWithVolume>(&sql)
            .bind(comic_directory_fk)
            .bind(&search_pattern)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    /// Remove todos os capítulos vinculados a um quadrinho específico.
    ///
    /// Usado pelo rescan profundo, já que o cascade de FK não roda em runtime
    /// (`PRAGMA foreign_keys` não é habilitado no pool de produção).
    pub async fn delete_by_comic(&self, comic_directory_fk: i64) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM chapter_archive WHERE comic_directory_fk = ?")
            .bind(comic_directory_fk)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Retorna todos os IDs de capítulo de um quadrinho, sem paginação.
    pub async fn find_all_ids_by_directory(
        &self, comic_directory_fk: i64,
    ) -> Result<Vec<i64>, DbError> {
        let rows = sqlx::query_as::<_, (i64,)>(
            "SELECT id FROM chapter_archive WHERE comic_directory_fk = ?",
        )
        .bind(comic_directory_fk)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// Busca um capítulo pelo id determinístico (`path_hash` do arquivo) — usado pelo
    /// scanner pra decidir, antes de tocar o disco, se um capítulo é novo, mudou
    /// (`last_modified` diferente) ou só precisa de backfill de checksum.
    pub async fn find_by_id(&self, id: i64) -> Result<Option<ChapterArchive>, DbError> {
        let result = sqlx::query_as::<_, ChapterArchive>(
            "SELECT id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, volume_fk, last_modified
             FROM chapter_archive WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Busca um capítulo pela chave natural (nome do quadrinho + rótulo do capítulo),
    /// já garantida única pelo `UNIQUE(comic_directory_fk, chapter)` da tabela. Usado pelo
    /// sync P2P pra resolver entradas recebidas de outro device, que não compartilha os
    /// mesmos IDs autoincrement locais.
    pub async fn find_by_comic_and_chapter(
        &self, comic_directory_fk: i64, chapter: &str,
    ) -> Result<Option<ChapterArchive>, DbError> {
        let result = sqlx::query_as::<_, ChapterArchive>(
            "SELECT id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, volume_fk, last_modified
             FROM chapter_archive WHERE comic_directory_fk = ? AND chapter = ?",
        )
        .bind(comic_directory_fk)
        .bind(chapter)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Busca um capítulo por `chapter_sort` em vez do rótulo — usado pelo sync P2P de
    /// **histórico**, não de arquivos. O rótulo (`chapter`) não é comparável entre devices
    /// (cada um nomeia o arquivo como quiser), mas `chapter_sort` é derivado do mesmo jeito
    /// nos dois apps a partir do nome, então funciona como chave natural cross-device — é
    /// também o que o Android usa pra resolver `chapter_archive_id` depois de receber um
    /// manifesto (via `updateHistoryChapterIdBySort`).
    pub async fn find_by_comic_and_chapter_sort(
        &self, comic_directory_fk: i64, chapter_sort: &str,
    ) -> Result<Option<ChapterArchive>, DbError> {
        let result = sqlx::query_as::<_, ChapterArchive>(
            "SELECT id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, volume_fk, last_modified
             FROM chapter_archive WHERE comic_directory_fk = ? AND chapter_sort = ?",
        )
        .bind(comic_directory_fk)
        .bind(chapter_sort)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Retorna todos os capítulos indexados junto do nome do quadrinho, usado pelo sync de
    /// arquivos pra montar o manifesto local sem depender de N+1 queries por quadrinho.
    /// Só SQL, de propósito — nenhum `tokio::fs::metadata` por capítulo. Usada por
    /// `browse-library` (P2P) pra listar título + contagem de capítulos + versão de capa sem
    /// pagar o custo de I/O de disco por capítulo, que já estourou o timeout do protocolo numa
    /// biblioteca grande (ver `FileSyncService::build_manifest`, usado até então). `cover_version`
    /// reaproveita `comic_directory.last_modified`, já existente — sem hash novo.
    pub async fn get_library_summary(&self) -> Result<Vec<LibrarySummaryRow>, DbError> {
        let result = sqlx::query_as::<_, LibrarySummaryRow>(
            "SELECT cd.name AS comic_name, COUNT(ca.id) AS chapter_count, cd.last_modified AS cover_version
             FROM comic_directory cd
             JOIN chapter_archive ca ON ca.comic_directory_fk = cd.id
             GROUP BY cd.name
             ORDER BY cd.name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_all_with_comic_name(&self) -> Result<Vec<ChapterArchiveWithComic>, DbError> {
        let result = sqlx::query_as::<_, ChapterArchiveWithComic>(
            "SELECT ca.id, ca.chapter, ca.path, ca.checksum, ca.last_modified, c.name AS comic_name
             FROM chapter_archive ca
             JOIN comic_directory c ON ca.comic_directory_fk = c.id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_chapters_by_volume(
        &self, comic_directory_fk: i64, volume_fk: i64, page_size: i64, offset: i64,
        criteria: ChapterSortCriteria,
    ) -> Result<Vec<ChapterArchiveWithVolume>, DbError> {
        let order = Self::order_clause(criteria);
        let sql = format!(
            "SELECT ca.*, va.name AS volume_name, va.volume_sort, va.is_special AS volume_is_special FROM chapter_archive ca LEFT JOIN volume_archive va ON ca.volume_fk = va.id WHERE ca.comic_directory_fk = ? AND ca.volume_fk = ? {} LIMIT ? OFFSET ?",
            order
        );

        sqlx::query_as::<_, ChapterArchiveWithVolume>(&sql)
            .bind(comic_directory_fk)
            .bind(volume_fk)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{ChapterArchive, ChapterRepository, ChapterSortCriteria};
    use crate::{
        infra::error::DbError,
        tests::utils::setup_test_db::{setup_test_db_with_comic, setup_test_db_with_volumes},
    };

    fn chapter(id: i64, chapter_sort: &str) -> ChapterArchive {
        ChapterArchive {
            id,
            chapter: format!("Capítulo {}", id),
            path: format!("/quadrinhos/berserk/cap{}", id),
            chapter_sort: chapter_sort.to_string(),
            is_special: false,
            checksum: None,
            comic_directory_fk: 1,
            volume_fk: None,
            last_modified: 123456789,
        }
    }

    #[tokio::test]
    async fn test_insert_and_find_all() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        let inserted = repo.base.insert(&chapter(1, "001")).await.unwrap();

        assert_eq!(inserted.id, 1);
        assert_eq!(inserted.chapter, "Capítulo 1");

        let all = repo.base.find_all().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_counts() {
        let pool = setup_test_db_with_volumes().await;
        let repo = ChapterRepository::new(pool);
        repo.base.insert(&chapter(1, "1")).await.unwrap();
        repo.base.insert(&chapter(2, "2")).await.unwrap();
        repo.base
            .insert(&ChapterArchive { id: 3, volume_fk: Some(1), ..chapter(3, "3") })
            .await
            .unwrap();

        assert_eq!(repo.count_by_directory_id(1).await.unwrap(), 3);
        assert_eq!(repo.count_root_chapters(1).await.unwrap(), 2);
        assert_eq!(repo.get_total_count_by_volume(1).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_sorting_by_number_asc() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        repo.base.insert(&chapter(1, "0.9")).await.unwrap();
        repo.base.insert(&chapter(2, "0.10")).await.unwrap();
        repo.base.insert(&chapter(3, "1.0")).await.unwrap();
        repo.base.insert(&chapter(4, "0.1")).await.unwrap();

        let result =
            repo.get_chapters_by_directory(1, 10, 0, ChapterSortCriteria::NumberAsc).await.unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].chapter_sort, "0.1");
        assert_eq!(result[1].chapter_sort, "0.9");
        assert_eq!(result[2].chapter_sort, "0.10");
        assert_eq!(result[3].chapter_sort, "1.0");
    }

    #[tokio::test]
    async fn test_sorting_by_number_desc() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        repo.base.insert(&chapter(1, "0.9")).await.unwrap();
        repo.base.insert(&chapter(2, "0.10")).await.unwrap();
        repo.base.insert(&chapter(3, "1.0")).await.unwrap();
        repo.base.insert(&chapter(4, "0.1")).await.unwrap();

        let result = repo
            .get_chapters_by_directory(1, 10, 0, ChapterSortCriteria::NumberDesc)
            .await
            .unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].chapter_sort, "1.0");
        assert_eq!(result[1].chapter_sort, "0.10");
        assert_eq!(result[2].chapter_sort, "0.9");
        assert_eq!(result[3].chapter_sort, "0.1");
    }

    #[tokio::test]
    async fn test_sorting_by_modified_asc() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        let mut c1 = chapter(1, "1");
        c1.last_modified = 300;
        let mut c2 = chapter(2, "2");
        c2.last_modified = 100;
        let mut c3 = chapter(3, "3");
        c3.last_modified = 200;

        repo.base.insert(&c1).await.unwrap();
        repo.base.insert(&c2).await.unwrap();
        repo.base.insert(&c3).await.unwrap();

        let result = repo
            .get_chapters_by_directory(1, 10, 0, ChapterSortCriteria::ModifiedAsc)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].last_modified, 100);
        assert_eq!(result[1].last_modified, 200);
        assert_eq!(result[2].last_modified, 300);
    }

    #[tokio::test]
    async fn test_sorting_by_modified_desc() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        let mut c1 = chapter(1, "1");
        c1.last_modified = 300;
        let mut c2 = chapter(2, "2");
        c2.last_modified = 100;
        let mut c3 = chapter(3, "3");
        c3.last_modified = 200;

        repo.base.insert(&c1).await.unwrap();
        repo.base.insert(&c2).await.unwrap();
        repo.base.insert(&c3).await.unwrap();

        let result = repo
            .get_chapters_by_directory(1, 10, 0, ChapterSortCriteria::ModifiedDesc)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].last_modified, 300);
        assert_eq!(result[1].last_modified, 200);
        assert_eq!(result[2].last_modified, 100);
    }

    #[tokio::test]
    async fn test_error_on_duplicate_insert() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        repo.base.insert(&chapter(1, "001")).await.unwrap();
        let result = repo.base.insert(&chapter(1, "001")).await;

        assert!(
            matches!(result, Err(DbError::UniqueViolation)),
            "Deveria ter retornado UniqueViolation, mas veio: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_error_on_updating_nonexistent() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        let result = repo.base.update(&chapter(999, "001")).await;

        assert!(
            matches!(result, Err(DbError::NotFound)),
            "Deveria ter retornado NotFound, mas veio: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_delete_by_comic_removes_only_chapters_from_that_comic() {
        let pool = setup_test_db_with_comic().await;
        sqlx::query(
            "INSERT INTO comic_directory (id, name, path, last_modified, external_sync_enabled, hidden)
             VALUES (2, 'Other', '/other', 0, 0, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = ChapterRepository::new(pool);
        repo.base.insert(&chapter(1, "1")).await.unwrap();
        repo.base.insert(&chapter(2, "2")).await.unwrap();
        repo.base
            .insert(&ChapterArchive { comic_directory_fk: 2, ..chapter(3, "1") })
            .await
            .unwrap();

        let removed = repo.delete_by_comic(1).await.unwrap();
        assert_eq!(removed, 2);

        let remaining = repo.base.find_all().await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].comic_directory_fk, 2);
    }

    #[tokio::test]
    async fn test_find_all_ids_by_directory() {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterRepository::new(pool);

        repo.base.insert(&chapter(1, "1")).await.unwrap();
        repo.base.insert(&chapter(2, "2")).await.unwrap();

        let ids = repo.find_all_ids_by_directory(1).await.unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[tokio::test]
    async fn test_error_on_invalid_fk_insert() {
        let pool = setup_test_db_with_comic().await;
        sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await.unwrap();
        let repo = ChapterRepository::new(pool);

        let invalid = ChapterArchive { comic_directory_fk: 999, ..chapter(1, "001") };
        let result = repo.base.insert(&invalid).await;

        assert!(
            matches!(result, Err(DbError::ForeignKeyViolation)),
            "Deveria ter retornado ForeignKeyViolation, mas veio: {:?}",
            result
        );
    }
}
