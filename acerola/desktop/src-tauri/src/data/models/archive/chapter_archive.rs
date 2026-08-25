use serde::{Deserialize, Serialize};
use sqlx::{query::Query, sqlite::SqliteArguments, Sqlite};

use crate::data::repositories::{Bindable, Entity};

/// Contrato com o [`crate::data::repositories::Repository`] genérico.
impl Entity for ChapterArchive {
    fn columns() -> &'static [&'static str] {
        &[
            "id",
            "chapter",
            "path",
            "chapter_sort",
            "is_special",
            "checksum",
            "comic_directory_fk",
            "volume_fk",
            "last_modified",
        ]
    }
    fn table_name() -> &'static str {
        "chapter_archive"
    }
    fn id(&self) -> i64 {
        self.id
    }
}

/// Garante que o código consiga serializar o sql para o objeto
impl Bindable for ChapterArchive {
    fn bind_insert<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(self.id)
            .bind(&self.chapter)
            .bind(&self.path)
            .bind(&self.chapter_sort)
            .bind(self.is_special)
            .bind(&self.checksum)
            .bind(self.comic_directory_fk)
            .bind(self.volume_fk)
            .bind(self.last_modified)
    }

    fn bind_update<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(&self.chapter)
            .bind(&self.path)
            .bind(&self.chapter_sort)
            .bind(self.is_special)
            .bind(&self.checksum)
            .bind(self.comic_directory_fk)
            .bind(self.volume_fk)
            .bind(self.last_modified)
            .bind(self.id) // <- id pro WHERE id = ?
    }
}

impl ChapterArchive {
    pub fn format_sort(chapter: u64, decimal: Option<String>) -> String {
        match decimal {
            Some(d) if !d.is_empty() => format!("{chapter}.{d}"),
            _ => chapter.to_string(),
        }
    }

    pub fn fallback_sort(file_name: &str, index: usize) -> String {
        file_name
            .find(|character: char| character.is_ascii_digit())
            .map(|start_index| {
                file_name[start_index..]
                    .chars()
                    .scan(false, |has_decimal_separator, current_char| match current_char {
                        digit if digit.is_ascii_digit() => Some(digit),
                        separator
                            if (separator == '.' || separator == ',')
                                && !*has_decimal_separator =>
                        {
                            *has_decimal_separator = true;
                            Some('.')
                        },
                        _ => None,
                    })
                    .collect::<String>()
                    .trim_end_matches('.')
                    .to_string()
            })
            .filter(|numeric_sequence| !numeric_sequence.is_empty())
            .unwrap_or_else(|| (index + 1).to_string())
    }
}

// NOTE: Migration em src-tauri\migrations\archive
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ChapterArchive {
    pub id: i64,
    pub chapter: String,
    pub path: String,
    pub chapter_sort: String,
    pub is_special: bool,
    pub checksum: Option<String>,
    pub comic_directory_fk: i64,
    pub volume_fk: Option<i64>,
    pub last_modified: i64,
}

pub use crate::infra::pattern::volume_special::is_special_name;

#[cfg(test)]
mod tests {
    use super::ChapterArchive;

    // NOTE: Format sort

    #[test]
    fn test_formatting_detected_template() {
        assert_eq!(ChapterArchive::format_sort(10, Some("1".to_string())), "10.1");
    }

    #[test]
    fn test_formatting_without_decimal() {
        assert_eq!(ChapterArchive::format_sort(10, None), "10");
    }

    #[test]
    fn test_formatting_with_decimal_only() {
        assert_eq!(ChapterArchive::format_sort(0, Some("10".to_string())), "0.10");
    }

    #[test]
    fn test_formatting_preserves_leading_zero_in_decimal() {
        assert_eq!(ChapterArchive::format_sort(0, Some("01".to_string())), "0.01");
    }

    // NOTE: Fallback sort

    #[test]
    fn test_fallback_without_detected_template() {
        assert_eq!(ChapterArchive::fallback_sort("10,5", 0), "10.5");
    }

    #[test]
    fn test_fallback_file_with_letters() {
        assert_eq!(ChapterArchive::fallback_sort("abc", 0), "1");
    }

    #[test]
    fn test_fallback_file_without_decimal() {
        assert_eq!(ChapterArchive::fallback_sort("10", 0), "10");
    }

    #[test]
    fn test_fallback_file_with_decimal_only() {
        assert_eq!(ChapterArchive::fallback_sort("0,10", 0), "0.10");
    }
}
