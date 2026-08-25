use sqlx::SqlitePool;

use crate::{
    data::{
        models::history::chapter_read::ChapterRead,
        repositories::{Entity, Repository},
    },
    infra::error::DbError,
};

#[derive(Clone)]
pub struct ChapterReadRepository {
    pub base: Repository<ChapterRead>,
    pool: SqlitePool,
}

impl ChapterReadRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    /// Registra um capítulo como lido. Ignora silenciosamente se já existir
    /// (chave composta `comic_directory_id` + `chapter_archive_id`).
    pub async fn insert_or_ignore(&self, chapter_read: &ChapterRead) -> Result<(), DbError> {
        let cols = ChapterRead::columns().join(", ");
        let table = ChapterRead::table_name();

        sqlx::query(&format!(
            "INSERT INTO {} ({})
             VALUES (?, ?, ?)
             ON CONFLICT(comic_directory_id, chapter_archive_id) DO NOTHING",
            table, cols
        ))
        .bind(chapter_read.comic_directory_id)
        .bind(chapter_read.chapter_archive_id)
        .bind(chapter_read.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Retorna os IDs dos capítulos lidos de um quadrinho.
    pub async fn find_ids_by_comic(&self, comic_directory_id: i64) -> Result<Vec<i64>, DbError> {
        let table = ChapterRead::table_name();

        let rows = sqlx::query_as::<_, (i64,)>(&format!(
            "SELECT chapter_archive_id FROM {} WHERE comic_directory_id = ?",
            table
        ))
        .bind(comic_directory_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// Remove o registro de leitura de um capítulo (marca como não lido). Não usa a
    /// abstração base (`Repository::delete`, que apaga por `id`): aqui a chave é composta
    /// por `comic_directory_id` + `chapter_archive_id`, daí o nome mais explícito.
    pub async fn delete_by_comic_and_chapter(
        &self, comic_directory_id: i64, chapter_archive_id: i64,
    ) -> Result<(), DbError> {
        let table = ChapterRead::table_name();

        sqlx::query(&format!(
            "DELETE FROM {} WHERE comic_directory_id = ? AND chapter_archive_id = ?",
            table
        ))
        .bind(comic_directory_id)
        .bind(chapter_archive_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Registra múltiplos capítulos como lidos em uma única query. Ignora
    /// silenciosamente os que já existirem.
    pub async fn insert_batch(
        &self, comic_directory_id: i64, chapter_ids: &[i64], created_at: i64,
    ) -> Result<usize, DbError> {
        if chapter_ids.is_empty() {
            return Ok(0);
        }

        let table = ChapterRead::table_name();
        let cols = ChapterRead::columns().join(", ");
        let placeholders =
            chapter_ids.iter().map(|_| "(?, ?, ?)").collect::<Vec<_>>().join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {} ON CONFLICT(comic_directory_id, chapter_archive_id) DO NOTHING",
            table, cols, placeholders
        );

        let mut query = sqlx::query(&sql);
        for &chapter_id in chapter_ids {
            query = query.bind(comic_directory_id).bind(chapter_id).bind(created_at);
        }

        let rows_affected = query.execute(&self.pool).await?.rows_affected();
        Ok(rows_affected as usize)
    }

    /// Retorna todos os marcadores de "lido" já traduzidos pra chave natural (nome do
    /// quadrinho + `chapter_sort` do capítulo), usado pelo sync P2P pra montar o manifesto
    /// local — os IDs autoincrement não são comparáveis entre bases SQLite de devices
    /// diferentes. Usa `chapter_sort`, não o rótulo (`chapter`): o rótulo não é comparável
    /// entre devices (cada um nomeia o arquivo como quiser), `chapter_sort` sim.
    pub async fn find_all_with_natural_keys(&self) -> Result<Vec<(String, String, i64)>, DbError> {
        let rows = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT c.name, ca.chapter_sort, cr.created_at
             FROM chapter_read cr
             JOIN comic_directory c  ON cr.comic_directory_id = c.id
             JOIN chapter_archive ca ON cr.chapter_archive_id = ca.id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Remove o registro de leitura de múltiplos capítulos em batch.
    pub async fn delete_batch(
        &self, comic_directory_id: i64, chapter_ids: &[i64],
    ) -> Result<usize, DbError> {
        if chapter_ids.is_empty() {
            return Ok(0);
        }

        let table = ChapterRead::table_name();
        let placeholders = chapter_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        let sql = format!(
            "DELETE FROM {} WHERE comic_directory_id = ? AND chapter_archive_id IN ({})",
            table, placeholders
        );

        let mut query = sqlx::query(&sql).bind(comic_directory_id);
        for &chapter_id in chapter_ids {
            query = query.bind(chapter_id);
        }

        let rows_affected = query.execute(&self.pool).await?.rows_affected();
        Ok(rows_affected as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::ChapterReadRepository;
    use crate::{
        data::models::history::chapter_read::ChapterRead,
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    fn capitulo_lido(comic_directory_id: i64, chapter_archive_id: i64) -> ChapterRead {
        ChapterRead { comic_directory_id, chapter_archive_id, created_at: 1000 }
    }

    async fn setup() -> (sqlx::SqlitePool, ChapterReadRepository) {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterReadRepository::new(pool.clone());
        (pool, repo)
    }

    #[tokio::test]
    async fn test_insert_or_ignore_records_read() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.insert_or_ignore(&capitulo_lido(1, 1)).await.unwrap();

        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 1);
    }

    #[tokio::test]
    async fn test_insert_or_ignore_does_not_duplicate() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.insert_or_ignore(&capitulo_lido(1, 1)).await.unwrap();
        repo.insert_or_ignore(&capitulo_lido(1, 1)).await.unwrap();

        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_find_ids_by_comic_empty() {
        let (_, repo) = setup().await;
        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert!(ids.is_empty());
    }

    async fn inserir_capitulos(pool: &sqlx::SqlitePool, ids: &[i64]) {
        for &id in ids {
            sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (?, ?, ?, ?, 0, 1, 0)")
                .bind(id)
                .bind(format!("{}", id))
                .bind(format!("path{}", id))
                .bind(format!("{}", id))
                .execute(pool)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_delete_removes_read_chapter() {
        let (pool, repo) = setup().await;
        inserir_capitulos(&pool, &[1]).await;

        repo.insert_or_ignore(&capitulo_lido(1, 1)).await.unwrap();
        assert_eq!(repo.find_ids_by_comic(1).await.unwrap().len(), 1);

        repo.delete_by_comic_and_chapter(1, 1).await.unwrap();
        assert!(repo.find_ids_by_comic(1).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_chapter_does_not_fail() {
        let (_, repo) = setup().await;
        repo.delete_by_comic_and_chapter(1, 999).await.unwrap();
    }

    #[tokio::test]
    async fn test_insert_batch_records_multiple() {
        let (pool, repo) = setup().await;
        inserir_capitulos(&pool, &[1, 2, 3]).await;

        let count = repo.insert_batch(1, &[1, 2, 3], 1000).await.unwrap();
        assert_eq!(count, 3);

        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn test_insert_batch_ignores_duplicates() {
        let (pool, repo) = setup().await;
        inserir_capitulos(&pool, &[1, 2]).await;

        repo.insert_batch(1, &[1], 1000).await.unwrap();
        let count = repo.insert_batch(1, &[1, 2], 1000).await.unwrap();

        assert_eq!(count, 1);
        assert_eq!(repo.find_ids_by_comic(1).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_insert_batch_empty_returns_zero() {
        let (_, repo) = setup().await;
        let count = repo.insert_batch(1, &[], 1000).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_delete_batch_removes_multiple() {
        let (pool, repo) = setup().await;
        inserir_capitulos(&pool, &[1, 2, 3]).await;
        repo.insert_batch(1, &[1, 2, 3], 1000).await.unwrap();

        let count = repo.delete_batch(1, &[1, 2]).await.unwrap();

        assert_eq!(count, 2);
        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids, vec![3]);
    }

    #[tokio::test]
    async fn test_delete_batch_empty_returns_zero() {
        let (_, repo) = setup().await;
        let count = repo.delete_batch(1, &[]).await.unwrap();
        assert_eq!(count, 0);
    }
}
