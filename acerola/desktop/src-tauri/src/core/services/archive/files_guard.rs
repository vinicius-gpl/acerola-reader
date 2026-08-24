use std::path::Path;

use crate::infra::{
    error::FileError,
    pattern::{archive_format::ArchiveFormat, image_file_format::ImageFileFormat},
};

pub struct SupportedFileGuard;
pub struct ArchiveFileGuard;
pub struct MetadataFileGuard;
pub struct ArtworkFileGuard;

pub trait FileGuard: Send + Sync {
    fn is_allowed(&self, path: &Path) -> Result<(), FileError>;
}

impl FileGuard for SupportedFileGuard {
    fn is_allowed(&self, path: &Path) -> Result<(), FileError> {
        let ext =
            path.extension().and_then(|ext| ext.to_str()).ok_or(FileError::MissingExtension)?;

        match ArchiveFormat::from_extension(ext) {
            Some(_) => Ok(()),
            None => Err(FileError::ExtensionNotAllowed(ext.to_string())),
        }
    }
}

impl FileGuard for ArchiveFileGuard {
    fn is_allowed(&self, path: &Path) -> Result<(), FileError> {
        let ext =
            path.extension().and_then(|ext| ext.to_str()).ok_or(FileError::MissingExtension)?;

        match ArchiveFormat::from_extension(ext) {
            Some(ArchiveFormat::Pdf) | None => Err(FileError::ExtensionNotAllowed(ext.to_string())),
            Some(_) => Ok(()),
        }
    }
}

impl FileGuard for MetadataFileGuard {
    fn is_allowed(&self, path: &Path) -> Result<(), FileError> {
        let name =
            path.file_name().and_then(|name| name.to_str()).ok_or(FileError::MissingFileName)?;

        if name.eq_ignore_ascii_case("ComicInfo.xml") {
            Ok(())
        } else {
            Err(FileError::FileNameNotAllowed(name.to_string()))
        }
    }
}

impl FileGuard for ArtworkFileGuard {
    fn is_allowed(&self, path: &Path) -> Result<(), FileError> {
        let name =
            path.file_name().and_then(|name| name.to_str()).ok_or(FileError::MissingFileName)?;
        let stem =
            path.file_stem().and_then(|name| name.to_str()).ok_or(FileError::MissingFileName)?;

        match (stem, ImageFileFormat::from_path(path)) {
            ("cover" | "banner", Some(_)) => Ok(()),
            _ => Err(FileError::FileNameNotAllowed(name.to_string())),
        }
    }
}

impl ArtworkFileGuard {
    pub fn is_cover(&self, path: &Path) -> bool {
        if self.is_allowed(path).is_err() {
            return false;
        }

        is_named_artwork(path, "cover")
    }

    pub fn is_banner(&self, path: &Path) -> bool {
        if self.is_allowed(path).is_err() {
            return false;
        }

        is_named_artwork(path, "banner")
    }
}

fn is_named_artwork(path: &Path, expected_stem: &str) -> bool {
    let stem = path.file_stem().and_then(|file_name| file_name.to_str()).unwrap_or("");

    stem == expected_stem && ImageFileFormat::from_path(path).is_some()
}

pub struct ScannerGuard {
    guards: Vec<Box<dyn FileGuard>>,
}

impl ScannerGuard {
    pub fn new() -> Self {
        Self {
            guards: vec![
                Box::new(SupportedFileGuard),
                Box::new(MetadataFileGuard),
                Box::new(ArtworkFileGuard),
            ],
        }
    }

    pub fn is_allowed(&self, path: &Path) -> Result<(), FileError> {
        let all_rejected = self.guards.iter().all(|guard| guard.is_allowed(path).is_err());

        if all_rejected {
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("Unknown");

            return Err(FileError::not_allowed(name));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        is_named_artwork, ArchiveFileGuard, ArtworkFileGuard, FileGuard, MetadataFileGuard,
        ScannerGuard, SupportedFileGuard,
    };
    use crate::infra::error::FileError;

    #[test]
    fn test_comic_valid_extension() {
        let guard = SupportedFileGuard;
        assert!(guard.is_allowed(Path::new("berserk.cbz")).is_ok());
        assert!(guard.is_allowed(Path::new("berserk.cbr")).is_ok());
        assert!(guard.is_allowed(Path::new("berserk.pdf")).is_ok());
    }

    #[test]
    fn test_comic_invalid_extension() {
        let guard = SupportedFileGuard;
        let result = guard.is_allowed(Path::new("berserk.exe"));
        assert!(matches!(result, Err(FileError::ExtensionNotAllowed(ext)) if ext == "exe"));
    }

    #[test]
    fn test_comic_missing_extension() {
        let guard = SupportedFileGuard;
        assert!(matches!(guard.is_allowed(Path::new("berserk")), Err(FileError::MissingExtension)));
    }

    #[test]
    fn test_metadata_valid_name() {
        let guard = MetadataFileGuard;
        assert!(guard.is_allowed(Path::new("ComicInfo.xml")).is_ok());
    }

    #[test]
    fn test_metadata_invalid_name() {
        let guard = MetadataFileGuard;
        let result = guard.is_allowed(Path::new("info.xml"));
        assert!(matches!(result, Err(FileError::FileNameNotAllowed(name)) if name == "info.xml"));
    }

    #[test]
    fn test_archive_valid_extension() {
        let guard = ArchiveFileGuard;
        assert!(guard.is_allowed(Path::new("berserk.cbz")).is_ok());
        assert!(guard.is_allowed(Path::new("berserk.cbr")).is_ok());
    }

    #[test]
    fn test_archive_rejects_pdf() {
        let guard = ArchiveFileGuard;
        let result = guard.is_allowed(Path::new("berserk.pdf"));
        assert!(matches!(result, Err(FileError::ExtensionNotAllowed(ext)) if ext == "pdf"));
    }

    #[test]
    fn test_archive_rejects_unknown_extension() {
        let guard = ArchiveFileGuard;
        let result = guard.is_allowed(Path::new("berserk.exe"));
        assert!(matches!(result, Err(FileError::ExtensionNotAllowed(ext)) if ext == "exe"));
    }

    #[test]
    fn test_artwork_is_banner() {
        let guard = ArtworkFileGuard;
        assert!(guard.is_banner(Path::new("banner.png")));
        assert!(!guard.is_banner(Path::new("cover.png")));
    }

    #[test]
    fn test_artwork_is_cover() {
        let guard = ArtworkFileGuard;
        assert!(guard.is_cover(Path::new("cover.png")));
        assert!(!guard.is_cover(Path::new("banner.png")));
    }

    #[test]
    fn test_is_named_artwork_requires_matching_stem() {
        assert!(is_named_artwork(Path::new("banner.png"), "banner"));
        assert!(!is_named_artwork(Path::new("banner.png"), "cover"));
        assert!(!is_named_artwork(Path::new("banner.txt"), "banner"));
    }

    #[test]
    fn test_artwork_valid_names() {
        let guard = ArtworkFileGuard;
        assert!(guard.is_allowed(Path::new("cover.png")).is_ok());
        assert!(guard.is_allowed(Path::new("cover.jpg")).is_ok());
        assert!(guard.is_allowed(Path::new("cover.jpeg")).is_ok());
        assert!(guard.is_allowed(Path::new("banner.png")).is_ok());
        assert!(guard.is_allowed(Path::new("banner.jpg")).is_ok());
        assert!(guard.is_allowed(Path::new("banner.jpeg")).is_ok());
    }

    #[test]
    fn test_artwork_invalid_name() {
        let guard = ArtworkFileGuard;
        let result = guard.is_allowed(Path::new("thumbnail.png"));
        assert!(
            matches!(result, Err(FileError::FileNameNotAllowed(name)) if name == "thumbnail.png")
        );
    }

    #[test]
    fn test_scanner_accepts_comic() {
        let guard = ScannerGuard::new();
        assert!(guard.is_allowed(Path::new("berserk.cbz")).is_ok());
    }

    #[test]
    fn test_scanner_accepts_metadata() {
        let guard = ScannerGuard::new();
        assert!(guard.is_allowed(Path::new("ComicInfo.xml")).is_ok());
    }

    #[test]
    fn test_scanner_accepts_artwork() {
        let guard = ScannerGuard::new();
        assert!(guard.is_allowed(Path::new("cover.png")).is_ok());
    }

    #[test]
    fn test_scanner_rejects_unknown_file() {
        let guard = ScannerGuard::new();
        let result = guard.is_allowed(Path::new("script.sh"));
        assert!(matches!(result, Err(FileError::NotAllowed(_))));
    }
}
