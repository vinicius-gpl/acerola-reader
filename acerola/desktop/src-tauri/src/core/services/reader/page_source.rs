use std::{
    fs::File,
    io::Read,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use zip::ZipArchive;

use crate::infra::{
    error::ReaderError,
    pattern::{
        archive_format::ArchiveFormat, image_file_format::ImageFileFormat,
        natural_sort::natural_cmp,
    },
};

pub(super) struct RawPage {
    pub(super) name: String,
    pub(super) bytes: Vec<u8>,
}

pub(super) trait PageSource: Send {
    fn page_count(&self) -> NonZeroUsize;
    fn read_page(&self, index: usize) -> Result<RawPage, ReaderError>;
}

struct CbzPageSource {
    path: PathBuf,
    entries: Vec<String>,
    page_count: NonZeroUsize,
}

impl CbzPageSource {
    fn open(path: &Path) -> Result<Self, ReaderError> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut entries = Vec::new();

        for index in 0..archive.len() {
            let entry = archive.by_index(index)?;
            let name = entry.name().to_string();

            if entry.is_file() && is_supported_page_image_name(&name) {
                entries.push(name);
            }
        }

        sort_page_names(&mut entries);
        let page_count = page_count_from_len(path, entries.len())?;

        Ok(Self { path: path.to_path_buf(), entries, page_count })
    }
}

impl PageSource for CbzPageSource {
    fn page_count(&self) -> NonZeroUsize {
        self.page_count
    }

    fn read_page(&self, index: usize) -> Result<RawPage, ReaderError> {
        let name = self
            .entries
            .get(index)
            .ok_or(ReaderError::PageOutOfBounds { index, total: self.page_count.get() })?;

        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut entry = archive.by_name(name).map_err(|err| match err {
            zip::result::ZipError::FileNotFound => {
                ReaderError::PageEntryMissing { index, name: name.clone() }
            },
            err => ReaderError::from(err),
        })?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;

        Ok(RawPage { name: name.clone(), bytes })
    }
}

struct CbrPageSource {
    path: PathBuf,
    entries: Vec<PathBuf>,
    page_count: NonZeroUsize,
}

impl CbrPageSource {
    fn open(path: &Path) -> Result<Self, ReaderError> {
        let archive = unrar::Archive::new(path).open_for_listing()?;
        let mut entries = Vec::new();

        for entry in archive {
            let entry = entry?;

            if entry.is_file() && is_supported_page_image_path(&entry.filename) {
                entries.push(entry.filename);
            }
        }

        sort_page_paths(&mut entries);
        let page_count = page_count_from_len(path, entries.len())?;

        Ok(Self { path: path.to_path_buf(), entries, page_count })
    }
}

impl PageSource for CbrPageSource {
    fn page_count(&self) -> NonZeroUsize {
        self.page_count
    }

    fn read_page(&self, index: usize) -> Result<RawPage, ReaderError> {
        let target = self
            .entries
            .get(index)
            .ok_or(ReaderError::PageOutOfBounds { index, total: self.page_count.get() })?;

        let mut archive = unrar::Archive::new(&self.path).open_for_processing()?;

        loop {
            let Some(entry) = archive.read_header()? else {
                break;
            };

            if &entry.entry().filename == target {
                let name = target.to_string_lossy().to_string();
                let (bytes, _) = entry.read()?;
                return Ok(RawPage { name, bytes });
            }

            archive = entry.skip()?;
        }

        Err(ReaderError::PageEntryMissing { index, name: target.to_string_lossy().to_string() })
    }
}

pub(super) fn source_from_path(path: &Path) -> Result<Box<dyn PageSource>, ReaderError> {
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            path.to_string_lossy().to_string(),
        )
        .into());
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ArchiveFormat::from_extension(&extension) {
        Some(ArchiveFormat::Cbz) => Ok(Box::new(CbzPageSource::open(path)?)),
        Some(ArchiveFormat::Cbr) => Ok(Box::new(CbrPageSource::open(path)?)),
        Some(ArchiveFormat::Pdf) => Err(ReaderError::UnsupportedFormat(extension)),
        None => Err(ReaderError::UnsupportedFormat(extension)),
    }
}

fn is_supported_page_image_name(name: &str) -> bool {
    ImageFileFormat::from_path(Path::new(name)).is_some()
}

fn is_supported_page_image_path(path: &Path) -> bool {
    ImageFileFormat::from_path(path).is_some()
}

fn page_count_from_len(path: &Path, len: usize) -> Result<NonZeroUsize, ReaderError> {
    NonZeroUsize::new(len)
        .ok_or_else(|| ReaderError::EmptyChapter(path.to_string_lossy().to_string()))
}

fn sort_page_names(entries: &mut [String]) {
    entries.sort_by(|left, right| natural_cmp(left, right));
}

fn sort_page_paths(entries: &mut [PathBuf]) {
    entries.sort_by(|left, right| natural_cmp(&left.to_string_lossy(), &right.to_string_lossy()));
}

pub(super) fn mime_type_for(name: &str) -> Result<String, ReaderError> {
    ImageFileFormat::from_path(Path::new(name))
        .map(|format| format.mime_type().to_string())
        .ok_or_else(|| ReaderError::Image(name.to_string()))
}
