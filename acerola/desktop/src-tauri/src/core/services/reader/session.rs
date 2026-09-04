use std::{num::NonZeroUsize, path::PathBuf, sync::Arc};

use lru::LruCache;

use super::page_source::{mime_type_for, source_from_path, PageSource};
use crate::{
    cmd::events::reader::{
        ReaderChapterPayload, ReaderPagePayload, ReaderSessionPayload, ReaderStatusPayload,
    },
    infra::error::ReaderError,
};

// INFO: Janela deslizante — não "o capítulo inteiro". Cobre o raio de prefetch
// (radius 2 = 5 páginas) com folga pra scroll rápido sem acumular capítulos
// grandes (webtoon) inteiros em RAM (cada página decodificada pode passar de
// 40MB em bitmap).
pub(super) const DEFAULT_CACHE_CAPACITY: usize = 20;

pub(super) struct ReaderSession {
    chapter: ReaderChapterPayload,
    source: Box<dyn PageSource>,
    cache: LruCache<usize, Arc<CachedPage>>,
    current_page: usize,
}

impl ReaderSession {
    pub(super) fn open(chapter: ReaderChapterPayload) -> Result<Self, ReaderError> {
        let path = PathBuf::from(&chapter.path);
        let source = source_from_path(&path)?;

        Ok(Self {
            chapter,
            source,
            cache: LruCache::new(NonZeroUsize::new(DEFAULT_CACHE_CAPACITY).unwrap()),
            current_page: 0,
        })
    }

    pub(super) fn session_payload(&self) -> ReaderSessionPayload {
        ReaderSessionPayload {
            chapter: self.chapter.clone(),
            page_count: self.total_pages(),
            current_page: self.current_page,
            cache_capacity: self.cache.cap().get(),
        }
    }

    pub(super) fn status_payload(&self) -> ReaderStatusPayload {
        ReaderStatusPayload {
            is_open: true,
            chapter_id: Some(self.chapter.id.clone()),
            page_count: self.total_pages(),
            current_page: Some(self.current_page),
            cache_keys: self.cache.iter().map(|(key, _)| *key).collect(),
            cache_capacity: self.cache.cap().get(),
        }
    }

    fn total_pages(&self) -> usize {
        self.source.page_count().get()
    }

    pub(super) fn set_current_page(&mut self, index: usize) -> Result<(), ReaderError> {
        self.ensure_index(index)?;
        self.current_page = index;

        if self.cache.contains(&index) {
            self.cache.get(&index);
        }

        Ok(())
    }

    pub(super) fn load_page(
        &mut self, index: usize, set_current: bool,
    ) -> Result<ReaderPagePayload, ReaderError> {
        self.ensure_index(index)?;

        if set_current {
            self.current_page = index;
        }

        let (page, cache_hit) = self.cached_page(index)?;

        Ok(ReaderPagePayload {
            chapter_id: self.chapter.id.clone(),
            index,
            total: self.total_pages(),
            mime_type: page.mime_type.clone(),
            bytes: page.bytes.clone(),
            cache_hit,
        })
    }

    pub(super) fn prefetch_window(
        &mut self, center: usize, radius: usize,
    ) -> Result<(), ReaderError> {
        self.ensure_index(center)?;

        for index in window_indices(center, self.total_pages(), radius) {
            self.ensure_cached(index)?;
        }

        Ok(())
    }

    fn cached_page(&mut self, index: usize) -> Result<(Arc<CachedPage>, bool), ReaderError> {
        if let Some(page) = self.cache.get(&index) {
            return Ok((Arc::clone(page), true));
        }

        let page = Arc::new(self.read_page(index)?);
        self.cache.put(index, Arc::clone(&page));

        Ok((page, false))
    }

    fn ensure_cached(&mut self, index: usize) -> Result<(), ReaderError> {
        if self.cache.contains(&index) {
            return Ok(());
        }

        let page = Arc::new(self.read_page(index)?);
        self.cache.put(index, page);
        Ok(())
    }

    fn read_page(&self, index: usize) -> Result<CachedPage, ReaderError> {
        let raw = self.source.read_page(index)?;
        let mime_type = mime_type_for(&raw.name)?;

        Ok(CachedPage { mime_type, bytes: raw.bytes })
    }

    fn ensure_index(&self, index: usize) -> Result<(), ReaderError> {
        let total = self.total_pages();

        if index >= total {
            return Err(ReaderError::PageOutOfBounds { index, total });
        }

        Ok(())
    }
}

struct CachedPage {
    mime_type: String,
    bytes: Vec<u8>,
}

pub(super) fn window_indices(center: usize, total: usize, radius: usize) -> Vec<usize> {
    if total == 0 {
        return Vec::new();
    }

    let start = center.saturating_sub(radius);
    let end = center.saturating_add(radius).min(total - 1);

    (start..=end).collect()
}

#[cfg(test)]
mod tests {
    use super::window_indices;
    use crate::infra::pattern::natural_sort::natural_key;

    #[test]
    fn test_sorts_numbered_pages_in_natural_order() {
        let mut pages = vec!["page10.jpg", "page2.jpg", "page001.jpg"];
        pages.sort_by_key(|page| natural_key(page));

        assert_eq!(pages, vec!["page001.jpg", "page2.jpg", "page10.jpg"]);
    }

    #[test]
    fn test_page_range_respects_bounds() {
        assert_eq!(window_indices(0, 5, 2), vec![0, 1, 2]);
        assert_eq!(window_indices(4, 5, 2), vec![2, 3, 4]);
    }

    #[test]
    fn test_page_range_with_zero_total_returns_empty() {
        assert_eq!(window_indices(0, 0, 2), Vec::<usize>::new());
    }
}
