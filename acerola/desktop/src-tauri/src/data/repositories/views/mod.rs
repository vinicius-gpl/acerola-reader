use sqlx::{query_as, SqlitePool};

use crate::{
    data::{
        models::views::ComicSummaryView,
        repositories::{Entity, Repository},
    },
    infra::error::DbError,
};

/// Criteria para ordenação da biblioteca na Home.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortCriteria {
    #[default]
    TitleAsc,
    TitleDesc,
    ChapterCountAsc,
    ChapterCountDesc,
    LastUpdatedAsc,
    LastUpdatedDesc,
}

#[derive(Clone)]
pub struct HomeRepository {
    pub base: Repository<ComicSummaryView>,
    pool: SqlitePool,
}

impl HomeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    pub async fn search_by_title(&self, query_str: &str) -> Result<Vec<ComicSummaryView>, DbError> {
        let table = ComicSummaryView::table_name();
        let cols = ComicSummaryView::columns().join(", ");
        let search_pattern = format!("%{}%", query_str.to_lowercase());

        let sql = format!(
            "SELECT {} FROM {} WHERE LOWER(folder_name) LIKE ? OR LOWER(metadata_title) LIKE ?",
            cols, table
        );

        let result = query_as::<_, ComicSummaryView>(&sql)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .fetch_all(&self.pool)
            .await?;
        Ok(result)
    }

    /// Busca todos os quadrinhos com ordenação e filtros.
    pub async fn find_all_sorted(
        &self, criteria: SortCriteria, show_hidden: bool, metadata_source: Option<String>,
    ) -> Result<Vec<ComicSummaryView>, DbError> {
        let table = ComicSummaryView::table_name();
        let cols = ComicSummaryView::columns()
            .iter()
            .map(|c| format!("v.{}", c))
            .collect::<Vec<_>>()
            .join(", ");

        // Build WHERE clause with filters on joined tables
        let mut filters = Vec::new();
        if !show_hidden {
            filters.push("d.hidden = 0".to_string());
        }
        match metadata_source.as_deref() {
            Some("no_metadata") => filters.push("v.active_source IS NULL".to_string()),
            Some(source) if !source.is_empty() => {
                filters.push(format!("LOWER(v.active_source) = '{}'", source.to_lowercase()));
            },
            _ => {},
        }
        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", filters.join(" AND "))
        };

        match criteria {
            SortCriteria::TitleAsc | SortCriteria::TitleDesc => {
                let dir = if matches!(criteria, SortCriteria::TitleAsc) { "ASC" } else { "DESC" };
                let sql = format!(
                    "SELECT {} FROM {} v JOIN comic_directory d ON v.directory_fk = d.id {} ORDER BY COALESCE(v.metadata_title, v.folder_name) {}",
                    cols, table, where_clause, dir
                );
                let result = query_as::<_, ComicSummaryView>(&sql).fetch_all(&self.pool).await?;
                Ok(result)
            },
            SortCriteria::ChapterCountAsc | SortCriteria::ChapterCountDesc => {
                let dir = if matches!(criteria, SortCriteria::ChapterCountAsc) { "ASC" } else { "DESC" };
                let sql = format!(
                    "SELECT {} FROM {} v JOIN comic_directory d ON v.directory_fk = d.id LEFT JOIN (SELECT comic_directory_fk, COUNT(*) as chapter_count FROM chapter_archive GROUP BY comic_directory_fk) c ON v.directory_fk = c.comic_directory_fk {} ORDER BY COALESCE(c.chapter_count, 0) {}",
                    cols, table, where_clause, dir
                );
                let result = query_as::<_, ComicSummaryView>(&sql).fetch_all(&self.pool).await?;
                Ok(result)
            },
            SortCriteria::LastUpdatedAsc | SortCriteria::LastUpdatedDesc => {
                let dir =
                    if matches!(criteria, SortCriteria::LastUpdatedAsc) { "ASC" } else { "DESC" };
                let sql = format!(
                    "SELECT {} FROM {} v JOIN comic_directory d ON v.directory_fk = d.id {} ORDER BY d.last_modified {}",
                    cols, table, where_clause, dir
                );
                let result = query_as::<_, ComicSummaryView>(&sql).fetch_all(&self.pool).await?;
                Ok(result)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::setup_test_db::{
        insert_comic_directory, insert_comic_metadata, insert_comic_with_chapters, setup_test_db,
    };

    async fn setup() -> (SqlitePool, HomeRepository) {
        let pool = setup_test_db().await;
        let repo = HomeRepository::new(pool.clone());
        (pool, repo)
    }

    #[tokio::test]
    async fn test_search_by_title_lower_and_upper() {
        let (pool, repo) = setup().await;

        insert_comic_directory(&pool, 1, "One Piece", "/mangas/one piece").await;
        insert_comic_metadata(&pool, 1, 1, "One Piece - Piratas").await;

        insert_comic_directory(&pool, 2, "NARUTO", "/mangas/naruto").await;

        let results_upper = repo.search_by_title("PIECE").await.unwrap();
        assert_eq!(results_upper.len(), 1);
        assert_eq!(results_upper[0].folder_name, "One Piece");

        let results_lower = repo.search_by_title("naruto").await.unwrap();
        assert_eq!(results_lower.len(), 1);
        assert_eq!(results_lower[0].folder_name, "NARUTO");

        let results_empty = repo.search_by_title("Dragon Ball").await.unwrap();
        assert_eq!(results_empty.len(), 0);
    }

    #[tokio::test]
    async fn test_sorting_by_title_asc() {
        let (pool, repo) = setup().await;

        insert_comic_with_chapters(&pool, 1, "Zatch Bell", "Zatch Bell", 1).await;
        insert_comic_with_chapters(&pool, 2, "Attack on Titan", "Attack on Titan", 1).await;
        insert_comic_with_chapters(&pool, 3, "Berserk", "Berserk", 1).await;

        let result = repo.find_all_sorted(SortCriteria::TitleAsc, false, None).await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].metadata_title, Some("Attack on Titan".to_string()));
        assert_eq!(result[1].metadata_title, Some("Berserk".to_string()));
        assert_eq!(result[2].metadata_title, Some("Zatch Bell".to_string()));
    }

    #[tokio::test]
    async fn test_sorting_by_title_desc() {
        let (pool, repo) = setup().await;

        insert_comic_with_chapters(&pool, 1, "Zatch Bell", "Zatch Bell", 1).await;
        insert_comic_with_chapters(&pool, 2, "Attack on Titan", "Attack on Titan", 1).await;
        insert_comic_with_chapters(&pool, 3, "Berserk", "Berserk", 1).await;

        let result = repo.find_all_sorted(SortCriteria::TitleDesc, false, None).await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].metadata_title, Some("Zatch Bell".to_string()));
        assert_eq!(result[1].metadata_title, Some("Berserk".to_string()));
        assert_eq!(result[2].metadata_title, Some("Attack on Titan".to_string()));
    }

    #[tokio::test]
    async fn test_sorting_by_chapter_count_asc() {
        let (pool, repo) = setup().await;

        insert_comic_with_chapters(&pool, 1, "Manga A", "Manga A", 5).await;
        insert_comic_with_chapters(&pool, 2, "Manga B", "Manga B", 2).await;
        insert_comic_with_chapters(&pool, 3, "Manga C", "Manga C", 10).await;

        let result = repo.find_all_sorted(SortCriteria::ChapterCountAsc, false, None).await.unwrap();
        assert_eq!(result.len(), 3);
        // Ordenado por contagem: B (2), A (5), C (10)
        assert_eq!(result[0].folder_name, "Manga B");
        assert_eq!(result[1].folder_name, "Manga A");
        assert_eq!(result[2].folder_name, "Manga C");
    }

    #[tokio::test]
    async fn test_sorting_by_chapter_count_desc() {
        let (pool, repo) = setup().await;

        insert_comic_with_chapters(&pool, 1, "Manga A", "Manga A", 5).await;
        insert_comic_with_chapters(&pool, 2, "Manga B", "Manga B", 2).await;
        insert_comic_with_chapters(&pool, 3, "Manga C", "Manga C", 10).await;

        let result = repo.find_all_sorted(SortCriteria::ChapterCountDesc, false, None).await.unwrap();
        assert_eq!(result.len(), 3);
        // Ordenado por contagem: C (10), A (5), B (2)
        assert_eq!(result[0].folder_name, "Manga C");
        assert_eq!(result[1].folder_name, "Manga A");
        assert_eq!(result[2].folder_name, "Manga B");
    }
}
