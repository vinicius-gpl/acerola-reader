use sqlx::SqlitePool;

use crate::{
    data::{
        models::archive::volume_archive::VolumeArchive,
        repositories::{archive::chapter_archive_repo::ChapterSortCriteria, Entity, Repository},
    },
    infra::error::DbError,
};

pub struct VolumeRepository {
    pub base: Repository<VolumeArchive>,
    pub pool: SqlitePool,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct VolumeWithCount {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub volume_sort: String,
    pub is_special: bool,
    pub cover: Option<String>,
    pub banner: Option<String>,
    pub comic_directory_fk: i64,
    pub last_modified: i64,
    pub chapter_count: i64,
}

impl VolumeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    /// Remove todos os volumes vinculados a um quadrinho específico.
    ///
    /// Usado pelo rescan profundo, já que o cascade de FK não roda em runtime
    /// (`PRAGMA foreign_keys` não é habilitado no pool de produção).
    pub async fn delete_by_comic(&self, comic_directory_fk: i64) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM volume_archive WHERE comic_directory_fk = ?")
            .bind(comic_directory_fk)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn find_by_comic(
        &self, comic_directory_fk: i64,
    ) -> Result<Vec<VolumeArchive>, DbError> {
        let cols = VolumeArchive::columns().join(", ");
        let result = sqlx::query_as::<_, VolumeArchive>(&format!(
            "SELECT {} FROM volume_archive WHERE comic_directory_fk = ? ORDER BY CAST(volume_sort AS INTEGER) ASC",
            cols
        ))
        .bind(comic_directory_fk)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    /// Atualiza a capa de um volume especifico.
    pub async fn update_cover(&self, id: i64, cover: &str) -> Result<VolumeArchive, DbError> {
        let cols = VolumeArchive::columns().join(", ");

        let result = sqlx::query_as::<_, VolumeArchive>(&format!(
            "UPDATE volume_archive SET cover = ? WHERE id = ? RETURNING {}",
            cols
        ))
        .bind(cover)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    fn order_clause_for_volumes(criteria: ChapterSortCriteria) -> String {
        match criteria {
            ChapterSortCriteria::NumberAsc => "ORDER BY va.is_special ASC, CAST(va.volume_sort AS INTEGER) ASC, CAST(CASE WHEN va.volume_sort LIKE '%.%' THEN SUBSTR(va.volume_sort, INSTR(va.volume_sort, '.') + 1) ELSE 0 END AS INTEGER) ASC".to_string(),
            ChapterSortCriteria::NumberDesc => "ORDER BY va.is_special ASC, CAST(va.volume_sort AS INTEGER) DESC, CAST(CASE WHEN va.volume_sort LIKE '%.%' THEN SUBSTR(va.volume_sort, INSTR(va.volume_sort, '.') + 1) ELSE 0 END AS INTEGER) DESC".to_string(),
            ChapterSortCriteria::ModifiedAsc => "ORDER BY va.is_special ASC, va.last_modified ASC".to_string(),
            ChapterSortCriteria::ModifiedDesc => "ORDER BY va.is_special ASC, va.last_modified DESC".to_string(),
        }
    }

    pub async fn find_by_comic_with_counts_sorted(
        &self, comic_directory_fk: i64, criteria: ChapterSortCriteria,
    ) -> Result<Vec<VolumeWithCount>, DbError> {
        let order = Self::order_clause_for_volumes(criteria);
        let sql = format!(
            "SELECT va.*, COUNT(ca.id) as chapter_count FROM volume_archive va LEFT JOIN chapter_archive ca ON ca.volume_fk = va.id WHERE va.comic_directory_fk = ? GROUP BY va.id {}",
            order
        );

        sqlx::query_as::<_, VolumeWithCount>(&sql)
            .bind(comic_directory_fk)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn find_by_comic_with_counts(
        &self, comic_directory_fk: i64,
    ) -> Result<Vec<VolumeWithCount>, DbError> {
        let result = sqlx::query_as::<_, VolumeWithCount>(
            "SELECT 
                va.*,
                COUNT(ca.id) as chapter_count
                FROM volume_archive va
                LEFT JOIN chapter_archive ca ON ca.volume_fk = va.id
                WHERE va.comic_directory_fk = ?
                GROUP BY va.id
                ORDER BY 
                    va.is_special ASC,
                    CAST(va.volume_sort AS INTEGER) ASC,
                    CAST(
                        CASE 
                            WHEN va.volume_sort LIKE '%.%' 
                            THEN SUBSTR(va.volume_sort, INSTR(va.volume_sort, '.') + 1) 
                            ELSE 0 
                        END AS INTEGER
                    ) ASC",
        )
        .bind(comic_directory_fk)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeRepository;
    use crate::{
        data::models::archive::volume_archive::VolumeArchive, infra::error::DbError,
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    fn vol1() -> VolumeArchive {
        VolumeArchive {
            id: 1,
            name: "Vol. 01".to_string(),
            path: "/test/Vol. 01".to_string(),
            volume_sort: "1".to_string(),
            is_special: false,
            cover: None,
            banner: None,
            comic_directory_fk: 1,
            last_modified: 1700000000,
        }
    }

    #[tokio::test]
    async fn test_insert_and_find_all() {
        let pool = setup_test_db_with_comic().await;
        let repo = VolumeRepository::new(pool);
        let inserted = repo.base.insert(&vol1()).await.unwrap();
        assert_eq!(inserted.id, 1);
        assert_eq!(inserted.name, "Vol. 01");
        assert_eq!(inserted.volume_sort, "1");
        let all = repo.base.find_all().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_update() {
        let pool = setup_test_db_with_comic().await;
        let repo = VolumeRepository::new(pool);
        repo.base.insert(&vol1()).await.unwrap();
        let updated = VolumeArchive { name: "Vol. 001 Special".to_string(), ..vol1() };
        let result = repo.base.update(&updated).await.unwrap();
        assert_eq!(result.name, "Vol. 001 Special");
    }

    #[tokio::test]
    async fn test_delete() {
        let pool = setup_test_db_with_comic().await;
        let repo = VolumeRepository::new(pool);
        repo.base.insert(&vol1()).await.unwrap();
        repo.base.delete(1).await.unwrap();
        assert_eq!(repo.base.find_all().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_delete_by_comic_removes_only_volumes_from_that_comic() {
        let pool = setup_test_db_with_comic().await;
        sqlx::query(
            "INSERT INTO comic_directory (id, name, path, last_modified, external_sync_enabled, hidden)
             VALUES (2, 'Other', '/other', 0, 0, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let repo = VolumeRepository::new(pool);
        repo.base.insert(&vol1()).await.unwrap();
        repo.base
            .insert(&VolumeArchive {
                id: 2,
                name: "Vol. 02".to_string(),
                volume_sort: "2".to_string(),
                ..vol1()
            })
            .await
            .unwrap();
        repo.base
            .insert(&VolumeArchive { id: 3, comic_directory_fk: 2, ..vol1() })
            .await
            .unwrap();

        let removed = repo.delete_by_comic(1).await.unwrap();
        assert_eq!(removed, 2);

        let remaining = repo.base.find_all().await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].comic_directory_fk, 2);
    }

    #[tokio::test]
    async fn test_find_by_comic() {
        let pool = setup_test_db_with_comic().await;
        let repo = VolumeRepository::new(pool);
        repo.base.insert(&vol1()).await.unwrap();
        repo.base
            .insert(&VolumeArchive {
                id: 2,
                name: "Vol. 02".to_string(),
                volume_sort: "2".to_string(),
                ..vol1()
            })
            .await
            .unwrap();

        let volumes = repo.find_by_comic(1).await.unwrap();
        assert_eq!(volumes.len(), 2);
        assert_eq!(volumes[0].volume_sort, "1");
        assert_eq!(volumes[1].volume_sort, "2");
    }

    #[tokio::test]
    async fn test_error_unique_volume_sort_per_comic() {
        let pool = setup_test_db_with_comic().await;
        let repo = VolumeRepository::new(pool);
        repo.base.insert(&vol1()).await.unwrap();
        let dup = VolumeArchive { id: 2, ..vol1() };
        let result = repo.base.insert(&dup).await;
        assert!(matches!(result, Err(DbError::UniqueViolation)));
    }

    #[tokio::test]
    async fn test_error_on_updating_nonexistent() {
        let pool = setup_test_db_with_comic().await;
        let repo = VolumeRepository::new(pool);
        let result = repo.base.update(&vol1()).await;
        assert!(matches!(result, Err(DbError::NotFound)));
    }
}
