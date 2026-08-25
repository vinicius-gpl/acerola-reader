#[derive(Debug, sqlx::FromRow, Clone)]
pub struct ChapterArchiveWithVolume {
    pub id: i64,
    pub chapter: String,
    pub path: String,
    pub chapter_sort: String,
    pub is_special: bool,
    pub checksum: Option<String>,
    pub comic_directory_fk: i64,
    pub volume_fk: Option<i64>,
    pub last_modified: i64,
    pub volume_name: Option<String>,
    pub volume_sort: Option<String>,
    pub volume_is_special: Option<bool>,
}
