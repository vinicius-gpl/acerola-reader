use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VolumeViewType {
    Chapter,
    Volume,
    CoverVolume,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterFileDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub chapter_sort: String,
    pub volume_id: Option<String>,
    pub volume_name: Option<String>,
    pub is_special: bool,
    pub last_modified: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VolumeArchiveDto {
    pub id: String,
    pub name: String,
    pub volume_sort: String,
    pub is_special: bool,
    pub cover_uri: Option<String>,
    pub banner_uri: Option<String>,
    pub last_modified: i64,
    pub chapter_count: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VolumeChapterGroupDto {
    pub volume: VolumeArchiveDto,
    pub items: Vec<ChapterFileDto>,
    pub total_chapters: i64,
    pub loaded_count: i64,
    pub has_more: bool,
    pub current_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterPageDto {
    pub items: Vec<ChapterFileDto>,
    pub volumes: Vec<VolumeArchiveDto>,
    pub page_size: i64,
    pub page: i64,
    pub total: i64,
    pub volume_sections: Vec<VolumeChapterGroupDto>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterDto {
    pub archive: ChapterPageDto,
    pub show_volume_headers: bool,
    pub has_volume_structure: bool,
    pub effective_view_mode: VolumeViewType,
}
