use serde::Serialize;
use sqlx::prelude::FromRow;

use crate::data::repositories::Entity;

/// Contrato com o [`crate::data::repositories::Repository`] genérico.
impl Entity for ComicSummaryView {
    fn columns() -> &'static [&'static str] {
        &[
            "directory_fk",
            "folder_name",
            "folder_cover",
            "folder_banner",
            "external_sync",
            "metadata_title",
            "active_source",
            "metadata_fk",
        ]
    }
    fn table_name() -> &'static str {
        "comic_summary_view"
    }
    fn id(&self) -> i64 {
        self.directory_fk
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct ComicSummaryView {
    pub directory_fk: i64,
    pub folder_name: String,
    pub folder_cover: Option<String>,
    pub folder_banner: Option<String>,
    pub external_sync: bool,
    pub metadata_title: Option<String>,
    pub active_source: Option<String>,
    pub metadata_fk: Option<i64>,
}
