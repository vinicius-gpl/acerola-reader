use std::{fs::File, io::Read, path::Path};

use zip::ZipArchive;

use crate::infra::{
    error::ComicError,
    pattern::{
        archive_format::ArchiveFormat, image_file_format::ImageFileFormat,
        natural_sort::natural_cmp,
    },
};

/// Extrai os bytes da primeira página (em ordem natural) de um arquivo de capítulo CBZ/CBR.
///
/// Usado pela geração manual de capa (ver [`crate::core::services::comic::ComicService`]) quando
/// o usuário opta por usar a página do capítulo como capa do quadrinho/volume.
/// Não deve ser chamado a partir de contexto async diretamente — é uma operação bloqueante de I/O
/// (mesma razão pela qual o `ReaderService` roda suas leituras de arquivo em `spawn_blocking`).
pub fn extract_first_page(archive_path: &Path) -> Result<(Vec<u8>, ImageFileFormat), ComicError> {
    let extension = archive_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ArchiveFormat::from_extension(&extension) {
        Some(ArchiveFormat::Cbz) => extract_first_page_cbz(archive_path),
        Some(ArchiveFormat::Cbr) => extract_first_page_cbr(archive_path),
        _ => Err(ComicError::SystemFailure(format!(
            "Unsupported archive format for cover extraction: {extension}"
        ))),
    }
}

/// Persiste os bytes extraídos como `cover.{jpg,png}` dentro do diretório informado,
/// substituindo qualquer `cover.jpg`/`cover.jpeg`/`cover.png` pré-existente na pasta
/// (inclusive com uma extensão diferente da gerada agora, para não deixar arquivo órfão).
pub async fn persist_cover(
    directory: &Path, bytes: Vec<u8>, format: ImageFileFormat,
) -> Result<String, ComicError> {
    let extension = match format {
        ImageFileFormat::Jpeg => "jpg",
        ImageFileFormat::Png => "png",
    };
    let cover_path = directory.join(format!("cover.{extension}"));

    for stale_extension in ["jpg", "jpeg", "png"] {
        if stale_extension == extension {
            continue;
        }

        let stale_path = directory.join(format!("cover.{stale_extension}"));
        let _ = tokio::fs::remove_file(&stale_path).await;
    }

    tokio::fs::write(&cover_path, bytes).await?;

    Ok(cover_path.to_string_lossy().to_string())
}

fn extract_first_page_cbz(path: &Path) -> Result<(Vec<u8>, ImageFileFormat), ComicError> {
    let file = File::open(path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| ComicError::SystemFailure(error.to_string()))?;

    let mut names: Vec<String> = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| ComicError::SystemFailure(error.to_string()))?;

        if entry.is_file() && ImageFileFormat::from_path(Path::new(entry.name())).is_some() {
            names.push(entry.name().to_string());
        }
    }
    names.sort_by(|left, right| natural_cmp(left, right));

    let name = names
        .first()
        .ok_or_else(|| ComicError::SystemFailure("CBZ has no image pages".to_string()))?;
    let format = ImageFileFormat::from_path(Path::new(name))
        .ok_or_else(|| ComicError::SystemFailure("Unsupported page image format".to_string()))?;

    let mut entry =
        archive.by_name(name).map_err(|error| ComicError::SystemFailure(error.to_string()))?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;

    Ok((bytes, format))
}

fn extract_first_page_cbr(path: &Path) -> Result<(Vec<u8>, ImageFileFormat), ComicError> {
    let listing = unrar::Archive::new(path)
        .open_for_listing()
        .map_err(|error| ComicError::SystemFailure(error.to_string()))?;

    let mut names = Vec::new();
    for entry in listing {
        let entry = entry.map_err(|error| ComicError::SystemFailure(error.to_string()))?;

        if entry.is_file() && ImageFileFormat::from_path(&entry.filename).is_some() {
            names.push(entry.filename);
        }
    }
    names.sort_by(|left, right| natural_cmp(&left.to_string_lossy(), &right.to_string_lossy()));

    let target = names
        .first()
        .cloned()
        .ok_or_else(|| ComicError::SystemFailure("CBR has no image pages".to_string()))?;
    let format = ImageFileFormat::from_path(&target)
        .ok_or_else(|| ComicError::SystemFailure("Unsupported page image format".to_string()))?;

    let mut archive = unrar::Archive::new(path)
        .open_for_processing()
        .map_err(|error| ComicError::SystemFailure(error.to_string()))?;

    loop {
        let Some(header) =
            archive.read_header().map_err(|error| ComicError::SystemFailure(error.to_string()))?
        else {
            break;
        };

        if header.entry().filename == target {
            let (bytes, _) =
                header.read().map_err(|error| ComicError::SystemFailure(error.to_string()))?;
            return Ok((bytes, format));
        }

        archive = header.skip().map_err(|error| ComicError::SystemFailure(error.to_string()))?;
    }

    Err(ComicError::SystemFailure("Page entry missing in CBR".to_string()))
}
