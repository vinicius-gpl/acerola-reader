use sqlx::SqlitePool;

use crate::{
    cmd::events::summary::{
        ChapterDto, ChapterFileDto, ChapterPageDto, VolumeArchiveDto, VolumeChapterGroupDto,
        VolumeViewType,
    },
    data::repositories::{
        archive::{
            chapter_archive_repo::{ChapterRepository, ChapterSortCriteria},
            volume_archive_repo::VolumeRepository,
        },
        metadata::MetadataRepository,
    },
    infra::error::ComicError,
};

pub struct ChapterService {
    chapter_repo: ChapterRepository,
    volume_repo: VolumeRepository,
    metadata_repo: MetadataRepository,
}

impl ChapterService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            chapter_repo: ChapterRepository::new(pool.clone()),
            volume_repo: VolumeRepository::new(pool.clone()),
            metadata_repo: MetadataRepository::new(pool),
        }
    }

    pub async fn get_comic_chapters(
        &self, comic_directory_fk: i64, volume_id_filter: Option<i64>, page: i32, page_size: i32,
        sort_criteria: ChapterSortCriteria, search_query: Option<&str>,
    ) -> Result<ChapterDto, ComicError> {
        let page_i64 = page as i64;
        let page_size_i64 = page_size as i64;
        let offset = page_i64 * page_size_i64;

        let volumes_with_counts = self
            .volume_repo
            .find_by_comic_with_counts_sorted(comic_directory_fk, sort_criteria)
            .await?;
        let has_volume_structure = !volumes_with_counts.is_empty();

        let (chapters_with_volume, total_chapters) = if let Some(volume_id) = volume_id_filter {
            let items = self
                .chapter_repo
                .get_chapters_by_volume(
                    comic_directory_fk,
                    volume_id,
                    page_size_i64,
                    offset,
                    sort_criteria,
                )
                .await?;
            let count = self.chapter_repo.get_total_count_by_volume(volume_id).await?;
            (items, count)
        } else {
            let query = search_query.unwrap_or("");
            let has_search = !query.is_empty();

            let items = if has_search {
                self.chapter_repo
                    .get_chapters_by_directory_with_search(
                        comic_directory_fk,
                        page_size_i64,
                        offset,
                        query,
                        sort_criteria,
                    )
                    .await?
            } else {
                self.chapter_repo
                    .get_chapters_by_directory(
                        comic_directory_fk,
                        page_size_i64,
                        offset,
                        sort_criteria,
                    )
                    .await?
            };

            let count = if has_search {
                self.chapter_repo
                    .count_by_directory_id_with_search(comic_directory_fk, query)
                    .await?
            } else {
                self.chapter_repo.count_by_directory_id(comic_directory_fk).await?
            };
            (items, count)
        };

        let chapter_items = chapters_with_volume
            .iter()
            .map(|chapter| ChapterFileDto {
                id: chapter.id.to_string(),
                name: chapter.chapter.clone(),
                path: chapter.path.clone(),
                chapter_sort: chapter.chapter_sort.clone(),
                volume_id: chapter.volume_fk.map(|id| id.to_string()),
                volume_name: chapter.volume_name.clone(),
                is_special: chapter.is_special,
                last_modified: chapter.last_modified,
            })
            .collect::<Vec<_>>();

        let volume_dtos = volumes_with_counts
            .iter()
            .map(|volume| VolumeArchiveDto {
                id: volume.id.to_string(),
                name: volume.name.clone(),
                volume_sort: volume.volume_sort.clone(),
                is_special: volume.is_special,
                cover_uri: volume.cover.clone(),
                banner_uri: volume.banner.clone(),
                last_modified: volume.last_modified,
                chapter_count: volume.chapter_count,
            })
            .collect::<Vec<_>>();

        let mut volume_sections = Vec::new();

        if has_volume_structure {
            for volume in &volumes_with_counts {
                let chapters_in_volume = chapters_with_volume
                    .iter()
                    .filter(|chapter| chapter.volume_fk == Some(volume.id))
                    .map(|chapter| ChapterFileDto {
                        id: chapter.id.to_string(),
                        name: chapter.chapter.clone(),
                        path: chapter.path.clone(),
                        chapter_sort: chapter.chapter_sort.clone(),
                        volume_id: chapter.volume_fk.map(|id| id.to_string()),
                        volume_name: chapter.volume_name.clone(),
                        is_special: chapter.is_special,
                        last_modified: chapter.last_modified,
                    })
                    .collect::<Vec<_>>();

                if !chapters_in_volume.is_empty() {
                    let total_in_volume = volume.chapter_count;
                    let total_pages_vol =
                        (total_in_volume as f64 / page_size_i64 as f64).ceil() as i64;
                    let loaded_count_in_vol = chapters_in_volume.len() as i64;

                    volume_sections.push(VolumeChapterGroupDto {
                        volume: VolumeArchiveDto {
                            id: volume.id.to_string(),
                            name: volume.name.clone(),
                            volume_sort: volume.volume_sort.clone(),
                            is_special: volume.is_special,
                            cover_uri: volume.cover.clone(),
                            banner_uri: volume.banner.clone(),
                            last_modified: volume.last_modified,
                            chapter_count: volume.chapter_count,
                        },
                        items: chapters_in_volume,
                        total_chapters: total_in_volume,
                        loaded_count: loaded_count_in_vol,
                        has_more: (offset + loaded_count_in_vol) < total_in_volume,
                        current_page: page_i64,
                        total_pages: total_pages_vol,
                    });
                }
            }
        }

        let effective_view_mode =
            if has_volume_structure { VolumeViewType::Volume } else { VolumeViewType::Chapter };

        Ok(ChapterDto {
            archive: ChapterPageDto {
                items: chapter_items,
                volumes: volume_dtos,
                page_size: page_size_i64,
                page: page_i64,
                total: total_chapters,
                volume_sections,
            },
            show_volume_headers: has_volume_structure,
            has_volume_structure,
            effective_view_mode,
        })
    }

    /// Retorna todos os IDs de capítulo de um quadrinho, sem paginação.
    pub async fn get_all_chapter_ids(
        &self, comic_directory_fk: i64,
    ) -> Result<Vec<i64>, ComicError> {
        self.chapter_repo.find_all_ids_by_directory(comic_directory_fk).await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{models::archive::chapter_archive::ChapterArchive, repositories::Repository},
        tests::utils::setup_test_db::setup_test_db_with_volumes,
    };

    async fn popular_dados(pool: &SqlitePool) {
        let chapter_repo = Repository::<ChapterArchive>::new(pool.clone());
        chapter_repo
            .insert(&ChapterArchive {
                id: 1,
                chapter: "Cap 1".to_string(),
                path: "p1".to_string(),
                chapter_sort: "1".to_string(),
                is_special: false,
                checksum: None,
                comic_directory_fk: 1,
                volume_fk: Some(1),
                last_modified: 0,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_comic_chapters() {
        let pool = setup_test_db_with_volumes().await;
        popular_dados(&pool).await;

        let service = ChapterService::new(pool);
        let result = service
            .get_comic_chapters(1, None, 0, 25, ChapterSortCriteria::NumberAsc, None)
            .await
            .unwrap();

        assert_eq!(result.archive.total, 1);
        assert_eq!(result.archive.items.len(), 1);
        assert!(result.has_volume_structure);
        assert_eq!(result.archive.volume_sections.len(), 1);
        assert_eq!(result.archive.volume_sections[0].volume.name, "Vol 01");
        assert_eq!(result.archive.volumes[0].chapter_count, 1);
    }

    #[tokio::test]
    async fn test_get_all_chapter_ids() {
        let pool = setup_test_db_with_volumes().await;
        popular_dados(&pool).await;

        let service = ChapterService::new(pool);
        let ids = service.get_all_chapter_ids(1).await.unwrap();

        assert_eq!(ids, vec![1]);
    }
}
