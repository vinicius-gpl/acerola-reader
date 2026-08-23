use std::{
    fs::File,
    io::{Cursor, Write},
    path::PathBuf,
    sync::OnceLock,
    time::Instant,
};

use image::{codecs::jpeg::JpegEncoder, ImageEncoder};
use pdfium_render::prelude::*;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::infra::error::ComicError;

static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

fn get_pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(Pdfium::default)
}

pub struct ConverterService;

impl ConverterService {
    pub fn new() -> Self {
        Self
    }

    /// Converte um arquivo PDF localizado em `pdf_path` para um arquivo `.cbz`.
    pub async fn convert_pdf_to_cbz(&self, pdf_path: PathBuf) -> Result<PathBuf, ComicError> {
        let cbz_path = pdf_path.with_extension("cbz");

        if cbz_path.exists() {
            return Ok(cbz_path);
        }

        let pdf_path_clone = pdf_path.clone();
        let cbz_path_clone = cbz_path.clone();

        tokio::task::spawn_blocking(move || {
            let pdfium = get_pdfium();
            let document = pdfium.load_pdf_from_file(&pdf_path_clone, None).map_err(|error| {
                ComicError::SystemFailure(format!("Failed to load PDF: {}", error))
            })?;

            let temp_path = cbz_path_clone.with_extension("cbz.tmp");
            let _ = std::fs::remove_file(&temp_path);

            let archive_file = File::create(&temp_path).map_err(ComicError::Io)?;
            let mut zip_writer = ZipWriter::new(archive_file);

            let zip_options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

            let page_count = document.pages().len();
            let render_scale = 2.0;
            let started_at = Instant::now();

            tracing::info!(
                pdf = %pdf_path_clone.to_string_lossy(),
                total_pages = page_count,
                "Starting PDF to CBZ conversion"
            );

            let mut image_buffer = Vec::with_capacity(1024 * 1024 * 2);

            for (page_index, page) in document.pages().iter().enumerate() {
                if page_index == 0 || page_index + 1 == page_count as usize || page_index % 25 == 0
                {
                    tracing::info!(
                        pdf = %pdf_path_clone.to_string_lossy(),
                        page = page_index + 1,
                        total_pages = page_count,
                        elapsed_ms = started_at.elapsed().as_millis(),
                        "Rendering PDF page"
                    );
                }

                let bitmap = page
                    .render_with_config(
                        &PdfRenderConfig::new()
                            .set_target_width((page.width().value * render_scale) as i32)
                            .set_target_height((page.height().value * render_scale) as i32),
                    )
                    .map_err(|error| {
                        ComicError::SystemFailure(format!(
                            "Failed to render page {}: {}",
                            page_index, error
                        ))
                    })?;

                let rgba_data = bitmap.as_rgba_bytes();
                let target_width = bitmap.width();
                let target_height = bitmap.height();

                image_buffer.clear();

                let rgb_data: Vec<u8> = rgba_data
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|px| [px[0], px[1], px[2]])
                    .collect();

                let encoder = JpegEncoder::new_with_quality(Cursor::new(&mut image_buffer), 90);

                encoder
                    .write_image(
                        &rgb_data,
                        target_width as u32,
                        target_height as u32,
                        image::ExtendedColorType::Rgb8,
                    )
                    .map_err(|error| {
                        ComicError::SystemFailure(format!("Failed to encode JPEG: {}", error))
                    })?;

                let entry_name = format!("{:03}.jpg", page_index + 1);
                zip_writer.start_file(entry_name, zip_options).map_err(|error| {
                    ComicError::SystemFailure(format!("Failed to start zip entry: {}", error))
                })?;

                zip_writer.write_all(&image_buffer).map_err(ComicError::Io)?;
            }

            zip_writer.finish().map_err(|error| {
                ComicError::SystemFailure(format!("Failed to finalize CBZ archive: {}", error))
            })?;

            match std::fs::rename(&temp_path, &cbz_path_clone) {
                Ok(_) => {},
                Err(_error) if cbz_path_clone.exists() => {
                    std::fs::remove_file(&cbz_path_clone).map_err(ComicError::Io)?;
                    std::fs::rename(&temp_path, &cbz_path_clone).map_err(|rename_error| {
                        ComicError::SystemFailure(format!("Failed to rename CBZ: {}", rename_error))
                    })?;
                },
                Err(error) => return Err(ComicError::Io(error)),
            }

            tracing::info!(
                pdf = %pdf_path_clone.to_string_lossy(),
                cbz = %cbz_path_clone.to_string_lossy(),
                elapsed_ms = started_at.elapsed().as_millis(),
                "Finished PDF to CBZ conversion"
            );

            Ok(cbz_path_clone)
        })
        .await
        .map_err(|error| {
            ComicError::SystemFailure(format!("Conversion task panicked: {}", error))
        })?
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[tokio::test]
    #[ignore = "requires pdfium DLL which is not available in CI"]
    async fn test_convert_pdf_to_cbz() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

        let converter = ConverterService::new();
        let pdf_path = std::path::Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("tests/wdio/comic/pdf/witchcraft.pdf");

        let cbz_path = pdf_path.with_extension("cbz");
        if cbz_path.exists() {
            std::fs::remove_file(&cbz_path).unwrap();
        }

        let result = converter.convert_pdf_to_cbz(pdf_path).await;
        assert!(result.is_ok(), "Conversion failed: {:?}", result.err());

        let file = std::fs::File::open(&cbz_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).expect("Failed to read generated ZIP");
        assert!(!archive.is_empty(), "ZIP archive is empty");
        let mut first_file = archive.by_index(0).unwrap();
        assert_eq!(first_file.name(), "001.jpg");

        let mut bytes = Vec::new();
        first_file.read_to_end(&mut bytes).unwrap();
        assert!(!bytes.is_empty(), "First rendered page is empty");
    }
}
