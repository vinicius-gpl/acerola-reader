use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::SqlitePool;

use crate::{
    core::services::archive::path_guard::path_hash,
    data::{
        models::archive::{
            chapter_archive::{is_special_name, ChapterArchive},
            comic_directory::ComicDirectory,
        },
        repositories::archive::{
            chapter_archive_repo::ChapterRepository, comic_directory_repo::ComicRepository,
        },
    },
    infra::{
        error::{ComicError, DbError},
        sync::messages::{ComicSummaryEntry, FileChapterInfo, FileComicInfo, FileManifest},
    },
};

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

/// Substitui caracteres inválidos em nomes de pasta no Windows/Unix por `_`.
fn sanitize_folder_name(name: &str) -> String {
    name.chars()
        .map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Monta e aplica manifestos de arquivos (os `.cbz`/`.cbr` reais) entre dois devices.
///
/// Diferente do sync de histórico, este PODE criar quadrinhos novos localmente — é o
/// mecanismo que efetivamente "leva a biblioteca" de um device pro outro. Quadrinho novo
/// (sem entrada local ainda) é gravado direto em `<library_root>/<nome sanitizado>/` —
/// mesmo lugar que o scanner normal da biblioteca criaria se o usuário tivesse adicionado
/// manualmente, sem pasta `synced/` intermediária.
#[derive(Clone)]
pub struct FileSyncService {
    comic_repo: ComicRepository,
    chapter_repo: ChapterRepository,
    /// Resolvido sob demanda (nunca guardado como `PathBuf` fixo) porque `FileSyncService`
    /// é construído uma única vez, numa task em background, no boot do app
    /// (`bios::network::setup_network`) — se guardássemos um `PathBuf` já resolvido aqui,
    /// qualquer troca de `library_path` feita pelo usuário depois do boot (em
    /// `settings.json`) seria ignorada até o próximo restart, e o sync continuaria
    /// gravando no destino antigo (foi exatamente o bug: arquivos indo pro fallback
    /// `<app_data>/library` mesmo com o path certo já configurado).
    library_root: Arc<dyn Fn() -> PathBuf + Send + Sync>,
}

impl FileSyncService {
    pub fn new(
        pool: SqlitePool, library_root: impl Fn() -> PathBuf + Send + Sync + 'static,
    ) -> Self {
        Self {
            comic_repo: ComicRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool),
            library_root: Arc::new(library_root),
        }
    }

    /// Monta o manifesto local completo, com tamanho de arquivo lido do disco sob demanda
    /// (só roda quando uma sessão de sync de arquivos começa, não a cada scan).
    pub async fn build_manifest(&self) -> Result<FileManifest, ComicError> {
        let rows = self.chapter_repo.find_all_with_comic_name().await?;
        let mut by_comic: HashMap<String, Vec<FileChapterInfo>> = HashMap::new();

        for row in rows {
            let size = tokio::fs::metadata(&row.path).await.map(|meta| meta.len()).unwrap_or(0);
            let file_name = Path::new(&row.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&row.chapter)
                .to_string();

            by_comic.entry(row.comic_name).or_default().push(FileChapterInfo {
                chapter: row.chapter,
                file_name,
                checksum: row.checksum,
                size,
            });
        }

        let comics = by_comic
            .into_iter()
            .map(|(comic_name, chapters)| FileComicInfo { comic_name, chapters })
            .collect();

        Ok(FileManifest { comics })
    }

    /// Resumo da biblioteca (título + contagem de capítulos + versão de capa) via
    /// `ChapterRepository::get_library_summary` — SQL puro, sem tocar disco. Usado só por
    /// `acerola/browse-library/1` (`LibraryBrowseInbound`); `build_manifest()` continua sendo
    /// o que a transferência de arquivos (`sync-files`/`sync-comic`) usa, já que essa fase
    /// precisa mesmo dos bytes/checksum de cada capítulo.
    pub async fn get_library_summary(&self) -> Result<Vec<ComicSummaryEntry>, ComicError> {
        let rows = self.chapter_repo.get_library_summary().await?;
        Ok(rows
            .into_iter()
            .map(|row| ComicSummaryEntry {
                comic_name: row.comic_name,
                chapter_count: row.chapter_count as u32,
                cover_version: row.cover_version,
            })
            .collect())
    }

    /// Mesmo manifesto de `build_manifest()`, mas filtrado pra um único quadrinho — usado pelo
    /// sync individual (`acerola/sync-comic/1`) pra não trocar a biblioteca inteira quando só
    /// um `comic_name` foi pedido. Reaproveita a query completa em vez de uma busca dedicada
    /// no repositório: bibliotecas pessoais são pequenas o bastante pra isso não importar, e
    /// evita duplicar a lógica de montagem de `FileChapterInfo` (tamanho em disco, nome do
    /// arquivo) em dois lugares.
    pub async fn build_manifest_for_comic(&self, comic_name: &str) -> Result<FileManifest, ComicError> {
        let mut manifest = self.build_manifest().await?;
        manifest.comics.retain(|comic| comic.comic_name == comic_name);
        Ok(manifest)
    }

    /// Calcula o que EU quero, comparando o manifesto do peer contra a base local: capítulos
    /// ausentes ou com checksum diferente (arquivo desatualizado/corrompido localmente).
    pub async fn diff_wanted(
        &self, peer_manifest: &FileManifest,
    ) -> Result<Vec<(String, String)>, ComicError> {
        let mut wanted = Vec::new();

        for comic in &peer_manifest.comics {
            let local_comic = self.comic_repo.find_by_name(&comic.comic_name).await?;

            for chapter in &comic.chapters {
                let already_have = match &local_comic {
                    Some(existing) => {
                        match self
                            .chapter_repo
                            .find_by_comic_and_chapter(existing.id, &chapter.chapter)
                            .await?
                        {
                            Some(local_chapter) => {
                                local_chapter.checksum.is_some()
                                    && local_chapter.checksum == chapter.checksum
                            },
                            None => false,
                        }
                    },
                    None => false,
                };

                if !already_have {
                    wanted.push((comic.comic_name.clone(), chapter.chapter.clone()));
                }
            }
        }

        Ok(wanted)
    }

    /// Resolve o path local + metadados de um capítulo já existente, pra enviar ao peer.
    pub async fn resolve_local_file(
        &self, comic_name: &str, chapter: &str,
    ) -> Result<Option<(PathBuf, u64, Option<String>, String)>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(None);
        };
        let Some(chapter_row) =
            self.chapter_repo.find_by_comic_and_chapter(comic.id, chapter).await?
        else {
            return Ok(None);
        };

        let path = PathBuf::from(&chapter_row.path);
        let size = tokio::fs::metadata(&path).await.map(|meta| meta.len()).unwrap_or(0);
        let file_name =
            path.file_name().and_then(|name| name.to_str()).unwrap_or(chapter).to_string();

        Ok(Some((path, size, chapter_row.checksum, file_name)))
    }

    /// Capa local (versão + bytes) de `comic_name`, usada por `acerola/browse-cover/1`. `None`
    /// cobre tanto "quadrinho não existe" quanto "existe mas não tem capa salva" — os dois casos
    /// resultam na mesma resposta pro peer, não há necessidade de distinguir.
    pub async fn get_local_cover(&self, comic_name: &str) -> Result<Option<(i64, Vec<u8>)>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(None);
        };
        let Some(cover_path) = comic.cover else {
            return Ok(None);
        };

        let bytes = match tokio::fs::read(&cover_path).await {
            Ok(bytes) => bytes,
            Err(_) => return Ok(None),
        };

        Ok(Some((comic.last_modified, bytes)))
    }

    /// Resolve o `ComicDirectory` de um quadrinho por nome, criando-o (via
    /// `create_synced_comic`) se ainda não existir localmente, e garante que a pasta exista
    /// em disco. Público pra quem recebe bytes (`transfer::receive_files`) poder gravar o
    /// arquivo temporário JÁ dentro dessa pasta antes de saber que o download terminou —
    /// evita depender de uma pasta de staging à parte (`rename` teria que ser garantido no
    /// mesmo volume de qualquer forma) e não deixa nenhuma pasta extra visível na biblioteca.
    pub async fn resolve_comic_dir(&self, comic_name: &str) -> Result<ComicDirectory, ComicError> {
        let comic = match self.comic_repo.find_by_name(comic_name).await? {
            Some(existing) => existing,
            None => self.create_synced_comic(comic_name).await?,
        };

        let comic_dir = PathBuf::from(&comic.path);
        tokio::fs::create_dir_all(&comic_dir).await.map_err(ComicError::Io)?;

        Ok(comic)
    }

    /// Move o arquivo já recebido (e verificado) de `temp_path` pro destino final dentro da
    /// biblioteca, criando o quadrinho no banco se ainda não existir, e indexa o capítulo
    /// reaproveitando o mesmo padrão de insert/update do scanner.
    ///
    /// **Contrato pro rótulo do capítulo** (o mesmo que qualquer peer, incluindo o Android,
    /// precisa seguir pro `chapter` recebido bater com o que esse desktop já produz no scan
    /// local via `Path::file_stem` — ver `chapter_scanner_engine.rs::scan_chapter`):
    /// o rótulo exibido é o nome do arquivo **sem a última extensão** — tudo antes do
    /// último `.`, exceto quando esse `.` é o primeiro caractere do nome (dotfile sem
    /// extensão de verdade, ex: `.gitignore`, que fica igual). `"Ch. 1.cbz"` → `"Ch. 1"`;
    /// `"archive.tar.gz"` → `"archive.tar"` (só a ÚLTIMA extensão sai).
    ///
    /// Por isso ignoramos o `chapter` que veio no `FileHeader` pra montar o rótulo — ele é
    /// só a chave natural que o peer usou pra decidir QUAL capítulo mandar (via
    /// `resolve_local_file`/`find_by_comic_and_chapter`), não necessariamente já vem nesse
    /// formato. Um peer com esse contrato quebrado (era o caso: o Android manda o
    /// `DocumentsContract` `DISPLAY_NAME` cru, com extensão, sem passar por um `file_stem`
    /// equivalente) não devia contaminar a biblioteca local — recalculamos aqui a partir do
    /// `file_name` real do arquivo, que é sempre confiável (é literalmente o nome do arquivo
    /// que acabamos de gravar em disco).
    pub async fn persist_received_chapter(
        &self, comic_name: &str, chapter: &str, file_name: &str, temp_path: &Path,
        checksum: String,
    ) -> Result<(), ComicError> {
        let comic = self.resolve_comic_dir(comic_name).await?;
        let comic_dir = PathBuf::from(&comic.path);
        let dest_path = comic_dir.join(file_name);
        tokio::fs::rename(temp_path, &dest_path).await.map_err(ComicError::Io)?;

        let display_chapter = Path::new(file_name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(chapter);

        // `(comic_directory_fk, chapter)` já existente cobre um reenvio (sync interrompido e
        // retomado, ou o mesmo capítulo pedido duas vezes) — nesse caso precisa ser um UPDATE
        // na linha existente, não um INSERT novo: o `id` dela é o `path_hash` do path ANTIGO,
        // que só bate com o `path_hash(&dest_path)` calculado abaixo se o nome do arquivo não
        // mudou entre as duas tentativas. Sem isso, o INSERT colide na constraint UNIQUE, e
        // como o arquivo já foi renomeado em disco (linha acima), a linha do banco fica
        // desatualizada silenciosamente (path/checksum antigos) mesmo com o conteúdo novo já
        // gravado — era exatamente esse o bug: `Err(DbError::UniqueViolation) => Ok(())`
        // descartava o conflito em vez de propagar o dado novo pra linha existente.
        let existing =
            self.chapter_repo.find_by_comic_and_chapter(comic.id, display_chapter).await?;

        // Caminho antigo só é removido depois que o registro novo está de pé — nunca antes,
        // pra um erro no meio do caminho não deixar nem arquivo nem linha de banco.
        if let Some(existing) = &existing {
            if existing.path != dest_path.to_string_lossy() {
                tokio::fs::remove_file(&existing.path).await.ok();
            }
        }

        let chapter_row = ChapterArchive {
            id: existing.as_ref().map(|c| c.id).unwrap_or_else(|| path_hash(&dest_path)),
            chapter: display_chapter.to_string(),
            path: dest_path.to_string_lossy().to_string(),
            chapter_sort: ChapterArchive::fallback_sort(display_chapter, 0),
            is_special: is_special_name(display_chapter),
            checksum: Some(checksum),
            comic_directory_fk: comic.id,
            volume_fk: existing.as_ref().and_then(|c| c.volume_fk),
            last_modified: now_secs(),
        };

        if existing.is_some() {
            self.chapter_repo.base.update(&chapter_row).await.map(|_| ()).map_err(ComicError::from)
        } else {
            match self.chapter_repo.base.insert(&chapter_row).await {
                // Corrida genuína (duas inserções concorrentes pro mesmo par no mesmo
                // instante) ainda pode colidir aqui mesmo depois do `find_by_comic_and_chapter`
                // não achar nada — janela entre o SELECT e o INSERT sem transação. Continua
                // sendo um no-op aceitável nesse caso raríssimo: a outra sessão já gravou dado
                // equivalente (mesmo capítulo, mesmo transfer).
                Ok(_) | Err(DbError::UniqueViolation) => Ok(()),
                Err(error) => Err(ComicError::from(error)),
            }
        }
    }

    /// Cria um novo `comic_directory` sob `<library_root>/<nome>` — usado quando um
    /// capítulo recebido pertence a um quadrinho que ainda não existe localmente. Mesmo
    /// path que o scanner normal da biblioteca usaria pra um quadrinho adicionado
    /// manualmente — sem pasta `synced/` intermediária.
    async fn create_synced_comic(&self, comic_name: &str) -> Result<ComicDirectory, ComicError> {
        let comic_dir = self.library_root().join(sanitize_folder_name(comic_name));

        let comic = ComicDirectory {
            id: path_hash(&comic_dir),
            name: comic_name.to_string(),
            path: comic_dir.to_string_lossy().to_string(),
            cover: None,
            banner: None,
            last_modified: now_secs(),
            archive_template_fk: None,
            external_sync_enabled: true,
            hidden: false,
        };

        match self.comic_repo.base.insert(&comic).await {
            Ok(saved) => Ok(saved),
            Err(DbError::UniqueViolation) => {
                self.comic_repo.find_by_name(comic_name).await?.ok_or(ComicError::NotFound)
            },
            Err(error) => Err(ComicError::from(error)),
        }
    }

    /// Resolvido a cada chamada — nunca cacheado — pra sempre refletir o `library_path`
    /// atual em `settings.json`, mesmo que o usuário tenha trocado depois do boot.
    pub fn library_root(&self) -> PathBuf {
        (self.library_root)()
    }
}

#[cfg(test)]
mod tests {
    use super::FileSyncService;
    use crate::infra::sync::messages::{FileChapterInfo, FileComicInfo, FileManifest};

    async fn setup() -> (sqlx::SqlitePool, tempfile::TempDir, FileSyncService) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db_with_comic().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', 'p/Cap 1.cbz', '1', 0, 'abc', 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool.clone(), move || root.clone());
        (pool, temp_dir, service)
    }

    #[tokio::test]
    async fn build_manifest_for_comic_scopes_to_a_single_comic() {
        let (_, _dir, service) = setup().await;

        let manifest = service.build_manifest_for_comic("Test").await.unwrap();
        assert_eq!(manifest.comics.len(), 1);
        assert_eq!(manifest.comics[0].comic_name, "Test");

        let empty = service.build_manifest_for_comic("Nao Existe").await.unwrap();
        assert!(empty.comics.is_empty());
    }

    #[tokio::test]
    async fn diff_wanted_ignores_chapter_with_same_checksum() {
        let (_, _dir, service) = setup().await;

        let peer_manifest = FileManifest {
            comics: vec![FileComicInfo {
                comic_name: "Test".into(),
                chapters: vec![FileChapterInfo {
                    chapter: "Cap 1".into(),
                    file_name: "Cap 1.cbz".into(),
                    checksum: Some("abc".into()),
                    size: 10,
                }],
            }],
        };

        let wanted = service.diff_wanted(&peer_manifest).await.unwrap();
        assert!(wanted.is_empty());
    }

    #[tokio::test]
    async fn diff_wanted_wants_chapter_with_different_checksum() {
        let (_, _dir, service) = setup().await;

        let peer_manifest = FileManifest {
            comics: vec![FileComicInfo {
                comic_name: "Test".into(),
                chapters: vec![FileChapterInfo {
                    chapter: "Cap 1".into(),
                    file_name: "Cap 1.cbz".into(),
                    checksum: Some("different".into()),
                    size: 10,
                }],
            }],
        };

        let wanted = service.diff_wanted(&peer_manifest).await.unwrap();
        assert_eq!(wanted, vec![("Test".to_string(), "Cap 1".to_string())]);
    }

    #[tokio::test]
    async fn diff_wanted_wants_comic_not_present_locally() {
        let (_, _dir, service) = setup().await;

        let peer_manifest = FileManifest {
            comics: vec![FileComicInfo {
                comic_name: "Novo Quadrinho".into(),
                chapters: vec![FileChapterInfo {
                    chapter: "Cap 1".into(),
                    file_name: "Cap 1.cbz".into(),
                    checksum: Some("x".into()),
                    size: 10,
                }],
            }],
        };

        let wanted = service.diff_wanted(&peer_manifest).await.unwrap();
        assert_eq!(wanted, vec![("Novo Quadrinho".to_string(), "Cap 1".to_string())]);
    }

    #[tokio::test]
    async fn persist_received_chapter_creates_new_comic_and_indexes_chapter() {
        let (pool, dir, service) = setup().await;

        let temp_path = dir.path().join("incoming.tmp");
        tokio::fs::write(&temp_path, b"fake cbz bytes").await.unwrap();

        service
            .persist_received_chapter(
                "Quadrinho Recebido",
                "Cap 1",
                "Cap 1.cbz",
                &temp_path,
                "checksum123".to_string(),
            )
            .await
            .unwrap();

        let comic: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comic_directory WHERE name = 'Quadrinho Recebido'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(comic.0, 1);

        let chapter: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chapter_archive WHERE chapter = 'Cap 1' AND checksum = 'checksum123'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(chapter.0, 1);

        assert!(!temp_path.exists(), "temp file should have been moved, not copied");
    }

    /// Trava o contrato do rótulo de capítulo: mesmo que o peer mande `chapter` sujo (com
    /// extensão, exatamente o bug reproduzido com o Android — `DocumentsContract`
    /// `DISPLAY_NAME` cru), o desktop nunca grava a extensão. O rótulo persistido é sempre
    /// recalculado a partir do `file_name` real do arquivo, via `file_stem`.
    #[tokio::test]
    async fn persist_received_chapter_ignores_extension_in_dirty_chapter_from_peer() {
        let (pool, dir, service) = setup().await;

        let temp_path = dir.path().join("incoming.tmp");
        tokio::fs::write(&temp_path, b"fake cbz bytes").await.unwrap();

        service
            .persist_received_chapter(
                "Mato Seihei no Slave",
                "Ch. 1.cbz", // <- rótulo sujo, exatamente como veio no log real do Android
                "Ch. 1.cbz",
                &temp_path,
                "checksum456".to_string(),
            )
            .await
            .unwrap();

        let chapter: (String, String) = sqlx::query_as(
            "SELECT chapter, chapter_sort FROM chapter_archive WHERE comic_directory_fk = (SELECT id FROM comic_directory WHERE name = 'Mato Seihei no Slave')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(chapter.0, "Ch. 1", "chapter não pode carregar a extensão do arquivo");
        assert_eq!(chapter.1, "1");
    }

    /// Regressão do bug reportado em 22/08/2026: sync de "Omoide Emanon" reenviando um
    /// capítulo já indexado (retry de uma sessão anterior interrompida) colidia no
    /// `UNIQUE(comic_directory_fk, chapter)` e o conflito era descartado silenciosamente
    /// (`Err(DbError::UniqueViolation) => Ok(())`), deixando o checksum/path antigos no banco
    /// mesmo com o arquivo novo já sobrescrito em disco. Recebimento repetido do mesmo
    /// `(comic_name, chapter)` precisa fazer UPDATE na linha existente, não descartar.
    #[tokio::test]
    async fn persist_received_chapter_updates_existing_row_on_resend_instead_of_dropping_it() {
        let (pool, dir, service) = setup().await;

        let first_temp = dir.path().join("first.tmp");
        tokio::fs::write(&first_temp, b"conteudo antigo").await.unwrap();
        service
            .persist_received_chapter(
                "Omoide Emanon",
                "Ch. 1",
                "Ch. 1.cbz",
                &first_temp,
                "checksum-antigo".to_string(),
            )
            .await
            .unwrap();

        let second_temp = dir.path().join("second.tmp");
        tokio::fs::write(&second_temp, b"conteudo novo, reenviado").await.unwrap();
        service
            .persist_received_chapter(
                "Omoide Emanon",
                "Ch. 1",
                "Ch. 1.cbz",
                &second_temp,
                "checksum-novo".to_string(),
            )
            .await
            .unwrap();

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT checksum FROM chapter_archive WHERE comic_directory_fk = (SELECT id FROM comic_directory WHERE name = 'Omoide Emanon') AND chapter = 'Ch. 1'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 1, "reenvio não pode criar uma segunda linha pro mesmo capítulo");
        assert_eq!(rows[0].0, "checksum-novo", "o reenvio precisa sobrescrever o checksum antigo");
    }
}
