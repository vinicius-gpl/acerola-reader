use sqlx::SqlitePool;

use crate::{
    data::{
        models::metadata::{
            anilist_source::AnilistSource, author::AuthorMetadata, comic::ComicMetadata,
        },
        repositories::{Entity, Repository},
    },
    infra::error::DbError,
};

#[derive(Clone)]
pub struct MetadataRepository {
    pub comic_repo: Repository<ComicMetadata>,
    pub author_repo: Repository<AuthorMetadata>,
    pub anilist_repo: Repository<AnilistSource>,
    pool: SqlitePool,
}

impl MetadataRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            comic_repo: Repository::new(pool.clone()),
            author_repo: Repository::new(pool.clone()),
            anilist_repo: Repository::new(pool.clone()),
            pool,
        }
    }

    pub async fn get_comic_metadata_by_comic_id(
        &self, comic_id: i64,
    ) -> Result<Option<ComicMetadata>, DbError> {
        let table = ComicMetadata::table_name();
        let cols = ComicMetadata::columns().join(", ");

        let result = sqlx::query_as::<_, ComicMetadata>(&format!(
            "SELECT {} FROM {} WHERE comic_directory_fk = ?",
            cols, table
        ))
        .bind(comic_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_author_metadata_by_comic_metadata_id(
        &self, metadata_id: i64,
    ) -> Result<Vec<AuthorMetadata>, DbError> {
        let table = AuthorMetadata::table_name();
        let cols = AuthorMetadata::columns().join(", ");

        let result = sqlx::query_as::<_, AuthorMetadata>(&format!(
            "SELECT {} FROM {} WHERE comic_metadata_fk = ?",
            cols, table
        ))
        .bind(metadata_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    /// Busca a linha de dados do AniList (nota, popularidade) vinculada a um `comic_metadata`.
    pub async fn get_anilist_source_by_comic_metadata_id(
        &self, metadata_id: i64,
    ) -> Result<Option<AnilistSource>, DbError> {
        let table = AnilistSource::table_name();
        let cols = AnilistSource::columns().join(", ");

        let result = sqlx::query_as::<_, AnilistSource>(&format!(
            "SELECT {} FROM {} WHERE comic_metadata_fk = ?",
            cols, table
        ))
        .bind(metadata_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Remove por completo os metadados sincronizados de um quadrinho: o registro
    /// principal em `comic_metadata` e todas as tabelas filhas (autores e fontes
    /// externas). O cascade de FK não roda em runtime (ver comentário em
    /// `VolumeRepository::delete_by_comic`), então cada tabela filha precisa ser
    /// limpa explicitamente antes de apagar o registro pai. No-op se o quadrinho
    /// nunca teve metadados sincronizados.
    pub async fn delete_by_comic_id(&self, comic_directory_fk: i64) -> Result<(), DbError> {
        let Some(metadata) = self.get_comic_metadata_by_comic_id(comic_directory_fk).await? else {
            return Ok(());
        };

        sqlx::query("DELETE FROM author WHERE comic_metadata_fk = ?")
            .bind(metadata.id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM genre WHERE comic_metadata_fk = ?")
            .bind(metadata.id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM anilist_source WHERE comic_metadata_fk = ?")
            .bind(metadata.id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM mangadex_source WHERE comic_metadata_fk = ?")
            .bind(metadata.id)
            .execute(&self.pool)
            .await?;

        self.comic_repo.delete(metadata.id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::setup_test_db::{setup_test_db, setup_test_db_with_comic};

    async fn setup() -> MetadataRepository {
        MetadataRepository::new(setup_test_db().await)
    }

    async fn setup_with_comic() -> MetadataRepository {
        MetadataRepository::new(setup_test_db_with_comic().await)
    }

    fn fake_comic_metadata() -> ComicMetadata {
        ComicMetadata {
            id: 1,
            title: "Berserk".to_string(),
            description: "Dark fantasy".to_string(),
            status: "Ongoing".to_string(),
            publication: Some(1989),
            sync_source: Some("MangaDex".to_string()),
            has_comic_info: false,
            comic_directory_fk: None,
        }
    }

    #[tokio::test]
    async fn should_insert_and_fetch_comic_metadata() {
        let repo = setup().await;

        let metadata = fake_comic_metadata();

        let inserted = repo.comic_repo.insert(&metadata).await.unwrap();
        assert_eq!(inserted.id, 1);
        assert_eq!(inserted.title, "Berserk");

        let all = repo.comic_repo.find_all().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn should_fetch_comic_metadata_by_id() {
        let repo = setup_with_comic().await;

        let mut metadata = fake_comic_metadata();
        metadata.comic_directory_fk = Some(1);
        repo.comic_repo.insert(&metadata).await.unwrap();

        let result = repo.get_comic_metadata_by_comic_id(1).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().title, "Berserk");

        let not_found = repo.get_comic_metadata_by_comic_id(123).await.unwrap();
        assert!(not_found.is_none());
    }
}
