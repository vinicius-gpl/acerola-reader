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

    /// Monta a entrada de progresso de UM único quadrinho, sem marcadores de "lido" — usado
    /// pelo push individual (`acerola/sync-history-entry/1`), que existe pra não precisar do
    /// manifesto da biblioteca inteira só pra levar o progresso de um quadrinho que acabou de
    /// mudar. `None` quando o quadrinho não existe localmente ou nunca teve progresso salvo.
    pub async fn build_entry_for_comic(
        &self, comic_name: &str,
    ) -> Result<Option<HistoryEntry>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(None);
        };
        let Some(progress) = self.reading_repo.find_by_comic_id(comic.id).await? else {
            return Ok(None);
        };
        let Some(chapter) = self.chapter_repo.find_by_id(progress.chapter_archive_id).await? else {
            return Ok(None);
        };

        Ok(Some(HistoryEntry {
            comic_name: comic_name.to_string(),
            chapter: chapter.chapter_sort,
            last_page: progress.last_page,
            is_completed: progress.is_completed,
            updated_at: progress.updated_at,
        }))
    }

    /// Monta o manifesto restrito a um subconjunto de capítulos de UM quadrinho — usado pelo
    /// envio explícito de capítulo(s) selecionado(s) pra um peer (`acerola/sync-history-entry/1`).
    /// Recebe `chapter_archive_id`s (mesma unidade que o resto da UI local, ex.:
    /// `markChaptersReadBatch`) e resolve pra `chapter_sort` via [`Self::resolve_chapter_sorts`] —
    /// só esse último é comparável entre os dois devices, então é o que efetivamente viaja no
    /// manifesto. IDs que não existem ou pertencem a outro quadrinho são ignorados
    /// silenciosamente.
    pub async fn build_manifest_for_chapters(
        &self, comic_name: &str, chapter_ids: &[i64],
    ) -> Result<HistoryManifest, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(HistoryManifest { entries: vec![], read_markers: vec![] });
        };

        let chapter_sorts = self.resolve_chapter_sorts(comic_name, chapter_ids).await?;

        let entries = self
            .current_progress_entry_if_selected(comic_name, comic.id, &chapter_sorts)
            .await?
            .into_iter()
            .collect();

        let read_markers = self
            .read_marker_repo
            .find_natural_keys_for_chapters(comic.id, &chapter_sorts)
            .await?
            .into_iter()
            .map(|(chapter, created_at)| ReadMarker {
                comic_name: comic_name.to_string(),
                chapter,
                created_at,
            })
            .collect();

        Ok(HistoryManifest { entries, read_markers })
    }

    /// Entrada de progresso do capítulo "atual" (o de `reading_history`) de um quadrinho, só se
    /// esse capítulo estiver entre os `chapter_sorts` selecionados — `None` em qualquer guarda
    /// que falhe (sem progresso salvo, capítulo não encontrado, ou capítulo não selecionado).
    /// Extraído de [`Self::build_manifest_for_chapters`] pra manter guardas encadeadas em vez de
    /// `if let` aninhado.
    async fn current_progress_entry_if_selected(
        &self, comic_name: &str, comic_id: i64, chapter_sorts: &[String],
    ) -> Result<Option<HistoryEntry>, ComicError> {
        let Some(progress) = self.reading_repo.find_by_comic_id(comic_id).await? else {
            return Ok(None);
        };
        let Some(chapter) = self.chapter_repo.find_by_id(progress.chapter_archive_id).await? else {
            return Ok(None);
        };
        if !chapter_sorts.iter().any(|sort| sort == &chapter.chapter_sort) {
            return Ok(None);
        }

        Ok(Some(HistoryEntry {
            comic_name: comic_name.to_string(),
            chapter: chapter.chapter_sort,
            last_page: progress.last_page,
            is_completed: progress.is_completed,
            updated_at: progress.updated_at,
        }))
    }

    /// Resolve `chapter_archive_id`s locais pra `chapter_sort` (a chave comparável entre
    /// devices) — usado tanto por [`Self::build_manifest_for_chapters`] quanto pelo protocolo
    /// `acerola/sync-history-entry/1` pra montar o `HistoryEntryRequest` (primeira mensagem do
    /// contrato, declarando o escopo antes do manifesto em si). IDs que não existem ou
    /// pertencem a outro quadrinho são ignorados silenciosamente; quadrinho inexistente
    /// devolve lista vazia.
    pub async fn resolve_chapter_sorts(
        &self, comic_name: &str, chapter_ids: &[i64],
    ) -> Result<Vec<String>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(vec![]);
        };

        let mut chapter_sorts = Vec::with_capacity(chapter_ids.len());
        for &chapter_id in chapter_ids {
            let Some(chapter) = self.chapter_repo.find_by_id(chapter_id).await? else {
                continue;
            };
            if chapter.comic_directory_fk != comic.id {
                continue;
            }

            chapter_sorts.push(chapter.chapter_sort);
        }

        Ok(chapter_sorts)
    }

    /// Aplica o manifesto recebido do peer localmente: last-write-wins por `updated_at`
    /// no progresso de leitura, união nos marcadores de "lido". Só toca em quadrinhos que
    /// já existem localmente — histórico não cria quadrinho novo, isso é papel do sync de
    /// arquivos. Retorna quantas entradas/marcadores foram de fato aplicados — é o que dá ao
    /// chamador (`acerola/sync-history-entry/1`) um sinal real de validação, em vez de "a
    /// sessão não caiu" (ver `HistoryEntryAck`).
    pub async fn apply_manifest(
        &self, manifest: &HistoryManifest,
    ) -> Result<ApplyManifestStats, ComicError> {
        let mut entries_applied = 0u32;
        let mut markers_applied = 0u32;

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

            entries_applied += 1;
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

            markers_applied += 1;
        }

        Ok(ApplyManifestStats { entries_applied, markers_applied })
    }

    /// Verifica só a existência do quadrinho (sem tocar em progresso/lido) — usado pelo
    /// contrato de `acerola/sync-history-entry/1` pra validar ANTES de responder "aplicado com
    /// sucesso": sem isso, um manifesto vazio (nenhum capítulo selecionado tinha progresso/
    /// marcador) não dava nenhum sinal de erro mesmo quando o peer nunca teve esse quadrinho.
    pub async fn comic_exists(&self, comic_name: &str) -> Result<bool, ComicError> {
        Ok(self.comic_repo.find_by_name(comic_name).await?.is_some())
    }
}

/// Resultado de [`HistorySyncService::apply_manifest`] — quantas entradas de progresso e
/// quantos marcadores de "lido" foram de fato escritos localmente (não só "a sessão não deu
/// erro"). Espelhado no Android por `HistorySyncStats` (`protocol/history/model.rs`), que já
/// tinha esse formato mais rico — este tipo alinha o Desktop ao mesmo padrão.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApplyManifestStats {
    pub entries_applied: u32,
    pub markers_applied: u32,
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
        assert_eq!(applied.entries_applied, 0);

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
        assert_eq!(applied.entries_applied, 1);

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.entries[0].last_page, 30);
        assert!(manifest.entries[0].is_completed);
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_includes_progress_only_when_selected() {
        let (pool, service) = setup().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        ReadingHistoryRepository::new(pool.clone())
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 5,
                is_completed: false,
                updated_at: 1000,
            })
            .await
            .unwrap();

        let not_selected = service.build_manifest_for_chapters("Test", &[2]).await.unwrap();
        assert!(not_selected.entries.is_empty());

        let selected = service.build_manifest_for_chapters("Test", &[1]).await.unwrap();
        assert_eq!(selected.entries.len(), 1);
        assert_eq!(selected.entries[0].chapter, "1");
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_only_includes_selected_read_markers() {
        use crate::data::repositories::history::chapter_read_repo::ChapterReadRepository;

        let (pool, service) = setup().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let read_repo = ChapterReadRepository::new(pool.clone());
        read_repo.insert_batch(1, &[1, 2], 500).await.unwrap();

        let manifest = service.build_manifest_for_chapters("Test", &[1]).await.unwrap();

        assert_eq!(manifest.read_markers.len(), 1);
        assert_eq!(manifest.read_markers[0].chapter, "1");
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_ignores_id_belonging_to_another_comic() {
        use crate::{
            data::repositories::history::chapter_read_repo::ChapterReadRepository,
            tests::utils::setup_test_db::insert_comic_directory,
        };

        let (pool, service) = setup().await;
        insert_comic_directory(&pool, 2, "Outro", "/outro").await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, '1', 'p', '1', 0, 2, 0)")
            .execute(&pool)
            .await
            .unwrap();
        ChapterReadRepository::new(pool.clone()).insert_batch(2, &[2], 500).await.unwrap();

        let manifest = service.build_manifest_for_chapters("Test", &[2]).await.unwrap();

        assert!(manifest.read_markers.is_empty());
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_unknown_comic_returns_empty_manifest() {
        let (_, service) = setup().await;

        let manifest = service.build_manifest_for_chapters("Nao Existe", &[1]).await.unwrap();

        assert!(manifest.entries.is_empty());
        assert!(manifest.read_markers.is_empty());
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
        assert_eq!(applied.entries_applied, 0);
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

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied.markers_applied, 2);

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.read_markers.len(), 2);
    }

    #[tokio::test]
    async fn resolve_chapter_sorts_ignores_unknown_id_and_unknown_comic() {
        let (_, service) = setup().await;

        assert_eq!(
            service.resolve_chapter_sorts("Test", &[1, 999]).await.unwrap(),
            vec!["1".to_string()]
        );
        assert!(service.resolve_chapter_sorts("Nao Existe", &[1]).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn comic_exists_reflects_local_presence() {
        let (_, service) = setup().await;

        assert!(service.comic_exists("Test").await.unwrap());
        assert!(!service.comic_exists("Nao Existe Aqui").await.unwrap());
    }
}
