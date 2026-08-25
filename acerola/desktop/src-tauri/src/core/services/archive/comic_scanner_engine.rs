use std::{
    collections::{HashMap, HashSet},
    future::Future,
    path::PathBuf,
};

use futures::stream::{self, StreamExt};
use tokio::{fs, sync::mpsc};

use crate::{
    core::services::archive::{
        chapter_scanner_engine::ChapterScannerService,
        converter::ConverterService,
        files_guard::{ArchiveFileGuard, ArtworkFileGuard, FileGuard, ScannerGuard},
        path_guard::{path_hash, PathGuard},
    },
    data::{
        models::archive::{
            archive_template::{ArchiveTemplate, SortType},
            chapter_archive::ChapterArchive,
            comic_directory::ComicDirectory,
            volume_archive::VolumeArchive,
        },
        repositories::archive::{
            archive_template_repo::ArchiveTemplateRepository,
            comic_directory_repo::ComicRepository, volume_archive_repo::VolumeRepository,
        },
    },
    infra::{
        error::{ComicError, DbError},
        fs::{DirectoryEntry, ScannerEngine},
        pattern::{
            archive_format::ArchiveFormat,
            template::{detect_template, extract_chapter_parts},
            template_validator::validate_chapter_template,
        },
    },
};

/// Orquestrador do escaneamento de bibliotecas de quadrinhos no sistema de arquivos.
///
/// Este serviço é responsável por percorrer diretórios, identificar pastas que representam
/// obras (Comics), gerenciar capítulos e volumes, além de lidar com a conversão de formatos.
///
/// ### IDs Determinísticos
/// Utiliza IDs baseados em hash do caminho relativo para garantir consistência entre scans
/// e facilitar futuras sincronizações sem depender do estado do banco de dados.
///
/// ### Fluxo de Conversão
/// Como parte do middleware de escaneamento, arquivos PDF são automaticamente interceptados,
/// convertidos para CBZ e apenas o resultado da conversão é indexado, mantendo o banco
/// de dados limpo com formatos otimizados para leitura.
pub struct ComicScannerService {
    path_guard: PathGuard,
    comic_repo: ComicRepository,
    chapter_scanner: ChapterScannerService,
    template_repo: ArchiveTemplateRepository,
    volume_repo: VolumeRepository,
    converter: ConverterService,
}

const PDF_CONVERSION_CONCURRENCY: usize = 3;

impl ComicScannerService {
    pub fn new(root: PathBuf, pool: sqlx::SqlitePool) -> Self {
        Self {
            path_guard: PathGuard::new(root),
            comic_repo: ComicRepository::new(pool.clone()),
            chapter_scanner: ChapterScannerService::new(pool.clone()),
            template_repo: ArchiveTemplateRepository::new(pool.clone()),
            volume_repo: VolumeRepository::new(pool.clone()),
            converter: ConverterService::new(),
        }
    }

    /// Executa um escaneamento completo (Refresh), indexando novas pastas e capítulos.
    ///
    /// Utiliza a estratégia `INSERT OR IGNORE`, garantindo que itens já existentes
    /// não sejam duplicados nem reprocessados desnecessariamente.
    pub async fn refresh_library(
        &self, path: PathBuf, mut on_progress: impl FnMut(String),
        mut on_converting: impl FnMut(String),
    ) -> Result<(), ComicError> {
        self.path_guard.execute(&path, |_guard_path| -> Result<(), String> { Ok(()) })?;

        let templates = self.template_repo.base.find_all().await?;
        let mut entries = self.collect_entries(path).await?;
        entries.sort_by_key(|entry| entry.directory.clone());

        let indexed: Vec<ComicDirectory> = self.comic_repo.base.find_all().await?;
        let discovered_paths: HashSet<String> =
            entries.iter().map(|entry| entry.directory.to_string_lossy().to_string()).collect();

        for comic in &indexed {
            if !discovered_paths.contains(&comic.path) {
                self.comic_repo.base.delete(comic.id).await?;
            }
        }

        // Converte todos os PDFs de todos os diretórios em paralelo antes do scan
        self.pre_convert_all_pdfs(&entries, &mut on_converting).await;

        let repository = self.comic_repo.clone();
        let mut processed_paths: Vec<String> = vec![];

        for entry in entries {
            let directory_path = entry.directory.to_string_lossy().to_string();

            // Pula se o diretório atual for subdiretório de algum já processado como comic
            if processed_paths.iter().any(|processed| {
                std::path::Path::new(&directory_path).starts_with(processed)
                    && directory_path != *processed
            }) {
                continue;
            }

            let repo_clone = repository.clone();
            let is_comic = match self
                .process_entry(entry, &templates, |comic| async move {
                    match repo_clone.base.insert(&comic).await {
                        Ok(saved) => Ok(saved),
                        Err(DbError::UniqueViolation) => Ok(comic),
                        Err(error) => Err(ComicError::from(error)),
                    }
                })
                .await
            {
                Ok(processed_successfully) => processed_successfully,
                Err(processing_error) => {
                    tracing::error!(
                        "Error processing comic in {}: {}",
                        directory_path,
                        processing_error
                    );
                    false
                },
            };

            if is_comic {
                on_progress(directory_path.clone());
                processed_paths.push(directory_path);
            }
        }

        Ok(())
    }

    /// Realiza um escaneamento incremental, detectando apenas modificações no disco.
    ///
    /// # Comportamento
    /// - Adiciona novos quadrinhos.
    /// - Atualiza quadrinhos cujas pastas tiveram data de modificação alterada.
    /// - Remove do banco de dados registros cujas pastas não existem mais no disco.
    pub async fn incremental_scan(
        &self, path: PathBuf, mut on_progress: impl FnMut(String),
        mut on_converting: impl FnMut(String),
    ) -> Result<(), ComicError> {
        self.path_guard.execute(&path, |_guard_path| -> Result<(), String> { Ok(()) })?;

        let templates = self.template_repo.base.find_all().await?;
        let mut discovered = self.collect_entries(path).await?;
        discovered.sort_by_key(|entry| entry.directory.clone());

        // Converte todos os PDFs de todos os diretórios em paralelo antes do scan
        self.pre_convert_all_pdfs(&discovered, &mut on_converting).await;

        let indexed: Vec<ComicDirectory> = self.comic_repo.base.find_all().await?;
        let repository = self.comic_repo.clone();

        let indexed_map: HashMap<String, &ComicDirectory> =
            indexed.iter().map(|comic| (comic.path.clone(), comic)).collect();

        let discovered_paths: HashSet<String> =
            discovered.iter().map(|entry| entry.directory.to_string_lossy().to_string()).collect();

        for comic in &indexed {
            if !discovered_paths.contains(&comic.path) {
                self.comic_repo.base.delete(comic.id).await?;
            }
        }

        let mut processed_paths: Vec<String> = vec![];

        for entry in discovered {
            let directory_path = entry.directory.to_string_lossy().to_string();

            if processed_paths.iter().any(|processed| {
                std::path::Path::new(&directory_path).starts_with(processed)
                    && directory_path != *processed
            }) {
                continue;
            }

            let directory_metadata = fs::metadata(&entry.directory).await?;
            let disk_modified = modified_secs(&directory_metadata);

            let existing_comic = indexed_map.get(&directory_path);
            let needs_processing = match existing_comic {
                None => true,
                Some(comic) => comic.last_modified < disk_modified,
            };

            let is_comic = if needs_processing {
                let repo_clone = repository.clone();
                match self
                    .process_entry(entry, &templates, |comic| async move {
                        match repo_clone.base.insert(&comic).await {
                            Ok(saved) => Ok(saved),
                            Err(DbError::UniqueViolation) => {
                                repo_clone.base.update(&comic).await.map_err(ComicError::from)
                            },
                            Err(error) => Err(ComicError::from(error)),
                        }
                    })
                    .await
                {
                    Ok(processed_successfully) => {
                        if processed_successfully {
                            on_progress(directory_path.clone());
                        }
                        processed_successfully
                    },
                    Err(processing_error) => {
                        tracing::error!(
                            "Error processing comic in {}: {}",
                            directory_path,
                            processing_error
                        );
                        false
                    },
                }
            } else {
                true
            };

            if is_comic {
                processed_paths.push(directory_path);
            }
        }

        Ok(())
    }

    /// Reconstrói a biblioteca do zero, ignorando o estado atual do banco de dados.
    ///
    /// Utiliza `DELETE` seguido de `INSERT` para todos os itens encontrados, o que
    /// força a re-verificação de todos os arquivos e a atualização de todos os metadados.
    pub async fn rebuild_library(
        &self, path: PathBuf, mut on_progress: impl FnMut(String),
        mut on_converting: impl FnMut(String),
    ) -> Result<(), ComicError> {
        self.path_guard.execute(&path, |_guard_path| -> Result<(), String> { Ok(()) })?;

        let templates = self.template_repo.base.find_all().await?;
        let mut entries = self.collect_entries(path).await?;
        entries.sort_by_key(|entry| entry.directory.clone());

        let indexed: Vec<ComicDirectory> = self.comic_repo.base.find_all().await?;
        let discovered_paths: HashSet<String> =
            entries.iter().map(|entry| entry.directory.to_string_lossy().to_string()).collect();

        for comic in &indexed {
            if !discovered_paths.contains(&comic.path) {
                self.comic_repo.base.delete(comic.id).await?;
            }
        }

        // Converts all PDFs from all directories in parallel before the scan
        self.pre_convert_all_pdfs(&entries, &mut on_converting).await;

        let repository = self.comic_repo.clone();
        let mut processed_paths: Vec<String> = vec![];

        for entry in entries {
            let directory_path = entry.directory.to_string_lossy().to_string();

            if processed_paths.iter().any(|processed| {
                std::path::Path::new(&directory_path).starts_with(processed)
                    && directory_path != *processed
            }) {
                continue;
            }

            let repo_clone = repository.clone();
            let is_comic = match self
                .process_entry(entry, &templates, |comic| async move {
                    repo_clone.base.delete(comic.id).await?;
                    repo_clone.base.insert(&comic).await.map_err(ComicError::from)
                })
                .await
            {
                Ok(processed_successfully) => processed_successfully,
                Err(processing_error) => {
                    tracing::error!(
                        "Error processing comic in {}: {}",
                        directory_path,
                        processing_error
                    );
                    false
                },
            };

            if is_comic {
                on_progress(directory_path.clone());
                processed_paths.push(directory_path);
            }
        }

        Ok(())
    }

    /// Reescaneia pontualmente um único quadrinho já indexado, sem comparar contra o
    /// restante da biblioteca — apenas insere/atualiza o que for encontrado na pasta.
    ///
    /// Diferente de [`Self::incremental_scan`], não remove capítulos cujo arquivo tenha
    /// sumido do disco (mesma limitação que o scan de biblioteca inteira já tem hoje para
    /// pastas que continuam existindo). Para invalidar tudo, veja [`Self::deep_rescan_comic`].
    pub async fn rescan_comic(
        &self, comic: ComicDirectory, on_progress: impl FnMut(String),
        on_converting: impl FnMut(String),
    ) -> Result<(), ComicError> {
        self.scan_comic_path(PathBuf::from(&comic.path), on_progress, on_converting).await
    }

    /// Invalida e reescaneia um único quadrinho do zero: apaga seus capítulos e volumes
    /// indexados antes de re-processar a pasta, garantindo que nada órfão sobre no banco.
    pub async fn deep_rescan_comic(
        &self, comic: ComicDirectory, on_progress: impl FnMut(String),
        on_converting: impl FnMut(String),
    ) -> Result<(), ComicError> {
        self.chapter_scanner.delete_by_comic(comic.id).await?;
        self.volume_repo.delete_by_comic(comic.id).await?;

        self.scan_comic_path(PathBuf::from(&comic.path), on_progress, on_converting).await
    }

    /// Núcleo compartilhado de [`Self::rescan_comic`] e [`Self::deep_rescan_comic`]: processa
    /// apenas as entradas encontradas dentro de `path`, sem tocar em quadrinhos de fora dele.
    async fn scan_comic_path(
        &self, path: PathBuf, mut on_progress: impl FnMut(String),
        mut on_converting: impl FnMut(String),
    ) -> Result<(), ComicError> {
        self.path_guard.execute(&path, |_guard_path| -> Result<(), String> { Ok(()) })?;

        let templates = self.template_repo.base.find_all().await?;
        let mut entries = self.collect_entries(path).await?;
        entries.sort_by_key(|entry| entry.directory.clone());

        self.pre_convert_all_pdfs(&entries, &mut on_converting).await;

        let repository = self.comic_repo.clone();
        let mut processed_paths: Vec<String> = vec![];

        for entry in entries {
            let directory_path = entry.directory.to_string_lossy().to_string();

            if processed_paths.iter().any(|processed| {
                std::path::Path::new(&directory_path).starts_with(processed)
                    && directory_path != *processed
            }) {
                continue;
            }

            let repo_clone = repository.clone();
            let is_comic = match self
                .process_entry(entry, &templates, |comic| async move {
                    match repo_clone.base.insert(&comic).await {
                        Ok(saved) => Ok(saved),
                        Err(DbError::UniqueViolation) => {
                            repo_clone.base.update(&comic).await.map_err(ComicError::from)
                        },
                        Err(error) => Err(ComicError::from(error)),
                    }
                })
                .await
            {
                Ok(processed_successfully) => processed_successfully,
                Err(processing_error) => {
                    tracing::error!(
                        "Error processing comic in {}: {}",
                        directory_path,
                        processing_error
                    );
                    false
                },
            };

            if is_comic {
                on_progress(directory_path.clone());
                processed_paths.push(directory_path);
            }
        }

        Ok(())
    }

    /// Coleta entradas de diretório recursivamente através do ScannerEngine.
    async fn collect_entries(&self, path: PathBuf) -> Result<Vec<DirectoryEntry>, ComicError> {
        let (tx, mut rx) = mpsc::channel(32);
        let scanner = ScannerEngine { max_depth: None };

        tokio::spawn(async move {
            let guard = ScannerGuard::new();
            if let Err(error) =
                scanner.scan(path, tx, move |entry_path| guard.is_allowed(entry_path).is_ok()).await
            {
                tracing::error!("Scanner engine failure: {}", error);
            }
        });

        let mut entries = Vec::new();
        while let Some(entry) = rx.recv().await {
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Processa uma única entrada de diretório, classificando arquivos e convertendo PDFs.
    ///
    /// Esta é a função principal que aplica a lógica de middleware para interceptar
    /// formatos não suportados nativamente pelo banco de dados (PDF) e transformá-los.
    async fn process_entry<F, Fut>(
        &self, entry: DirectoryEntry, templates: &[ArchiveTemplate], persist: F,
    ) -> Result<bool, ComicError>
    where
        F: FnOnce(ComicDirectory) -> Fut,
        Fut: Future<Output = Result<ComicDirectory, ComicError>>,
    {
        let archive_guard = ArchiveFileGuard;
        let artwork_guard = ArtworkFileGuard;

        let chapter_templates: Vec<&ArchiveTemplate> =
            templates.iter().filter(|template| template.sort_type == SortType::Chapter).collect();

        let volume_templates: Vec<&ArchiveTemplate> =
            templates.iter().filter(|template| template.sort_type == SortType::Volume).collect();

        let mut comic_cover = None;
        let mut comic_banner = None;
        let mut comic_files = Vec::new();
        let mut pdf_files = Vec::new();

        for file in entry.files {
            let is_cover = artwork_guard.is_cover(&file);
            let is_banner = artwork_guard.is_banner(&file);
            let is_archive = archive_guard.is_allowed(&file).is_ok();

            if comic_cover.is_none() && is_cover {
                comic_cover = Some(file.to_string_lossy().to_string());
                continue;
            }

            if comic_banner.is_none() && is_banner {
                comic_banner = Some(file.to_string_lossy().to_string());
                continue;
            }

            if is_archive {
                comic_files.push(file);
                continue;
            }

            let is_pdf = file
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ArchiveFormat::from_extension(ext) == Some(ArchiveFormat::Pdf));

            if is_pdf {
                pdf_files.push(file);
            }
        }

        let converted_files = self.convert_pdf_files(pdf_files, &comic_files, "comic").await;
        comic_files.extend(converted_files);

        let volume_pattern_strs: Vec<&str> =
            volume_templates.iter().map(|template| template.pattern.as_str()).collect();

        let mut matched_volumes = vec![];
        for subdirectory in &entry.subdirs {
            let directory_name =
                subdirectory.file_name().and_then(|name| name.to_str()).unwrap_or("");
            if let Some(pattern) = detect_template(directory_name, &volume_pattern_strs, |_| Ok(()))
            {
                matched_volumes.push((subdirectory, pattern));
            }
        }

        if comic_files.is_empty() && matched_volumes.is_empty() {
            return Ok(false);
        }

        let detected_template = self.detect_template_for(&comic_files, &chapter_templates);
        let template_fk = detected_template.map(|template| template.id);
        let template_pattern = detected_template.map(|template| template.pattern.as_str());

        let directory_metadata = fs::metadata(&entry.directory).await?;
        let directory_name = entry
            .directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let comic = ComicDirectory {
            id: path_hash(&entry.directory),
            name: directory_name,
            path: entry.directory.to_string_lossy().to_string(),
            cover: comic_cover,
            banner: comic_banner,
            last_modified: modified_secs(&directory_metadata),
            archive_template_fk: template_fk,
            external_sync_enabled: true,
            hidden: false,
        };

        let saved_comic = persist(comic).await?;

        for (index, file) in comic_files.iter().enumerate() {
            self.chapter_scanner
                .scan_chapter(file, index, saved_comic.id, None, template_pattern)
                .await?;
        }

        for (subdirectory, pattern) in matched_volumes {
            let directory_name =
                subdirectory.file_name().and_then(|name| name.to_str()).unwrap_or("");

            let volume_sort = extract_chapter_parts(directory_name, pattern, |_| Ok(()))
                .map(|(vol, dec)| ChapterArchive::format_sort(vol, dec))
                .unwrap_or_else(|| directory_name.to_string());

            let subdirectory_metadata = fs::metadata(subdirectory).await?;
            let is_special =
                crate::data::models::archive::chapter_archive::is_special_name(directory_name);

            let volume_files = self.collect_files(subdirectory).await.unwrap_or_default();

            let mut volume_cover = None;
            let mut volume_banner = None;
            let mut volume_archives = Vec::new();
            let mut volume_pdfs = Vec::new();

            for file in volume_files {
                if volume_cover.is_none() && artwork_guard.is_cover(&file) {
                    volume_cover = Some(file.to_string_lossy().to_string());
                    continue;
                }

                if volume_banner.is_none() && artwork_guard.is_banner(&file) {
                    volume_banner = Some(file.to_string_lossy().to_string());
                    continue;
                }

                if archive_guard.is_allowed(&file).is_ok() {
                    volume_archives.push(file);
                    continue;
                }

                let is_pdf = file.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
                    ArchiveFormat::from_extension(ext) == Some(ArchiveFormat::Pdf)
                });

                if is_pdf {
                    volume_pdfs.push(file);
                }
            }

            let converted_files =
                self.convert_pdf_files(volume_pdfs, &volume_archives, "volume").await;
            volume_archives.extend(converted_files);

            let volume_id = path_hash(subdirectory);
            let volume = VolumeArchive {
                id: volume_id,
                name: directory_name.to_string(),
                path: subdirectory.to_string_lossy().to_string(),
                volume_sort,
                is_special,
                cover: volume_cover,
                banner: volume_banner,
                comic_directory_fk: saved_comic.id,
                last_modified: modified_secs(&subdirectory_metadata),
            };

            match self.volume_repo.base.insert(&volume).await {
                Ok(_) | Err(DbError::UniqueViolation) => {},
                Err(error) => return Err(ComicError::from(error)),
            }

            volume_archives.sort();

            for (index, file) in volume_archives.iter().enumerate() {
                self.chapter_scanner
                    .scan_chapter(file, index, saved_comic.id, Some(volume_id), template_pattern)
                    .await?;
            }
        }

        Ok(true)
    }

    /// Coleta arquivos de um diretório específico de forma assíncrona.
    async fn collect_files(&self, directory: &PathBuf) -> Result<Vec<PathBuf>, ComicError> {
        let mut entries = fs::read_dir(directory).await.map_err(ComicError::Io)?;
        let mut files = vec![];

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if entry.metadata().await.map(|meta| meta.is_file()).unwrap_or(false) {
                files.push(path);
            }
        }

        Ok(files)
    }

    async fn convert_pdf_files(
        &self, pdf_files: Vec<PathBuf>, existing_archives: &[PathBuf], scope: &'static str,
    ) -> Vec<PathBuf> {
        let existing_archives: HashSet<PathBuf> = existing_archives.iter().cloned().collect();
        let mut seen_targets: HashSet<PathBuf> = HashSet::new();
        let mut pre_converted: Vec<(usize, PathBuf)> = Vec::new();

        let conversion_jobs: Vec<(usize, PathBuf)> = pdf_files
            .into_iter()
            .enumerate()
            .filter_map(|(index, pdf_path)| {
                let cbz_path = pdf_path.with_extension("cbz");

                if existing_archives.contains(&cbz_path) || !seen_targets.insert(cbz_path.clone()) {
                    return None;
                }

                // Já foi pré-convertido pelo pre_convert_all_pdfs — inclui sem reprocessar
                if cbz_path.exists() {
                    pre_converted.push((index, cbz_path));
                    return None;
                }

                Some((index, pdf_path))
            })
            .collect();

        if conversion_jobs.is_empty() {
            let mut result = pre_converted;
            result.sort_unstable_by_key(|(it, _)| *it);
            return result.into_iter().map(|(_, path)| path).collect();
        }

        tracing::info!(
            scope,
            total_pdfs = conversion_jobs.len(),
            concurrency = PDF_CONVERSION_CONCURRENCY,
            "Starting PDF conversion batch"
        );

        let stream_results =
            stream::iter(conversion_jobs.into_iter().map(|(index, pdf_path)| async move {
                let pdf_label = pdf_path.to_string_lossy().to_string();

                match self.converter.convert_pdf_to_cbz(pdf_path).await {
                    Ok(converted_path) => {
                        tracing::info!(
                            scope,
                            pdf = %pdf_label,
                            cbz = %converted_path.to_string_lossy(),
                            "Finished PDF conversion"
                        );
                        (index, Some(converted_path))
                    },
                    Err(error) => {
                        tracing::warn!(
                            scope,
                            pdf = %pdf_label,
                            error = %error,
                            "Failed to convert PDF to CBZ"
                        );
                        (index, None)
                    },
                }
            }))
            .buffer_unordered(PDF_CONVERSION_CONCURRENCY)
            .collect::<Vec<_>>()
            .await;

        let mut all_results = pre_converted;
        for (index, path_opt) in stream_results {
            if let Some(path) = path_opt {
                all_results.push((index, path));
            }
        }
        all_results.sort_unstable_by_key(|(i, _)| *i);
        all_results.into_iter().map(|(_, p)| p).collect()
    }

    /// Pré-converte todos os PDFs encontrados em TODAS as entradas antes do scan principal.
    ///
    /// Garante que PDFs de diretórios diferentes sejam processados em paralelo
    /// (até [`PDF_CONVERSION_CONCURRENCY`] conversões simultâneas) em vez de aguardar
    /// o processamento sequencial de cada entrada no loop principal.
    async fn pre_convert_all_pdfs(
        &self, entries: &[DirectoryEntry], on_converting: &mut impl FnMut(String),
    ) {
        let mut pdf_paths: HashSet<PathBuf> = HashSet::new();

        let add_pdfs = |files: &[PathBuf], paths: &mut HashSet<PathBuf>| {
            for file in files.iter().filter(|file| {
                file.extension().and_then(|it| it.to_str()).is_some_and(|ext| {
                    ArchiveFormat::from_extension(ext) == Some(ArchiveFormat::Pdf)
                })
            }) {
                if !file.with_extension("cbz").exists() {
                    paths.insert(file.clone());
                }
            }
        };

        for entry in entries {
            add_pdfs(&entry.files, &mut pdf_paths);

            for subdir in &entry.subdirs {
                if let Ok(subdir_files) = self.collect_files(subdir).await {
                    add_pdfs(&subdir_files, &mut pdf_paths);
                }
            }
        }

        if pdf_paths.is_empty() {
            return;
        }

        on_converting(format!("Convertendo {} arquivos para cbz...", pdf_paths.len()));

        tracing::info!(
            total_pdfs = pdf_paths.len(),
            concurrency = PDF_CONVERSION_CONCURRENCY,
            "Pre-converting all PDFs across directories"
        );

        stream::iter(pdf_paths)
            .map(|pdf| self.converter.convert_pdf_to_cbz(pdf))
            .buffer_unordered(PDF_CONVERSION_CONCURRENCY)
            .for_each(|result| async {
                match result {
                    Ok(cbz) => {
                        tracing::info!(cbz = %cbz.to_string_lossy(), "PDF pre-converted")
                    },
                    Err(error) => tracing::warn!(error = %error, "PDF pre-conversion failed"),
                }
            })
            .await;
    }

    fn detect_template_for<'a>(
        &self, files: &[PathBuf], templates: &[&'a ArchiveTemplate],
    ) -> Option<&'a ArchiveTemplate> {
        files
            .first()
            .and_then(|file| file.file_name())
            .and_then(|name| name.to_str())
            .and_then(|file_str| {
                let template_strs: Vec<&str> =
                    templates.iter().map(|template| template.pattern.as_str()).collect();

                detect_template(file_str, &template_strs, validate_chapter_template)
            })
            .and_then(|pattern| {
                templates.iter().copied().find(|template| template.pattern == pattern)
            })
    }
}

#[rustfmt::skip]
fn modified_secs(metadata: &std::fs::Metadata) -> i64 {
    metadata.modified().map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;
    use tokio::fs;

    use super::ComicScannerService;
    use crate::{
        data::{
            models::archive::comic_directory::ComicDirectory,
            repositories::archive::{
                chapter_archive_repo::ChapterRepository, comic_directory_repo::ComicRepository,
                volume_archive_repo::VolumeRepository,
            },
        },
        tests::utils::setup_test_db::{reset_comics_last_modified, setup_test_db},
    };

    async fn setup(root: &TempDir) -> (ComicScannerService, sqlx::SqlitePool) {
        let pool = setup_test_db().await;
        let service = ComicScannerService::new(root.path().to_path_buf(), pool.clone());
        (service, pool)
    }

    async fn create_comic_dir(root: &TempDir, name: &str, chapters: &[&str]) -> PathBuf {
        let dir = root.path().join(name);
        fs::create_dir_all(&dir).await.unwrap();
        for chapter in chapters {
            fs::write(dir.join(chapter), b"fake cbz").await.unwrap();
        }
        dir
    }

    async fn create_volume_dir(
        root: &TempDir, comic: &str, volume: &str, chapters: &[&str],
    ) -> PathBuf {
        let dir = root.path().join(comic).join(volume);
        fs::create_dir_all(&dir).await.unwrap();
        for chapter in chapters {
            fs::write(dir.join(chapter), b"fake cbz").await.unwrap();
        }
        dir
    }

    async fn count_comics(pool: &sqlx::SqlitePool) -> i64 {
        ComicRepository::new(pool.clone()).base.count().await.unwrap()
    }

    async fn count_chapters(pool: &sqlx::SqlitePool) -> i64 {
        ChapterRepository::new(pool.clone()).base.count().await.unwrap()
    }

    async fn count_volumes(pool: &sqlx::SqlitePool) -> i64 {
        VolumeRepository::new(pool.clone()).base.count().await.unwrap()
    }

    #[tokio::test]
    async fn refresh_library_indexes_all_comics() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz", "Ch. 2.cbz"]).await;
        create_comic_dir(&root, "Vinland Saga", &["Ch. 1.cbz"]).await;

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_comics(&pool).await, 2);
    }

    #[tokio::test]
    async fn refresh_library_indexes_chapters_of_each_comic() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz", "Ch. 2.cbz", "Ch. 3.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_chapters(&pool).await, 3);
    }

    #[tokio::test]
    async fn refresh_library_ignores_folder_without_cbz() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;
        let empty = root.path().join("NoFiles");
        fs::create_dir_all(&empty).await.unwrap();
        fs::write(empty.join("cover.jpg"), b"img").await.unwrap();
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_comics(&pool).await, 0);
    }

    #[tokio::test]
    async fn refresh_library_does_not_duplicate_when_run_twice() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;
        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz", "Ch. 2.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_comics(&pool).await, 1);
        assert_eq!(count_chapters(&pool).await, 2);
    }

    #[tokio::test]
    async fn refresh_library_indexes_volumes() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;
        create_volume_dir(&root, "Berserk", "Vol. 01", &["Ch. 1.cbz", "Ch. 2.cbz"]).await;
        create_volume_dir(&root, "Berserk", "Vol. 02", &["Ch. 3.cbz"]).await;

        // Popula os templates de volume
        let pool_ref = &pool;
        sqlx::query(include_str!(
            "../../../infra/db/migrations/seeds/001_seed_chapter_template.sql"
        ))
        .execute(pool_ref)
        .await
        .unwrap();

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        assert_eq!(count_comics(&pool).await, 1);
        assert_eq!(count_volumes(&pool).await, 2);
        assert_eq!(count_chapters(&pool).await, 3);
    }

    #[tokio::test]
    async fn refresh_library_root_chapters_without_volume_id() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        let chapters = ChapterRepository::new(pool.clone()).base.find_all().await.unwrap();
        assert!(chapters[0].volume_fk.is_none());
    }

    #[tokio::test]
    async fn incremental_scan_does_not_process_unchanged_comic() {
        let root = tempfile::tempdir().unwrap();
        let (service, _) = setup(&root).await;
        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        let mut progress_count = 0usize;
        service
            .incremental_scan(
                root.path().to_path_buf(),
                |_| {
                    progress_count += 1;
                },
                |_| {},
            )
            .await
            .unwrap();
        assert_eq!(progress_count, 0, "No folders should be reprocessed");
    }

    #[tokio::test]
    async fn incremental_scan_processes_new_folder() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"]).await;

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        create_comic_dir(&root, "Vinland Saga", &["Ch. 1.cbz"]).await;

        service.incremental_scan(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_comics(&pool).await, 2);
    }

    #[tokio::test]
    async fn incremental_scan_removes_deleted_folder() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"]).await;
        create_comic_dir(&root, "Vinland Saga", &["Ch. 1.cbz"]).await;

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        fs::remove_dir_all(root.path().join("Vinland Saga")).await.unwrap();

        service.incremental_scan(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        let comics = ComicRepository::new(pool.clone()).base.find_all().await.unwrap();
        assert_eq!(comics.len(), 1);
        assert_eq!(comics[0].name, "Berserk");
    }

    #[tokio::test]
    async fn incremental_scan_updates_cover_in_modified_folder() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;
        let dir = create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"]).await;

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        let before = ComicRepository::new(pool.clone()).base.find_all().await.unwrap();

        assert!(before[0].cover.is_none());
        reset_comics_last_modified(&pool).await;
        fs::write(dir.join("cover.jpg"), b"fake cover").await.unwrap();

        service.incremental_scan(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        let after = ComicRepository::new(pool.clone()).base.find_all().await.unwrap();
        assert!(after[0].cover.is_some(), "cover should have been updated by incremental scan");
    }

    #[tokio::test]
    async fn rebuild_library_does_not_duplicate_chapters() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz", "Ch. 2.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        let before = count_chapters(&pool).await;
        service.rebuild_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_chapters(&pool).await, before);
    }

    #[tokio::test]
    #[ignore = "requires pdfium DLL which is not available in CI"]
    async fn refresh_library_pre_converts_pdfs() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        let fixture_pdf = std::path::Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("tests/wdio/comic/pdf/witchcraft.pdf");

        // Configura o diretório com PDFs
        let comic_dir = root.path().join("PdfComic");
        fs::create_dir_all(&comic_dir).await.unwrap();
        fs::copy(&fixture_pdf, comic_dir.join("root.pdf")).await.unwrap();

        let vol_dir = comic_dir.join("Vol. 01");
        fs::create_dir_all(&vol_dir).await.unwrap();
        fs::copy(&fixture_pdf, vol_dir.join("vol.pdf")).await.unwrap();

        // Popula os templates para que o volume seja detectado
        sqlx::query(include_str!(
            "../../../infra/db/migrations/seeds/001_seed_chapter_template.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        // Verifica se os CBZs foram gerados
        assert!(comic_dir.join("root.cbz").exists(), "root.cbz should have been generated");
        assert!(vol_dir.join("vol.cbz").exists(), "vol.cbz should have been generated");

        // Verifica a indexação
        assert_eq!(count_comics(&pool).await, 1, "Comic should be indexed");
        assert_eq!(count_chapters(&pool).await, 2, "Both CBZs should be indexed as chapters");
        assert_eq!(count_volumes(&pool).await, 1, "Volume should be indexed");
    }

    /// Cria um CBZ válido (zip real) com uma única página de imagem — usado pelos testes
    /// de extração de capa, já que `create_comic_dir` grava bytes falsos que não abrem como zip.
    fn create_valid_cbz(path: &std::path::Path, page_name: &str, page_bytes: &[u8]) {
        use std::io::Write;

        let file = std::fs::File::create(path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        archive.start_file(page_name, options).unwrap();
        archive.write_all(page_bytes).unwrap();
        archive.finish().unwrap();
    }

    async fn find_comic_by_name(pool: &sqlx::SqlitePool, name: &str) -> ComicDirectory {
        ComicRepository::new(pool.clone()).find_by_name(name).await.unwrap().unwrap()
    }

    #[tokio::test]
    async fn rescan_comic_does_not_affect_other_comics_in_library() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        let berserk_dir = create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"]).await;
        create_comic_dir(&root, "Vinland Saga", &["Ch. 1.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_comics(&pool).await, 2);

        fs::write(berserk_dir.join("Ch. 2.cbz"), b"fake cbz").await.unwrap();
        let berserk = find_comic_by_name(&pool, "Berserk").await;

        service.rescan_comic(berserk, |_| {}, |_| {}).await.unwrap();

        assert_eq!(count_comics(&pool).await, 2, "Other comics must remain untouched");
        assert_eq!(count_chapters(&pool).await, 3, "New chapter file must be picked up");
    }

    #[tokio::test]
    async fn deep_rescan_comic_removes_chapter_deleted_from_disk() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        let berserk_dir = create_comic_dir(&root, "Berserk", &["Ch. 1.cbz", "Ch. 2.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();
        assert_eq!(count_chapters(&pool).await, 2);

        fs::remove_file(berserk_dir.join("Ch. 2.cbz")).await.unwrap();
        let berserk = find_comic_by_name(&pool, "Berserk").await;

        service.deep_rescan_comic(berserk, |_| {}, |_| {}).await.unwrap();

        assert_eq!(
            count_chapters(&pool).await,
            1,
            "Orphaned chapter row must be removed by the deep rescan"
        );
    }

    #[tokio::test]
    async fn deep_rescan_comic_does_not_affect_other_comics_in_library() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"]).await;
        create_comic_dir(&root, "Vinland Saga", &["Ch. 1.cbz"]).await;
        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        let berserk = find_comic_by_name(&pool, "Berserk").await;
        service.deep_rescan_comic(berserk, |_| {}, |_| {}).await.unwrap();

        assert_eq!(count_comics(&pool).await, 2, "Other comics must remain untouched");
    }

    #[tokio::test]
    async fn process_entry_does_not_auto_generate_cover_when_folder_has_no_cover() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        let comic_dir = root.path().join("Berserk");
        fs::create_dir_all(&comic_dir).await.unwrap();
        create_valid_cbz(&comic_dir.join("Ch. 1.cbz"), "001.jpg", b"fake page bytes");

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        let comic = find_comic_by_name(&pool, "Berserk").await;
        assert!(
            comic.cover.is_none(),
            "scan should no longer auto-generate a cover from the first chapter page"
        );
    }

    #[tokio::test]
    async fn process_entry_does_not_overwrite_existing_cover() {
        let root = tempfile::tempdir().unwrap();
        let (service, pool) = setup(&root).await;

        let comic_dir = root.path().join("Berserk");
        fs::create_dir_all(&comic_dir).await.unwrap();
        fs::write(comic_dir.join("cover.jpg"), b"real cover").await.unwrap();
        create_valid_cbz(&comic_dir.join("Ch. 1.cbz"), "001.jpg", b"fake page bytes");

        service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await.unwrap();

        let comic = find_comic_by_name(&pool, "Berserk").await;
        let cover_path = comic.cover.expect("existing cover should still be set");
        let bytes = fs::read(&cover_path).await.unwrap();
        assert_eq!(bytes, b"real cover", "existing cover.jpg must not be overwritten");
    }
}
