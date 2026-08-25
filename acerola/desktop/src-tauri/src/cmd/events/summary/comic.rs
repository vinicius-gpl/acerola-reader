use std::collections::HashMap;

use chrono::Local;
use serde::Serialize;

use crate::data::models::{
    category::category::Category,
    metadata::{author::AuthorMetadata, comic::ComicMetadata},
    views::ComicSummaryView,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicSummaryRelations {
    pub directory_id: String,
    pub metadata_id: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicSummaryFilesystem {
    pub folder_name: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicSummaryMetadata {
    pub title: Option<String>,
    pub external_sync: bool,
    pub active_source: Option<String>,
    pub chapter_count: i64,
    pub description: Option<String>,
    pub status: Option<String>,
    pub author: Option<String>,
    /// Nota do AniList, já convertida para escala 0-10. `None` quando o quadrinho
    /// não tem nota (não sincronizado via AniList, ou obra ainda sem votos).
    pub rating: Option<f64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicSummaryArtwork {
    pub cover: Option<String>,
    pub banner: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicSummaryBookmark {
    pub id: i64,
    pub name: String,
    pub color: i64,
}

impl From<Category> for ComicSummaryBookmark {
    fn from(c: Category) -> Self {
        Self { id: c.id.unwrap_or(0), name: c.name, color: c.color }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicSummaryItem {
    pub relations: ComicSummaryRelations,
    pub filesystem: ComicSummaryFilesystem,
    pub metadata: ComicSummaryMetadata,
    pub artwork: ComicSummaryArtwork,
    pub bookmark: Option<ComicSummaryBookmark>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicSummaryPayload {
    pub comics: Vec<ComicSummaryItem>,
    pub total: usize,
    pub fetched_at: String,
}

impl ComicSummaryPayload {
    pub fn from(
        comics: Vec<ComicSummaryView>, counts: HashMap<i64, i64>,
        metadata_map: HashMap<i64, (ComicMetadata, Option<AuthorMetadata>, Option<i64>)>,
        bookmark_map: HashMap<i64, Category>,
    ) -> Self {
        let items = comics
            .into_iter()
            .map(|view| {
                let count = counts.get(&view.directory_fk).cloned().unwrap_or(0);
                let meta = metadata_map.get(&view.directory_fk).cloned();
                let bookmark = bookmark_map.get(&view.directory_fk).cloned();
                ComicSummaryItem::from_view(view, count, meta, bookmark)
            })
            .collect::<Vec<_>>();
        let total = items.len();

        Self {
            total,
            comics: items,
            fetched_at: Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
        }
    }
}

impl ComicSummaryItem {
    pub fn from_view(
        view: ComicSummaryView, chapter_count: i64,
        full_metadata: Option<(ComicMetadata, Option<AuthorMetadata>, Option<i64>)>,
        bookmark: Option<Category>,
    ) -> Self {
        Self {
            relations: ComicSummaryRelations {
                directory_id: view.directory_fk.to_string(),
                metadata_id: view.metadata_fk.map(|id| id.to_string()),
            },
            filesystem: ComicSummaryFilesystem { folder_name: view.folder_name },
            metadata: ComicSummaryMetadata {
                title: view.metadata_title,
                external_sync: view.external_sync,
                active_source: view.active_source,
                chapter_count,
                description: full_metadata.as_ref().map(|(m, _, _)| m.description.clone()),
                status: full_metadata.as_ref().map(|(m, _, _)| m.status.clone()),
                author: full_metadata
                    .as_ref()
                    .and_then(|(_, a, _)| a.as_ref().map(|a| a.name.clone())),
                rating: full_metadata
                    .as_ref()
                    .and_then(|(_, _, rating)| rating.map(|score| score as f64 / 10.0)),
            },
            artwork: ComicSummaryArtwork { cover: view.folder_cover, banner: view.folder_banner },
            bookmark: bookmark.map(Into::into),
        }
    }
}
