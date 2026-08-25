use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::{
    data::{
        models::{
            category::category::Category,
            metadata::{author::AuthorMetadata, comic::ComicMetadata},
            views::ComicSummaryView,
        },
        repositories::{
            archive::chapter_archive_repo::ChapterRepository,
            category::CategoryRepository,
            metadata::MetadataRepository,
            views::{HomeRepository, SortCriteria},
        },
    },
    infra::error::ComicError,
};

pub struct HomeService {
    repo: HomeRepository,
    chapter_repo: ChapterRepository,
    metadata_repo: MetadataRepository,
    category_repo: CategoryRepository,
}

impl HomeService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: HomeRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool.clone()),
            metadata_repo: MetadataRepository::new(pool.clone()),
            category_repo: CategoryRepository::new(pool),
        }
    }

    pub async fn get_all(
        &self, search: Option<String>,
    ) -> Result<
        (
            Vec<ComicSummaryView>,
            HashMap<i64, i64>,
            HashMap<i64, (ComicMetadata, Option<AuthorMetadata>, Option<i64>)>,
            HashMap<i64, Category>,
        ),
        ComicError,
    > {
        let comics = match search {
            Some(query) if !query.trim().is_empty() => self.repo.search_by_title(&query).await?,
            _ => self.repo.base.find_all().await?,
        };
        let counts = self.chapter_repo.get_all_counts().await?;

        let all_meta = self.metadata_repo.comic_repo.find_all().await?;
        let mut meta_map = HashMap::new();
        for m in all_meta {
            if let Some(fk) = m.comic_directory_fk {
                let author = self
                    .metadata_repo
                    .get_author_metadata_by_comic_metadata_id(m.id)
                    .await?
                    .into_iter()
                    .next();
                // Rating não é buscado aqui para evitar N+1 na grade da home;
                // só é resolvido na tela de detalhe (get_by_folder_name).
                meta_map.insert(fk, (m, author, None));
            }
        }

        let bookmark_map = self.category_repo.get_all_comic_bookmarks().await?;

        Ok((comics, counts, meta_map, bookmark_map))
    }

    pub async fn get_all_sorted(
        &self, criteria: SortCriteria, show_hidden: bool, metadata_source: Option<String>,
    ) -> Result<
        (
            Vec<ComicSummaryView>,
            HashMap<i64, i64>,
            HashMap<i64, (ComicMetadata, Option<AuthorMetadata>, Option<i64>)>,
            HashMap<i64, Category>,
        ),
        ComicError,
    > {
        let comics = self.repo.find_all_sorted(criteria, show_hidden, metadata_source).await?;
        let counts = self.chapter_repo.get_all_counts().await?;

        let all_meta = self.metadata_repo.comic_repo.find_all().await?;
        let mut meta_map = HashMap::new();
        for m in all_meta {
            if let Some(fk) = m.comic_directory_fk {
                let author = self
                    .metadata_repo
                    .get_author_metadata_by_comic_metadata_id(m.id)
                    .await?
                    .into_iter()
                    .next();
                meta_map.insert(fk, (m, author, None));
            }
        }

        let bookmark_map = self.category_repo.get_all_comic_bookmarks().await?;

        Ok((comics, counts, meta_map, bookmark_map))
    }

    pub async fn get_by_folder_name(
        &self, folder_name: &str,
    ) -> Result<
        Option<(
            ComicSummaryView,
            i64,
            Option<(ComicMetadata, Option<AuthorMetadata>, Option<i64>)>,
            Option<Category>,
        )>,
        ComicError,
    > {
        let comics = self.repo.base.find_all().await?;
        let comic = comics.into_iter().find(|comic| comic.folder_name == folder_name);

        if let Some(view) = comic {
            let count = self.chapter_repo.count_by_directory_id(view.directory_fk).await?;
            let meta = self.metadata_repo.get_comic_metadata_by_comic_id(view.directory_fk).await?;
            let author = if let Some(m) = &meta {
                self.metadata_repo
                    .get_author_metadata_by_comic_metadata_id(m.id)
                    .await?
                    .into_iter()
                    .next()
            } else {
                None
            };
            let rating = if let Some(m) = &meta {
                self.metadata_repo
                    .get_anilist_source_by_comic_metadata_id(m.id)
                    .await?
                    .and_then(|source| source.average_score)
            } else {
                None
            };
            let bookmark = self.category_repo.get_comic_category(view.directory_fk).await?;
            Ok(Some((view, count, meta.map(|m| (m, author, rating)), bookmark)))
        } else {
            Ok(None)
        }
    }
}
