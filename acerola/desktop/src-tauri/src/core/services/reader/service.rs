use std::sync::{Arc, Mutex};

use super::session::{ReaderSession, DEFAULT_CACHE_CAPACITY};
use crate::{
    cmd::events::reader::{
        ReaderChapterPayload, ReaderPagePayload, ReaderSessionPayload, ReaderStatusPayload,
    },
    infra::error::ReaderError,
};

const DEFAULT_PREFETCH_RADIUS: usize = 2;

#[derive(Clone)]
pub struct ReaderService {
    state: Arc<Mutex<ReaderState>>,
}

impl ReaderService {
    pub fn new() -> Self {
        Self { state: Arc::new(Mutex::new(ReaderState::default())) }
    }

    pub async fn open_chapter(
        &self, chapter: ReaderChapterPayload,
    ) -> Result<ReaderSessionPayload, ReaderError> {
        self.with_state("open_chapter", move |state| {
            let session = ReaderSession::open(chapter)?;
            let payload = session.session_payload();
            state.session = Some(session);
            Ok(payload)
        })
        .await
    }

    pub async fn load_page(
        &self, index: usize, set_current: bool,
    ) -> Result<ReaderPagePayload, ReaderError> {
        self.with_state("load_page", move |state| {
            let session = state.session.as_mut().ok_or(ReaderError::ChapterNotOpen)?;
            session.load_page(index, set_current)
        })
        .await
    }

    pub async fn set_current_page(&self, index: usize) -> Result<ReaderStatusPayload, ReaderError> {
        self.with_state("set_current_page", move |state| {
            let session = state.session.as_mut().ok_or(ReaderError::ChapterNotOpen)?;
            session.set_current_page(index)?;
            Ok(state.status())
        })
        .await
    }

    pub async fn status(&self) -> Result<ReaderStatusPayload, ReaderError> {
        self.with_state("status", |state| Ok(state.status())).await
    }

    pub async fn close_chapter(&self) -> Result<ReaderStatusPayload, ReaderError> {
        self.with_state("close_chapter", |state| {
            state.session = None;
            Ok(state.status())
        })
        .await
    }

    pub fn prefetch_window_background(&self, center: usize, radius: Option<usize>) {
        let service = self.clone();
        let radius = radius.unwrap_or(DEFAULT_PREFETCH_RADIUS);

        tokio::spawn(async move {
            if let Err(err) = service.prefetch_window(center, radius).await {
                tracing::warn!(
                    center,
                    radius,
                    error = %err,
                    "[reader] prefetch window failed"
                );
            }
        });
    }

    async fn prefetch_window(&self, center: usize, radius: usize) -> Result<(), ReaderError> {
        self.with_state("prefetch_window", move |state| {
            let session = state.session.as_mut().ok_or(ReaderError::ChapterNotOpen)?;
            session.prefetch_window(center, radius)
        })
        .await
    }

    async fn with_state<F, R>(
        &self, operation_name: &'static str, operation: F,
    ) -> Result<R, ReaderError>
    where
        F: FnOnce(&mut ReaderState) -> Result<R, ReaderError> + Send + 'static,
        R: Send + 'static,
    {
        let state = Arc::clone(&self.state);

        let task_result = tokio::task::spawn_blocking(move || {
            let mut guard = match state.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    tracing::error!(
                        operation = operation_name,
                        error = %err,
                        "[reader] state lock poisoned"
                    );

                    return Err(ReaderError::SystemFailure(format!(
                        "reader state lock poisoned during {operation_name}"
                    )));
                },
            };

            operation(&mut guard)
        })
        .await;

        match task_result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(err)) => Err(err),
            Err(err) => {
                tracing::error!(
                    operation = operation_name,
                    is_cancelled = err.is_cancelled(),
                    is_panic = err.is_panic(),
                    error = %err,
                    "[reader] blocking state task failed"
                );

                Err(ReaderError::SystemFailure(format!(
                    "reader state task failed during {operation_name}"
                )))
            },
        }
    }
}

impl Default for ReaderService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct ReaderState {
    session: Option<ReaderSession>,
}

impl ReaderState {
    fn status(&self) -> ReaderStatusPayload {
        match &self.session {
            Some(session) => session.status_payload(),
            None => ReaderStatusPayload {
                is_open: false,
                chapter_id: None,
                page_count: 0,
                current_page: None,
                cache_keys: Vec::new(),
                cache_capacity: DEFAULT_CACHE_CAPACITY,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::Path};

    use tempfile::TempDir;
    use zip::{
        write::{SimpleFileOptions, ZipWriter},
        CompressionMethod,
    };

    use super::{ReaderError, ReaderService};
    use crate::cmd::events::reader::ReaderChapterPayload;

    fn chapter(path: &Path) -> ReaderChapterPayload {
        ReaderChapterPayload {
            id: "chapter-1".to_string(),
            name: "Chapter 1".to_string(),
            path: path.to_string_lossy().to_string(),
            chapter_sort: "1".to_string(),
            volume_id: Some("volume-1".to_string()),
            volume_name: Some("Volume 1".to_string()),
            is_special: false,
            last_modified: 0,
        }
    }

    fn create_cbz(temp_dir: &TempDir, name: &str, entries: &[(&str, &[u8])]) -> std::path::PathBuf {
        let path = temp_dir.path().join(name);
        let file = File::create(&path).unwrap();
        let mut archive = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for (entry_name, bytes) in entries {
            archive.start_file(*entry_name, options).unwrap();
            archive.write_all(bytes).unwrap();
        }

        archive.finish().unwrap();
        path
    }

    #[tokio::test]
    async fn test_opens_cbz_and_loads_pages_in_natural_order() {
        let temp_dir = TempDir::new().unwrap();
        let cbz_path = create_cbz(
            &temp_dir,
            "chapter.cbz",
            &[("page10.jpg", &[10]), ("page2.png", &[2]), ("page001.jpg", &[1])],
        );
        let reader = ReaderService::new();

        let session = reader.open_chapter(chapter(&cbz_path)).await.unwrap();

        assert_eq!(session.chapter.id, "chapter-1");
        assert_eq!(session.page_count, 3);
        assert_eq!(session.current_page, 0);
        assert_eq!(session.cache_capacity, 20);

        let first_page = reader.load_page(0, true).await.unwrap();
        assert_eq!(first_page.chapter_id, "chapter-1");
        assert_eq!(first_page.index, 0);
        assert_eq!(first_page.total, 3);
        assert_eq!(first_page.mime_type, "image/jpeg");
        assert_eq!(first_page.bytes, vec![1]);
        assert!(!first_page.cache_hit);

        let second_page = reader.load_page(1, true).await.unwrap();
        assert_eq!(second_page.mime_type, "image/png");
        assert_eq!(second_page.bytes, vec![2]);

        let status = reader.status().await.unwrap();
        assert_eq!(status.current_page, Some(1));
        assert_eq!(status.page_count, 3);
    }

    #[tokio::test]
    async fn test_reuses_cache_when_loading_same_page() {
        let temp_dir = TempDir::new().unwrap();
        let cbz_path = create_cbz(&temp_dir, "chapter.cbz", &[("001.jpg", &[1])]);
        let reader = ReaderService::new();

        reader.open_chapter(chapter(&cbz_path)).await.unwrap();

        let first_read = reader.load_page(0, false).await.unwrap();
        assert!(!first_read.cache_hit);

        let second_read = reader.load_page(0, false).await.unwrap();
        assert!(second_read.cache_hit);
        assert_eq!(second_read.bytes, vec![1]);
    }

    #[tokio::test]
    async fn test_changes_current_page_without_loading_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cbz_path = create_cbz(
            &temp_dir,
            "chapter.cbz",
            &[("001.jpg", &[1]), ("002.jpg", &[2]), ("003.jpg", &[3])],
        );
        let reader = ReaderService::new();

        reader.open_chapter(chapter(&cbz_path)).await.unwrap();

        let status = reader.set_current_page(2).await.unwrap();
        assert_eq!(status.current_page, Some(2));
        assert_eq!(status.cache_keys, Vec::<usize>::new());
    }

    #[tokio::test]
    async fn test_prefetch_loads_neighboring_pages() {
        let temp_dir = TempDir::new().unwrap();
        let cbz_path = create_cbz(
            &temp_dir,
            "chapter.cbz",
            &[
                ("001.jpg", &[1]),
                ("002.jpg", &[2]),
                ("003.jpg", &[3]),
                ("004.jpg", &[4]),
                ("005.jpg", &[5]),
            ],
        );
        let reader = ReaderService::new();

        reader.open_chapter(chapter(&cbz_path)).await.unwrap();
        reader.prefetch_window(2, 1).await.unwrap();

        let mut cache_keys = reader.status().await.unwrap().cache_keys;
        cache_keys.sort_unstable();
        assert_eq!(cache_keys, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_closes_chapter_and_clears_status() {
        let temp_dir = TempDir::new().unwrap();
        let cbz_path = create_cbz(&temp_dir, "chapter.cbz", &[("001.jpg", &[1])]);
        let reader = ReaderService::new();

        reader.open_chapter(chapter(&cbz_path)).await.unwrap();
        reader.load_page(0, true).await.unwrap();

        let status = reader.close_chapter().await.unwrap();
        assert!(!status.is_open);
        assert_eq!(status.chapter_id, None);
        assert_eq!(status.page_count, 0);
        assert_eq!(status.current_page, None);
        assert_eq!(status.cache_keys, Vec::<usize>::new());
    }

    #[tokio::test]
    async fn test_load_page_without_open_chapter_returns_error() {
        let reader = ReaderService::new();
        let error = reader.load_page(0, true).await.unwrap_err();

        assert!(matches!(error, ReaderError::ChapterNotOpen));
    }

    #[tokio::test]
    async fn test_load_page_out_of_bounds_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let cbz_path = create_cbz(&temp_dir, "chapter.cbz", &[("001.jpg", &[1])]);
        let reader = ReaderService::new();

        reader.open_chapter(chapter(&cbz_path)).await.unwrap();
        let error = reader.load_page(1, true).await.unwrap_err();

        assert!(matches!(error, ReaderError::PageOutOfBounds { index: 1, total: 1 }));
    }

    #[tokio::test]
    async fn test_open_chapter_rejects_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("chapter.txt");
        std::fs::write(&path, "content").unwrap();
        let reader = ReaderService::new();

        let error = reader.open_chapter(chapter(&path)).await.unwrap_err();

        assert!(matches!(error, ReaderError::UnsupportedFormat(format) if format == "txt"));
    }

    #[tokio::test]
    async fn test_open_chapter_rejects_cbz_without_images() {
        let temp_dir = TempDir::new().unwrap();
        let cbz_path = create_cbz(&temp_dir, "chapter.cbz", &[("notes.txt", b"without image")]);
        let reader = ReaderService::new();

        let error = reader.open_chapter(chapter(&cbz_path)).await.unwrap_err();

        assert!(matches!(error, ReaderError::EmptyChapter(path) if path.ends_with("chapter.cbz")));
    }
}
