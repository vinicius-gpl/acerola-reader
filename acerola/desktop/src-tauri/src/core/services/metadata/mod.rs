use std::{collections::HashMap, path::PathBuf};

use sqlx::SqlitePool;

use crate::{
    core::services::archive::files_guard::{
        ArchiveFileGuard, ArtworkFileGuard, FileGuard, MetadataFileGuard,
    },
    data::{
        models::metadata::{anilist_source::AnilistSource, author::AuthorMetadata, comic::ComicMetadata},
        repositories::{
            archive::comic_directory_repo::ComicRepository, metadata::MetadataRepository,
        },
    },
    infra::{
        api::{
            anilist::{AnilistClient, AnilistMedia},
            mangadex::{MangaData, MangadexClient, Relationship},
        },
        error::ComicError,
    },
};

pub mod comic_info;

fn find_xml_file_on_disk(directory: &std::path::Path) -> Option<String> {
    let metadata_guard = MetadataFileGuard;
    let entries = std::fs::read_dir(directory).ok()?;

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .find(|path| metadata_guard.is_allowed(path).is_ok())
        .and_then(|path| std::fs::read_to_string(path).ok())
}

fn find_xml_inside_archive(archive_path: &std::path::Path) -> Option<String> {
    let metadata_guard = MetadataFileGuard;
    let file = std::fs::File::open(archive_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    (0..archive.len()).find_map(|index| {
        let mut entry_file = archive.by_index(index).ok()?;
        let entry_path = std::path::Path::new(entry_file.name());
        if metadata_guard.is_allowed(entry_path).is_ok() {
            use std::io::Read;
            let mut content = String::new();
            entry_file.read_to_string(&mut content).ok().map(|_| content)
        } else {
            None
        }
    })
}

fn find_xml_in_directory_archives(directory: &std::path::Path) -> Option<String> {
    let archive_guard = ArchiveFileGuard;
    let entries = std::fs::read_dir(directory).ok()?;

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| archive_guard.is_allowed(path).is_ok())
        .find_map(|archive_path| find_xml_inside_archive(&archive_path))
}

pub struct MetadataService {
    repo: MetadataRepository,
    comic_directory_repo: ComicRepository,
    mangadex_client: MangadexClient,
    anilist_client: AnilistClient,
    http_client: reqwest::Client,
}

impl MetadataService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: MetadataRepository::new(pool.clone()),
            comic_directory_repo: ComicRepository::new(pool),
            mangadex_client: MangadexClient::new(),
            anilist_client: AnilistClient::new(),
            http_client: reqwest::Client::builder()
                .user_agent("AcerolaMangaApp/1.0 (Acerola Desktop)")
                .build()
                .unwrap(),
        }
    }

    pub async fn get_cover_and_banner(
        &self, comic_directory_fk: i64,
    ) -> (Option<String>, Option<String>) {
        if let Ok(Some(dir)) = self.comic_directory_repo.find_by_id(comic_directory_fk).await {
            (dir.cover, dir.banner)
        } else {
            (None, None)
        }
    }

    pub async fn sync_comic_mangadex(
        &self, title: &str, comic_directory_fk: i64, language: &str, generate_comic_info: bool,
    ) -> Result<ComicMetadata, ComicError> {
        let manga = self.fetch_manga_from_mangadex(title).await?;
        let metadata =
            self.build_metadata_from_mangadex(&manga, comic_directory_fk, language).await?;
        let saved = self.upsert_metadata(metadata, comic_directory_fk).await?;

        self.process_mangadex_author(&manga.relationships, saved.id).await?;
        self.process_mangadex_cover(&manga, comic_directory_fk, saved.id).await?;

        if generate_comic_info {
            self.generate_and_save_comic_info(comic_directory_fk, saved.id).await?;
        }

        Ok(saved)
    }

    pub async fn sync_comic_anilist(
        &self, title: &str, comic_directory_fk: i64, _language: &str, generate_comic_info: bool,
    ) -> Result<ComicMetadata, ComicError> {
        let media = self.fetch_media_from_anilist(title).await?;
        let metadata = self.build_metadata_from_anilist(&media, comic_directory_fk).await?;
        let saved = self.upsert_metadata(metadata, comic_directory_fk).await?;

        self.process_anilist_author(&media, saved.id).await?;
        self.process_anilist_images(&media, comic_directory_fk, saved.id).await?;
        self.process_anilist_score(&media, saved.id).await?;

        if generate_comic_info {
            self.generate_and_save_comic_info(comic_directory_fk, saved.id).await?;
        }

        Ok(saved)
    }

    pub async fn parse_and_sync_comic_info(
        &self, xml_content: &str, comic_directory_fk: i64,
    ) -> Result<ComicMetadata, ComicError> {
        let comic_info = quick_xml::de::from_str::<comic_info::ComicInfo>(xml_content)?;
        let metadata = self.build_metadata_from_comic_info(&comic_info, comic_directory_fk).await?;
        let saved = self.upsert_metadata(metadata, comic_directory_fk).await?;

        self.process_comic_info_author(&comic_info, saved.id).await?;

        Ok(saved)
    }

    pub async fn sync_comic_comic_info(
        &self, comic_directory_fk: i64,
    ) -> Result<ComicMetadata, ComicError> {
        let comic_dir = self
            .comic_directory_repo
            .find_by_id(comic_directory_fk)
            .await?
            .ok_or(ComicError::NotFound)?;

        let directory_path = PathBuf::from(&comic_dir.path);

        let xml_content = find_xml_file_on_disk(&directory_path)
            .or_else(|| find_xml_in_directory_archives(&directory_path))
            .ok_or(ComicError::ComicInfoNotFound)?;

        self.parse_and_sync_comic_info(&xml_content, comic_directory_fk).await
    }

    /// Remove por completo os metadados sincronizados e as artes (cover/banner +
    /// ComicInfo.xml) de um quadrinho, devolvendo-o ao estado "sem metadados" — como se
    /// nunca tivesse sido sincronizado. Pensada para reverter um sync que trouxe dados
    /// errados (ex.: match errado no AniList/MangaDex).
    ///
    /// A limpeza dos arquivos em disco é best-effort: um arquivo ausente ou bloqueado
    /// não interrompe a limpeza dos demais nem do banco.
    pub async fn clear_comic_metadata(&self, comic_directory_fk: i64) -> Result<(), ComicError> {
        let comic_dir = self
            .comic_directory_repo
            .find_by_id(comic_directory_fk)
            .await?
            .ok_or(ComicError::NotFound)?;

        let dir_path = PathBuf::from(&comic_dir.path);

        // Mesmo padrão de `find_xml_file_on_disk`: reaproveita os guards que já definem o
        // que conta como arquivo de metadados/artwork sincronizado, em vez de manter uma
        // lista solta de nomes de arquivo que precisaria ser atualizada nos dois lugares.
        let metadata_guard = MetadataFileGuard;
        let artwork_guard = ArtworkFileGuard;

        if let Ok(entries) = std::fs::read_dir(&dir_path) {
            for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
                let is_synced_file =
                    metadata_guard.is_allowed(&path).is_ok() || artwork_guard.is_allowed(&path).is_ok();

                if !is_synced_file {
                    continue;
                }

                if let Err(error) = std::fs::remove_file(&path) {
                    tracing::warn!(
                        file = %path.to_string_lossy(),
                        directory = %dir_path.to_string_lossy(),
                        error = %error,
                        "Failed to remove file while clearing comic metadata"
                    );
                }
            }
        }

        self.comic_directory_repo.clear_artwork(comic_directory_fk).await?;
        self.repo.delete_by_comic_id(comic_directory_fk).await?;

        Ok(())
    }

    /// Aplica [`Self::clear_comic_metadata`] em lote, prosseguindo mesmo se algum item
    /// falhar. Retorna quantos quadrinhos foram limpos com sucesso.
    pub async fn clear_comics_metadata_batch(&self, ids: &[i64]) -> usize {
        let mut cleared = 0;
        for &id in ids {
            match self.clear_comic_metadata(id).await {
                Ok(()) => cleared += 1,
                Err(error) => {
                    tracing::warn!(
                        comic_directory_fk = id,
                        error = %error,
                        "Failed to clear comic metadata"
                    );
                },
            }
        }
        cleared
    }

    pub async fn sync_all_comics_mangadex<F>(
        &self, language: &str, generate_comic_info: bool, on_progress: F,
    ) -> Result<(), ComicError>
    where
        F: Fn(String),
    {
        let comics = self.comic_directory_repo.base.find_all().await?;
        for comic in comics {
            if !comic.external_sync_enabled {
                continue;
            }
            on_progress(comic.name.clone());
            match self
                .sync_comic_mangadex(&comic.name, comic.id, language, generate_comic_info)
                .await
            {
                Ok(_) => tracing::info!("Successfully synced {} via MangaDex", comic.name),
                Err(err) => tracing::warn!("Failed to sync {} via MangaDex: {}", comic.name, err),
            }
        }
        Ok(())
    }

    pub async fn sync_all_comics_anilist<F>(
        &self, language: &str, generate_comic_info: bool, on_progress: F,
    ) -> Result<(), ComicError>
    where
        F: Fn(String),
    {
        let comics = self.comic_directory_repo.base.find_all().await?;
        for comic in comics {
            if !comic.external_sync_enabled {
                continue;
            }
            on_progress(comic.name.clone());
            match self
                .sync_comic_anilist(&comic.name, comic.id, language, generate_comic_info)
                .await
            {
                Ok(_) => tracing::info!("Successfully synced {} via AniList", comic.name),
                Err(err) => tracing::warn!("Failed to sync {} via AniList: {}", comic.name, err),
            }
        }
        Ok(())
    }
}

impl MetadataService {
    fn normalize_name(name: &str) -> String {
        name.chars().filter(|char| char.is_alphanumeric()).collect::<String>().to_lowercase()
    }

    async fn fetch_manga_from_mangadex(&self, title: &str) -> Result<MangaData, ComicError> {
        let response = self.mangadex_client.search_manga_by_title(title).await?;
        let normalized_input = Self::normalize_name(title);

        tracing::info!(
            "Searching for manga with title: '{}', normalized: '{}', found {} results",
            title,
            normalized_input,
            response.data.len()
        );

        // 1. Tentar encontrar match EXATO no título principal
        let primary_match = response.data.iter().find(|manga| {
            manga
                .attributes
                .title
                .values()
                .any(|text| Self::normalize_name(text) == normalized_input)
        });

        if let Some(manga) = primary_match {
            tracing::info!(
                "Found exact primary title match for '{}': {} - {}",
                title,
                manga.id,
                manga.attributes.title.values().next().unwrap_or(&String::new())
            );
            return Ok(manga.clone());
        }

        // 2. Se não achou no principal, tenta encontrar match EXATO nos títulos alternativos
        let alt_match = response.data.iter().find(|manga| {
            manga
                .attributes
                .alt_titles
                .iter()
                .flat_map(|alt| alt.values())
                .any(|text| Self::normalize_name(text) == normalized_input)
        });

        if let Some(manga) = alt_match {
            tracing::info!(
                "Found exact alt title match for '{}': {} - {}",
                title,
                manga.id,
                manga.attributes.title.values().next().unwrap_or(&String::new())
            );
            return Ok(manga.clone());
        }

        // 3. Fallback: Pega o primeiro resultado da busca se não houver match exato
        match response.data.into_iter().next() {
            Some(manga) => {
                tracing::warn!(
                    "No exact match for '{}', using first result: {} - {}",
                    title,
                    manga.id,
                    manga.attributes.title.values().next().unwrap_or(&String::new())
                );
                Ok(manga)
            },
            None => Err(ComicError::NotFound),
        }
    }

    async fn fetch_media_from_anilist(&self, title: &str) -> Result<AnilistMedia, ComicError> {
        let response = self.anilist_client.search_manga_by_title(title).await?;
        let normalized_input = Self::normalize_name(title);

        let exact_match = response.data.page.media.iter().find(|media| {
            let titles = vec![
                media.title.user_preferred.as_deref(),
                media.title.english.as_deref(),
                media.title.romaji.as_deref(),
            ];

            titles.into_iter().flatten().any(|text| Self::normalize_name(text) == normalized_input)
        });

        match exact_match {
            Some(media) => Ok(media.clone()),
            None => response.data.page.media.into_iter().next().ok_or(ComicError::NotFound),
        }
    }

    async fn build_metadata_from_mangadex(
        &self, manga: &MangaData, comic_directory_fk: i64, language: &str,
    ) -> Result<ComicMetadata, ComicError> {
        let lang_code = language.split('-').next().unwrap_or(language);

        Ok(ComicMetadata {
            id: self.repo.comic_repo.get_next_id().await?,
            title: Self::extract_localized_text(&manga.attributes.title, language, lang_code),
            description: Self::extract_localized_text(
                &manga.attributes.description,
                language,
                lang_code,
            ),
            status: manga.attributes.status.clone(),
            publication: manga.attributes.year,
            sync_source: Some("MangaDex".to_string()),
            has_comic_info: false,
            comic_directory_fk: Some(comic_directory_fk),
        })
    }

    async fn build_metadata_from_anilist(
        &self, media: &AnilistMedia, comic_directory_fk: i64,
    ) -> Result<ComicMetadata, ComicError> {
        Ok(ComicMetadata {
            id: self.repo.comic_repo.get_next_id().await?,
            title: media
                .title
                .user_preferred
                .clone()
                .or(media.title.english.clone())
                .or(media.title.romaji.clone())
                .unwrap_or_default(),
            description: media.description.clone().unwrap_or_default(),
            status: media.status.clone().unwrap_or_default(),
            publication: None,
            sync_source: Some("AniList".to_string()),
            has_comic_info: false,
            comic_directory_fk: Some(comic_directory_fk),
        })
    }

    async fn generate_and_save_comic_info(
        &self, comic_directory_fk: i64, metadata_id: i64,
    ) -> Result<(), ComicError> {
        let comic_dir = self
            .comic_directory_repo
            .find_by_id(comic_directory_fk)
            .await?
            .ok_or(ComicError::NotFound)?;
        let metadata = self
            .repo
            .get_comic_metadata_by_comic_id(comic_directory_fk)
            .await?
            .ok_or(ComicError::NotFound)?;

        let authors = self
            .repo
            .get_author_metadata_by_comic_metadata_id(metadata_id)
            .await
            .unwrap_or_default();
        let writer = authors.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ");

        let comic_info = comic_info::ComicInfo {
            title: Some(metadata.title),
            summary: Some(metadata.description),
            writer: if writer.is_empty() { None } else { Some(writer) },
            ..Default::default()
        };

        let xml_str = quick_xml::se::to_string(&comic_info)
            .map_err(|err| ComicError::SystemFailure(err.to_string()))?;
        let xml_str = xml_str.replace("<ComicInfo>", "<ComicInfo xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">");
        let xml_final = format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}", xml_str);

        let file_path = PathBuf::from(&comic_dir.path).join("ComicInfo.xml");
        std::fs::write(&file_path, xml_final)
            .map_err(|err| ComicError::SystemFailure(err.to_string()))?;

        Ok(())
    }

    async fn build_metadata_from_comic_info(
        &self, comic_info: &comic_info::ComicInfo, comic_directory_fk: i64,
    ) -> Result<ComicMetadata, ComicError> {
        Ok(ComicMetadata {
            id: self.repo.comic_repo.get_next_id().await?,
            title: comic_info.title.clone().unwrap_or_default(),
            description: comic_info.summary.clone().unwrap_or_default(),
            status: String::new(),
            publication: None,
            sync_source: Some("ComicInfo".to_string()),
            has_comic_info: true,
            comic_directory_fk: Some(comic_directory_fk),
        })
    }

    async fn upsert_metadata(
        &self, mut metadata: ComicMetadata, comic_directory_fk: i64,
    ) -> Result<ComicMetadata, ComicError> {
        let existing = self.repo.get_comic_metadata_by_comic_id(comic_directory_fk).await?;

        tracing::info!(
            "Upsert metadata for comic_directory_fk: {}, existing: {:?}",
            comic_directory_fk,
            existing.is_some()
        );

        match existing {
            Some(found) => {
                tracing::info!("Updating existing metadata with id: {}", found.id);
                metadata.id = found.id;
                Ok(self.repo.comic_repo.update(&metadata).await?)
            },
            None => {
                tracing::info!("Inserting new metadata");
                Ok(self.repo.comic_repo.insert(&metadata).await?)
            },
        }
    }

    fn extract_localized_text(
        map: &HashMap<String, String>, language: &str, fallback: &str,
    ) -> String {
        map.get(language)
            .or_else(|| map.get("en"))
            .or_else(|| map.get(fallback))
            .or_else(|| map.values().next())
            .cloned()
            .unwrap_or_default()
    }
}

impl MetadataService {
    async fn process_mangadex_author(
        &self, relationships: &[Relationship], metadata_id: i64,
    ) -> Result<(), ComicError> {
        let author_name = relationships
            .iter()
            .find(|rel| rel.kind == "author")
            .and_then(|rel| rel.attributes.as_ref())
            .and_then(|attr| attr.name.clone());

        if let Some(name) = author_name {
            self.upsert_author(&name, "author", metadata_id).await?;
        }
        Ok(())
    }

    async fn process_anilist_author(
        &self, media: &AnilistMedia, metadata_id: i64,
    ) -> Result<(), ComicError> {
        let author_edge = media.staff.as_ref().and_then(|staff| {
            staff.edges.iter().find(|edge| {
                let role = edge.role.to_lowercase();
                role.contains("story") || role.contains("art") || role.contains("author")
            })
        });

        if let Some(edge) = author_edge {
            self.upsert_author(&edge.node.name.full, &edge.role, metadata_id).await?;
        }
        Ok(())
    }

    /// Persiste a nota do AniList (escala 0-100) em `anilist_source`, ligada por
    /// `comic_metadata_fk`. Não faz nada se o AniList não retornou nota (manga sem votos).
    async fn process_anilist_score(
        &self, media: &AnilistMedia, metadata_id: i64,
    ) -> Result<(), ComicError> {
        let Some(average_score) = media.average_score else {
            return Ok(());
        };

        self.upsert_anilist_source(media.id as i64, average_score, metadata_id).await
    }

    async fn upsert_anilist_source(
        &self, anilist_id: i64, average_score: i64, metadata_id: i64,
    ) -> Result<(), ComicError> {
        let existing = self.repo.get_anilist_source_by_comic_metadata_id(metadata_id).await?;

        let source = AnilistSource {
            id: match &existing {
                Some(found) => found.id,
                None => self.repo.anilist_repo.get_next_id().await?,
            },
            anilist_id,
            average_score: Some(average_score),
            popularity: existing.as_ref().and_then(|found| found.popularity),
            trending: existing.as_ref().and_then(|found| found.trending),
            cover_image: existing.as_ref().and_then(|found| found.cover_image.clone()),
            banner_image: existing.as_ref().and_then(|found| found.banner_image.clone()),
            comic_metadata_fk: metadata_id,
        };

        match existing {
            Some(_) => self.repo.anilist_repo.update(&source).await?,
            None => self.repo.anilist_repo.insert(&source).await?,
        };

        Ok(())
    }

    async fn process_comic_info_author(
        &self, comic_info: &comic_info::ComicInfo, metadata_id: i64,
    ) -> Result<(), ComicError> {
        if let Some(writer) = comic_info.writer.as_ref().filter(|text| !text.trim().is_empty()) {
            self.upsert_author(writer, "writer", metadata_id).await?;
        }
        Ok(())
    }

    async fn upsert_author(
        &self, name: &str, author_type: &str, metadata_id: i64,
    ) -> Result<(), ComicError> {
        let existing = self
            .repo
            .get_author_metadata_by_comic_metadata_id(metadata_id)
            .await?
            .into_iter()
            .find(|author| author.name == name && author.r#type == author_type);

        match existing {
            Some(found) => {
                let author = AuthorMetadata {
                    id: found.id,
                    name: name.to_string(),
                    r#type: author_type.to_string(),
                    comic_metadata_fk: metadata_id,
                };
                self.repo.author_repo.update(&author).await?;
            },
            None => {
                let author = AuthorMetadata {
                    id: self.repo.author_repo.get_next_id().await?,
                    name: name.to_string(),
                    r#type: author_type.to_string(),
                    comic_metadata_fk: metadata_id,
                };
                self.repo.author_repo.insert(&author).await?;
            },
        }
        Ok(())
    }

    async fn save_author(
        &self, name: &str, author_type: &str, metadata_id: i64,
    ) -> Result<(), ComicError> {
        self.upsert_author(name, author_type, metadata_id).await
    }

    async fn process_mangadex_cover(
        &self, manga: &MangaData, comic_directory_fk: i64, _metadata_id: i64,
    ) -> Result<(), ComicError> {
        let comic_dir = self
            .comic_directory_repo
            .find_by_id(comic_directory_fk)
            .await?
            .ok_or(ComicError::NotFound)?;

        let file_name = manga
            .relationships
            .iter()
            .find(|rel| rel.kind == "cover_art")
            .and_then(|rel| rel.attributes.as_ref())
            .and_then(|attr| attr.file_name.clone())
            .ok_or(ComicError::NotFound)?;

        let cover_url = MangadexClient::get_cover_url(&manga.id, &file_name);
        self.download_and_save_image(&cover_url, &comic_dir.path, "cover.jpg").await?;

        let mut updated_dir = comic_dir.clone();
        updated_dir.cover =
            Some(PathBuf::from(&comic_dir.path).join("cover.jpg").to_string_lossy().to_string());
        self.comic_directory_repo.base.update(&updated_dir).await?;

        Ok(())
    }

    async fn process_anilist_images(
        &self, media: &AnilistMedia, comic_directory_fk: i64, _metadata_id: i64,
    ) -> Result<(), ComicError> {
        let comic_dir = self
            .comic_directory_repo
            .find_by_id(comic_directory_fk)
            .await?
            .ok_or(ComicError::NotFound)?;

        let cover_url = media.cover_image.as_ref().and_then(|cover| cover.large.clone());

        if let Some(url) = cover_url {
            self.download_and_save_image(&url, &comic_dir.path, "cover.jpg").await?;

            let mut updated_dir = comic_dir.clone();
            updated_dir.cover = Some(
                PathBuf::from(&comic_dir.path).join("cover.jpg").to_string_lossy().to_string(),
            );
            self.comic_directory_repo.base.update(&updated_dir).await?;
        }

        if let Some(ref banner_url) = media.banner_image {
            self.download_and_save_image(banner_url, &comic_dir.path, "banner.jpg").await?;

            let mut updated_dir = comic_dir.clone();
            updated_dir.banner = Some(
                PathBuf::from(&comic_dir.path).join("banner.jpg").to_string_lossy().to_string(),
            );
            self.comic_directory_repo.base.update(&updated_dir).await?;
        }

        Ok(())
    }

    async fn download_and_save_image(
        &self, url: &str, dir_path: &str, file_name: &str,
    ) -> Result<(), ComicError> {
        let file_path = PathBuf::from(dir_path).join(file_name);

        let response = self
            .http_client
            .get(url)
            .header("Accept", "image/*")
            .send()
            .await?
            .error_for_status()
            .map_err(|error| ComicError::SystemFailure(format!("HTTP error: {}", error)))?;

        let bytes = response.bytes().await?;

        tracing::info!("Downloaded {} bytes from {}", bytes.len(), url);

        tokio::fs::write(&file_path, &bytes).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    use super::*;
    use crate::tests::utils::setup_test_db::setup_test_db_with_comic;

    #[test]
    fn test_find_xml_file_on_disk_case_insensitive() {
        let temp_directory = tempdir().expect("Failed to create temp directory");
        let target_xml_path = temp_directory.path().join("comicinfo.xml");
        let xml_payload = "<ComicInfo><Title>Case Insensitive Test</Title></ComicInfo>";
        std::fs::write(&target_xml_path, xml_payload).expect("Failed to write test XML file");

        let extracted_content = find_xml_file_on_disk(temp_directory.path());
        assert_eq!(extracted_content, Some(xml_payload.to_string()));
    }

    #[test]
    fn test_find_xml_inside_cbz_archive() {
        let temp_directory = tempdir().expect("Failed to create temp directory");
        let cbz_file_path = temp_directory.path().join("chapter1.cbz");
        let archive_file =
            std::fs::File::create(&cbz_file_path).expect("Failed to create cbz file");
        let mut zip_writer = zip::ZipWriter::new(archive_file);

        let options = SimpleFileOptions::default();
        zip_writer
            .start_file("subfolder/ComicInfo.xml", options)
            .expect("Failed to start zip entry");
        let xml_payload = "<ComicInfo><Title>Zip Test Comic</Title></ComicInfo>";
        zip_writer.write_all(xml_payload.as_bytes()).expect("Failed to write zip content");
        zip_writer.finish().expect("Failed to finish zip file");

        let extracted_content = find_xml_in_directory_archives(temp_directory.path());
        assert_eq!(extracted_content, Some(xml_payload.to_string()));
    }

    #[tokio::test]
    async fn test_sync_comic_info_success() {
        let pool = setup_test_db_with_comic().await;
        let service = MetadataService::new(pool.clone());

        let temp_directory = tempdir().expect("Failed to create temp directory");
        let xml_file_path = temp_directory.path().join("ComicInfo.xml");
        let xml_payload = r#"<?xml version="1.0" encoding="utf-8"?>
<ComicInfo xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Title>Test ComicInfo Title</Title>
  <Summary>Test Summary from ComicInfo XML</Summary>
  <Writer>Test Writer</Writer>
</ComicInfo>"#;
        std::fs::write(&xml_file_path, xml_payload).expect("Failed to write XML file");

        let comic_repository = ComicRepository::new(pool.clone());
        let mut comic = comic_repository
            .find_by_id(1)
            .await
            .expect("Failed to fetch comic")
            .expect("Comic #1 should exist");
        comic.path = temp_directory.path().to_string_lossy().to_string();
        comic_repository.base.update(&comic).await.expect("Failed to update comic path");

        let metadata = service.sync_comic_comic_info(1).await.expect("Failed to sync comic_info");
        assert_eq!(metadata.title, "Test ComicInfo Title");
        assert_eq!(metadata.description, "Test Summary from ComicInfo XML");
        assert_eq!(metadata.sync_source, Some("ComicInfo".to_string()));
    }

    #[tokio::test]
    async fn test_sync_comic_info_when_file_missing_returns_error() {
        let pool = setup_test_db_with_comic().await;
        let service = MetadataService::new(pool.clone());

        let temp_directory = tempdir().expect("Failed to create temp directory");
        let comic_repository = ComicRepository::new(pool.clone());
        let mut comic = comic_repository
            .find_by_id(1)
            .await
            .expect("Failed to fetch comic")
            .expect("Comic #1 should exist");
        comic.path = temp_directory.path().to_string_lossy().to_string();
        comic_repository.base.update(&comic).await.expect("Failed to update comic path");

        let sync_result = service.sync_comic_comic_info(1).await;
        assert!(matches!(sync_result, Err(ComicError::ComicInfoNotFound)));
    }

    #[tokio::test]
    async fn test_upsert_anilist_source_inserts_score_on_first_sync() {
        let pool = setup_test_db_with_comic().await;
        let service = MetadataService::new(pool.clone());

        let metadata_id =
            service.repo.comic_repo.get_next_id().await.expect("Failed to get next id");
        service
            .repo
            .comic_repo
            .insert(&ComicMetadata {
                id: metadata_id,
                title: "Test".to_string(),
                description: String::new(),
                status: String::new(),
                publication: None,
                sync_source: Some("AniList".to_string()),
                has_comic_info: false,
                comic_directory_fk: Some(1),
            })
            .await
            .expect("Failed to insert metadata");

        service.upsert_anilist_source(42, 87, metadata_id).await.expect("Failed to upsert score");

        let source = service
            .repo
            .get_anilist_source_by_comic_metadata_id(metadata_id)
            .await
            .expect("Failed to fetch anilist source")
            .expect("anilist_source row should exist");
        assert_eq!(source.anilist_id, 42);
        assert_eq!(source.average_score, Some(87));
    }

    #[tokio::test]
    async fn test_upsert_anilist_source_updates_existing_score_without_duplicating() {
        let pool = setup_test_db_with_comic().await;
        let service = MetadataService::new(pool.clone());

        let metadata_id =
            service.repo.comic_repo.get_next_id().await.expect("Failed to get next id");
        service
            .repo
            .comic_repo
            .insert(&ComicMetadata {
                id: metadata_id,
                title: "Test".to_string(),
                description: String::new(),
                status: String::new(),
                publication: None,
                sync_source: Some("AniList".to_string()),
                has_comic_info: false,
                comic_directory_fk: Some(1),
            })
            .await
            .expect("Failed to insert metadata");

        service.upsert_anilist_source(42, 70, metadata_id).await.expect("Failed first upsert");
        service.upsert_anilist_source(42, 91, metadata_id).await.expect("Failed second upsert");

        let source = service
            .repo
            .get_anilist_source_by_comic_metadata_id(metadata_id)
            .await
            .expect("Failed to fetch anilist source")
            .expect("anilist_source row should exist");
        assert_eq!(source.average_score, Some(91), "score should be updated, not duplicated");
    }
}
