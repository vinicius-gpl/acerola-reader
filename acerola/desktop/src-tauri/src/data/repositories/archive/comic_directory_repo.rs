use sqlx::SqlitePool;

use crate::{
    data::{
        models::archive::comic_directory::ComicDirectory,
        repositories::{Entity, Repository},
    },
    infra::error::DbError,
};

#[derive(Clone)]
pub struct ComicRepository {
    pub base: Repository<ComicDirectory>,
    pool: SqlitePool,
}

impl ComicRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<ComicDirectory>, DbError> {
        let table = ComicDirectory::table_name();
        let cols = ComicDirectory::columns().join(", ");

        let result = sqlx::query_as::<_, ComicDirectory>(&format!(
            "SELECT {} FROM {} WHERE name = ?",
            cols, table
        ))
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<ComicDirectory>, DbError> {
        let table = ComicDirectory::table_name();
        let cols = ComicDirectory::columns().join(", ");

        let result = sqlx::query_as::<_, ComicDirectory>(&format!(
            "SELECT {} FROM {} WHERE id = ?",
            cols, table
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Atualiza o status de visibilidade de um quadrinho especifico.
    pub async fn update_hidden_status(
        &self, id: i64, hidden: bool,
    ) -> Result<ComicDirectory, DbError> {
        let table = ComicDirectory::table_name();
        let cols = ComicDirectory::columns().join(", ");

        let result = sqlx::query_as::<_, ComicDirectory>(&format!(
            "UPDATE {} SET hidden = ? WHERE id = ? RETURNING {}",
            table, cols
        ))
        .bind(hidden)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Atualiza o status de sincronizacao externa de um quadrinho especifico.
    pub async fn update_external_sync_enabled(
        &self, id: i64, external_sync_enabled: bool,
    ) -> Result<ComicDirectory, DbError> {
        let table = ComicDirectory::table_name();
        let cols = ComicDirectory::columns().join(", ");

        let result = sqlx::query_as::<_, ComicDirectory>(&format!(
            "UPDATE {} SET external_sync_enabled = ? WHERE id = ? RETURNING {}",
            table, cols
        ))
        .bind(external_sync_enabled)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Atualiza a capa de um quadrinho especifico.
    pub async fn update_cover(&self, id: i64, cover: &str) -> Result<ComicDirectory, DbError> {
        let table = ComicDirectory::table_name();
        let cols = ComicDirectory::columns().join(", ");

        let result = sqlx::query_as::<_, ComicDirectory>(&format!(
            "UPDATE {} SET cover = ? WHERE id = ? RETURNING {}",
            table, cols
        ))
        .bind(cover)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Remove a capa e o banner (apenas a referência no banco; não mexe nos arquivos).
    pub async fn clear_artwork(&self, id: i64) -> Result<ComicDirectory, DbError> {
        let table = ComicDirectory::table_name();
        let cols = ComicDirectory::columns().join(", ");

        let result = sqlx::query_as::<_, ComicDirectory>(&format!(
            "UPDATE {} SET cover = NULL, banner = NULL WHERE id = ? RETURNING {}",
            table, cols
        ))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Atualiza o status de visibilidade de multiplos quadrinhos em batch.
    pub async fn update_hidden_status_batch(
        &self, ids: &[i64], hidden: bool,
    ) -> Result<usize, DbError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let table = ComicDirectory::table_name();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        let sql = format!("UPDATE {} SET hidden = ? WHERE id IN ({})", table, placeholders);

        let mut query = sqlx::query(&sql).bind(hidden);
        for &id in ids {
            query = query.bind(id);
        }

        let rows_affected = query.execute(&self.pool).await?.rows_affected();
        Ok(rows_affected as usize)
    }

    /// Deleta multiplos quadrinhos em batch.
    pub async fn delete_batch(&self, ids: &[i64]) -> Result<usize, DbError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let table = ComicDirectory::table_name();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        let sql = format!("DELETE FROM {} WHERE id IN ({})", table, placeholders);

        let mut query = sqlx::query(&sql);
        for &id in ids {
            query = query.bind(id);
        }

        let rows_affected = query.execute(&self.pool).await?.rows_affected();
        Ok(rows_affected as usize)
    }

    /// Busca todos os quadrinhos filtrando pelo status de visibilidade.
    pub async fn find_all_by_hidden(&self, hidden: bool) -> Result<Vec<ComicDirectory>, DbError> {
        let table = ComicDirectory::table_name();
        let cols = ComicDirectory::columns().join(", ");

        let result = sqlx::query_as::<_, ComicDirectory>(&format!(
            "SELECT {} FROM {} WHERE hidden = ?",
            cols, table
        ))
        .bind(hidden)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::ComicRepository;
    use crate::{
        data::models::archive::comic_directory::ComicDirectory, infra::error::DbError,
        tests::utils::setup_test_db::setup_test_db,
    };

    fn berserk() -> ComicDirectory {
        ComicDirectory {
            id: 1,
            name: "Berserk".to_string(),
            path: "/quadrinhos/berserk".to_string(),
            cover: None,
            banner: None,
            last_modified: 1700000000,
            archive_template_fk: None,
            external_sync_enabled: true,
            hidden: false,
        }
    }

    async fn setup() -> ComicRepository {
        ComicRepository::new(setup_test_db().await)
    }

    #[tokio::test]
    async fn test_insert_and_find_all() {
        let repo = setup().await;

        let inserted = repo.base.insert(&berserk()).await.unwrap();

        assert_eq!(inserted.id, 1);
        assert_eq!(inserted.name, "Berserk");

        let all = repo.base.find_all().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_find_by_name() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();

        let result = repo.find_by_name("Berserk").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Berserk");
    }

    #[tokio::test]
    async fn test_update() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();

        let updated = ComicDirectory { name: "Berserk Deluxe".to_string(), ..berserk() };
        let result = repo.base.update(&updated).await.unwrap();

        assert_eq!(result.name, "Berserk Deluxe");
    }

    #[tokio::test]
    async fn test_delete() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();
        repo.base.delete(1).await.unwrap();

        let all = repo.base.find_all().await.unwrap();
        assert_eq!(all.len(), 0);
    }

    #[tokio::test]
    async fn test_find_by_nonexistent_name() {
        let repo = setup().await;

        let result = repo.find_by_name("Inexistente").await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_error_on_duplicate_insert() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();
        let result = repo.base.insert(&berserk()).await;

        assert!(
            matches!(result, Err(DbError::UniqueViolation)),
            "Deveria ter retornado UniqueViolation, mas veio: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_error_on_updating_nonexistent() {
        let repo = setup().await;

        let result = repo.base.update(&berserk()).await;

        assert!(
            matches!(result, Err(DbError::NotFound)),
            "Deveria ter retornado NotFound, mas veio: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_update_visibility_status() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();

        let result = repo.update_hidden_status(1, true).await.unwrap();
        assert!(result.hidden);

        let fetched = repo.find_by_name("Berserk").await.unwrap().unwrap();
        assert!(fetched.hidden);
    }

    #[tokio::test]
    async fn test_update_visibility_status_batch() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();
        repo.base
            .insert(&ComicDirectory {
                id: 2,
                name: "Vinland Saga".to_string(),
                path: "/quadrinhos/vinland".to_string(),
                cover: None,
                banner: None,
                last_modified: 1700000000,
                archive_template_fk: None,
                external_sync_enabled: false,
                hidden: false,
            })
            .await
            .unwrap();
        repo.base
            .insert(&ComicDirectory {
                id: 3,
                name: "One Piece".to_string(),
                path: "/quadrinhos/onepiece".to_string(),
                cover: None,
                banner: None,
                last_modified: 1700000000,
                archive_template_fk: None,
                external_sync_enabled: false,
                hidden: false,
            })
            .await
            .unwrap();

        let count = repo.update_hidden_status_batch(&[1, 2], true).await.unwrap();
        assert_eq!(count, 2);

        let hidden = repo.find_all_by_hidden(true).await.unwrap();
        assert_eq!(hidden.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_batch() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();
        repo.base
            .insert(&ComicDirectory {
                id: 2,
                name: "Vinland Saga".to_string(),
                path: "/quadrinhos/vinland".to_string(),
                cover: None,
                banner: None,
                last_modified: 1700000000,
                archive_template_fk: None,
                external_sync_enabled: false,
                hidden: false,
            })
            .await
            .unwrap();

        let count = repo.delete_batch(&[1, 2]).await.unwrap();
        assert_eq!(count, 2);

        let all = repo.base.find_all().await.unwrap();
        assert_eq!(all.len(), 0);
    }

    #[tokio::test]
    async fn test_find_by_visibility_status() {
        let repo = setup().await;

        repo.base.insert(&berserk()).await.unwrap();
        repo.base
            .insert(&ComicDirectory {
                id: 2,
                name: "Hidden Manga".to_string(),
                path: "/quadrinhos/hidden".to_string(),
                cover: None,
                banner: None,
                last_modified: 1700000000,
                archive_template_fk: None,
                external_sync_enabled: false,
                hidden: true,
            })
            .await
            .unwrap();

        let visible = repo.find_all_by_hidden(false).await.unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "Berserk");

        let hidden = repo.find_all_by_hidden(true).await.unwrap();
        assert_eq!(hidden.len(), 1);
        assert_eq!(hidden[0].name, "Hidden Manga");
    }
}
