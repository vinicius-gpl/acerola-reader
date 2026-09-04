use sqlx::SqlitePool;

use crate::{
    data::{
        models::history::{chapter_read::ChapterRead, reading_history::ReadingHistory},
        repositories::{
            archive::{
                chapter_archive_repo::ChapterRepository, comic_directory_repo::ComicRepository,
            },
            history::{
                chapter_read_repo::ChapterReadRepository,
                reading_history_repo::ReadingHistoryRepository,
            },
        },
    },
    infra::{
        error::ComicError,
        sync::messages::{HistoryEntry, HistoryManifest, ReadMarker},
    },
};

/// Monta e aplica manifestos de histórico de leitura entre dois devices.
///
/// A identidade cross-device é a chave natural `(comic.name, chapter.chapter_sort)` —
/// **não** o rótulo do capítulo (`chapter`). O rótulo é texto livre por device (cada um
/// nomeia o arquivo como quiser: "Cap 1", "Capítulo 01", etc.), então não serve como
/// chave entre dois devices independentes; `chapter_sort` é o valor normalizado que os
/// dois lados derivam do nome do arquivo, e é também o que o Android usa pra resolver o
/// `chapter_archive_id` local depois de aplicar um manifesto recebido — usar outra coisa
/// aqui faz o histórico sincronizar "com sucesso" mas gravar no capítulo errado (ou em
/// nenhum).
#[derive(Clone)]
pub struct HistorySyncService {
    comic_repo: ComicRepository,
    chapter_repo: ChapterRepository,
    reading_repo: ReadingHistoryRepository,
    read_marker_repo: ChapterReadRepository,
}

impl HistorySyncService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            comic_repo: ComicRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool.clone()),
            reading_repo: ReadingHistoryRepository::new(pool.clone()),
            read_marker_repo: ChapterReadRepository::new(pool),
        }
    }

    /// Monta o manifesto local completo (progresso de leitura + marcadores de lido).
    pub async fn build_manifest(&self) -> Result<HistoryManifest, ComicError> {
        let entries = self
            .reading_repo
            .find_all_payload_ordered()
            .await?
            .into_iter()
            .map(|payload| HistoryEntry {
                comic_name: payload.comic_name,
                chapter: payload.chapter_sort,
                last_page: payload.last_page,
                is_completed: payload.is_completed,
                updated_at: payload.updated_at,
            })
            .collect();

        let read_markers = self
            .read_marker_repo
            .find_all_with_natural_keys()
            .await?
            .into_iter()
            .map(|(comic_name, chapter, created_at)| ReadMarker { comic_name, chapter, created_at })
            .collect();

        Ok(HistoryManifest { entries, read_markers })
    }

    /// Aplica o manifesto recebido do peer localmente: last-write-wins por `updated_at`
    /// no progresso de leitura, união nos marcadores de "lido". Só toca em quadrinhos que
    /// já existem localmente — histórico não cria quadrinho novo, isso é papel do sync de
    /// arquivos. Retorna quantas entradas de progresso foram de fato aplicadas.
    pub async fn apply_manifest(&self, manifest: &HistoryManifest) -> Result<usize, ComicError> {
        let mut applied = 0usize;

        for entry in &manifest.entries {
            let Some(comic) = self.comic_repo.find_by_name(&entry.comic_name).await? else {
                continue;
            };
            let Some(chapter) =
                self.chapter_repo.find_by_comic_and_chapter_sort(comic.id, &entry.chapter).await?
            else {
                continue;
            };

            let local = self.reading_repo.find_by_comic_id(comic.id).await?;
            let should_apply = match &local {
                Some(existing) => entry.updated_at > existing.updated_at,
                None => true,
            };

            if !should_apply {
                continue;
            }

            self.reading_repo
                .upsert(&ReadingHistory {
                    comic_directory_id: comic.id,
                    chapter_archive_id: chapter.id,
                    last_page: entry.last_page,
                    is_completed: entry.is_completed,
                    updated_at: entry.updated_at,
                })
                .await?;

            applied += 1;
        }

        for marker in &manifest.read_markers {
            let Some(comic) = self.comic_repo.find_by_name(&marker.comic_name).await? else {
                continue;
            };
            let Some(chapter) =
                self.chapter_repo.find_by_comic_and_chapter_sort(comic.id, &marker.chapter).await?
            else {
                continue;
            };

            self.read_marker_repo
                .insert_or_ignore(&ChapterRead {
                    comic_directory_id: comic.id,
                    chapter_archive_id: chapter.id,
                    created_at: marker.created_at,
                })
                .await?;
        }

        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::HistorySyncService;
    use crate::{
        data::{
            models::history::reading_history::ReadingHistory,
            repositories::history::reading_history_repo::ReadingHistoryRepository,
        },
        infra::sync::messages::{HistoryEntry, HistoryManifest, ReadMarker},
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    async fn setup() -> (sqlx::SqlitePool, HistorySyncService) {
        let pool = setup_test_db_with_comic().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', 'p', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        (pool.clone(), HistorySyncService::new(pool))
    }

    #[tokio::test]
    async fn build_manifest_reflects_local_history() {
        let (pool, service) = setup().await;
        ReadingHistoryRepository::new(pool)
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 5,
                is_completed: false,
                updated_at: 1000,
            })
            .await
            .unwrap();

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].comic_name, "Test");
        // Carrega chapter_sort ("1"), não o rótulo ("Cap 1") — é a chave cross-device.
        assert_eq!(manifest.entries[0].chapter, "1");
    }

    #[tokio::test]
    async fn apply_manifest_ignores_older_entry() {
        let (pool, service) = setup().await;
        ReadingHistoryRepository::new(pool)
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 20,
                is_completed: false,
                updated_at: 5000,
            })
            .await
            .unwrap();

        let peer_manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Test".into(),
                chapter: "1".into(),
                last_page: 1,
                is_completed: false,
                updated_at: 100,
            }],
            read_markers: vec![],
        };

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied, 0);

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.entries[0].last_page, 20);
    }

    #[tokio::test]
    async fn apply_manifest_applies_newer_entry() {
        let (pool, service) = setup().await;
        ReadingHistoryRepository::new(pool)
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 1,
                is_completed: false,
                updated_at: 100,
            })
            .await
            .unwrap();

        let peer_manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Test".into(),
                chapter: "1".into(),
                last_page: 30,
                is_completed: true,
                updated_at: 9000,
            }],
            read_markers: vec![],
        };

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied, 1);

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.entries[0].last_page, 30);
        assert!(manifest.entries[0].is_completed);
    }

    #[tokio::test]
    async fn apply_manifest_ignores_comic_not_present_locally() {
        let (_, service) = setup().await;

        let peer_manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Nao Existe Aqui".into(),
                chapter: "1".into(),
                last_page: 1,
                is_completed: false,
                updated_at: 100,
            }],
            read_markers: vec![],
        };

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied, 0);
    }

    #[tokio::test]
    async fn apply_manifest_merges_read_markers() {
        let (pool, service) = setup().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let peer_manifest = HistoryManifest {
            entries: vec![],
            read_markers: vec![
                ReadMarker { comic_name: "Test".into(), chapter: "1".into(), created_at: 1 },
                ReadMarker { comic_name: "Test".into(), chapter: "2".into(), created_at: 2 },
            ],
        };

        service.apply_manifest(&peer_manifest).await.unwrap();

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.read_markers.len(), 2);
    }
}
