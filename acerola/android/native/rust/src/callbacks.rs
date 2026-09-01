//! Traits estrangeiras (implementadas em Kotlin) usadas pelos protocolos de sync para ler/gravar
//! dados no Room/SAF do app, mantendo a lógica de manifesto/diff/aplicação inteiramente em Rust.

use serde::{Deserialize, Serialize};

/// Fonte/destino dos dados de histórico de leitura (`reading_history` + `chapter_read`) usada
/// pelo protocolo `acerola/sync-history/1`. Implementada em Kotlin contra o Room.
#[uniffi::export(with_foreign)]
pub trait HistorySyncProvider: Send + Sync {
    /// Todas as linhas locais de progresso de leitura, com chave natural (nome do quadrinho).
    fn get_reading_progress(&self) -> Vec<FfiReadingProgressEntry>;

    /// Todas as linhas locais de capítulos marcados como lidos, com chave natural.
    fn get_chapters_read(&self) -> Vec<FfiChapterReadEntry>;

    /// Aplica uma linha de progresso recebida do peer (Rust já decidiu que ela vence via LWW).
    /// Retorna `false` se o quadrinho referenciado não existir localmente (entrada ignorada).
    fn apply_reading_progress(&self, entry: FfiReadingProgressEntry) -> bool;

    /// Aplica uma linha de "capítulo lido" recebida do peer (união idempotente).
    /// Retorna `false` se o quadrinho referenciado não existir localmente (entrada ignorada).
    fn apply_chapter_read(&self, entry: FfiChapterReadEntry) -> bool;
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiReadingProgressEntry {
    pub comic_name: String,
    pub chapter_sort: String,
    pub last_page: i32,
    pub is_completed: bool,
    pub updated_at: i64,
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiChapterReadEntry {
    pub comic_name: String,
    pub chapter_sort: String,
    pub created_at: i64,
}

/// Fonte/destino dos arquivos de capítulo usada pelo protocolo `acerola/sync-files/1`.
/// Implementada em Kotlin contra SAF/DocumentFile. Baseada em handles opacos (`i64`) porque
/// streams não cruzam a fronteira FFI diretamente — o Kotlin mantém um mapa de handles abertos.
#[uniffi::export(with_foreign)]
pub trait FileSyncProvider: Send + Sync {
    /// Todos os capítulos locais com checksum/tamanho, com chave natural `(comic_name, chapter)`.
    fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry>;

    /// Resumo por quadrinho (nome + contagem de capítulos), direto do Room — usado só por
    /// `acerola/browse-library/1`. Existe separado de `get_file_manifest()` de propósito: aquele
    /// método faz um `DocumentFile.exists()` via SAF por capítulo (uma transação binder cada),
    /// que só compensa quando o resultado realmente precisa apontar pra um arquivo (transferência
    /// de verdade). Pra só listar títulos isso é custo puro, e já estourou o timeout do
    /// protocolo numa biblioteca grande/recém-escaneada — ver o comentário em
    /// `FileSyncProviderImpl.getFileManifest()`.
    fn get_library_summary(&self) -> Vec<FfiComicSummaryEntry>;

    /// Abre um capítulo local para leitura em chunks (lado que envia). `-1` se não encontrado.
    fn open_chapter_for_read(&self, comic_name: String, chapter: String) -> i64;

    /// Lê até `chunk_size` bytes do handle aberto; um `Vec` vazio sinaliza fim de arquivo.
    fn read_chapter_chunk(&self, handle: i64, chunk_size: u32) -> Vec<u8>;

    /// Fecha um handle de leitura aberto por `open_chapter_for_read`.
    fn close_read_handle(&self, handle: i64);

    /// Começa a receber um capítulo (lado que recebe): cria um arquivo temporário em `synced/`.
    /// `file_name` é o nome de arquivo real anunciado pelo peer no header (schema compartilhado
    /// com o Desktop) — usado como nome final em vez do rótulo do capítulo, pra preservar a
    /// extensão original (.cbz/.cbr) do lado que enviou.
    fn begin_chapter_write(
        &self,
        comic_name: String,
        chapter: String,
        file_name: String,
        expected_checksum: String,
        size_bytes: u64,
    ) -> i64;

    /// Grava um chunk recebido no handle de escrita aberto por `begin_chapter_write`.
    fn write_chapter_chunk(&self, handle: i64, bytes: Vec<u8>) -> bool;

    /// Verifica o checksum do arquivo temporário e o move pro destino final em `synced/`.
    fn finalize_chapter_write(&self, handle: i64) -> bool;

    /// Aborta uma escrita em andamento (peer desconectou no meio, checksum não bateu, etc).
    fn abort_chapter_write(&self, handle: i64);

    /// Itens extra (capa/banner/`ComicInfo.xml`) locais, um `FfiExtraManifestEntry` por item
    /// presente — comics sem nenhum dos três simplesmente não aparecem aqui. Espelha
    /// `FileSyncService::build_manifest`'s extras do Desktop
    /// (`infra/sync/messages.rs::FileExtraInfo`).
    fn get_extras_manifest(&self) -> Vec<FfiExtraManifestEntry>;

    /// Abre um item extra local pra leitura em chunks (lado que envia). `-1` se não encontrado.
    /// `kind` é um dos `EXTRA_KIND_*` (`protocol::files::model`). Reaproveita
    /// `read_chapter_chunk`/`close_read_handle` já existentes pra ler/fechar — o handle é
    /// opaco, agnóstico a chapter vs. extra.
    fn open_extra_for_read(&self, comic_name: String, kind: String) -> i64;

    /// Começa a receber um item extra (lado que recebe). Reaproveita `write_chapter_chunk` já
    /// existente pra gravar os chunks recebidos.
    fn begin_extra_write(
        &self,
        comic_name: String,
        kind: String,
        file_name: String,
        expected_checksum: String,
        size_bytes: u64,
    ) -> i64;

    /// Verifica o checksum do arquivo temporário e o persiste no destino final: coluna
    /// `cover`/`banner` de `comic_directory` pra esses dois `kind`s, ou `ComicInfo.xml` puro
    /// (mais reprocessamento de metadata) pro terceiro.
    fn finalize_extra_write(&self, handle: i64) -> bool;

    /// Aborta uma escrita de extra em andamento.
    fn abort_extra_write(&self, handle: i64);
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiExtraManifestEntry {
    pub comic_name: String,
    /// Um dos `EXTRA_KIND_*` (`protocol::files::model`).
    pub kind: String,
    pub file_name: String,
    pub checksum: String,
    pub size_bytes: u64,
}

/// Armazenamento criptografado (Android Keystore, via `EncryptedSharedPreferences`) usado por
/// `SecureP2pStorage`/`SecureTrustedStore` (ver `storage.rs`/`trust_store.rs`) pra persistir
/// identidade do nó, cache de peers e lista de confiança TOFU sem tocar texto puro em disco.
/// Chave/valor puros — todo o layout (quais chaves existem, formato JSON de cada uma) é
/// decidido do lado Rust, o Kotlin só faz o armazenamento opaco.
#[uniffi::export(with_foreign)]
pub trait SecureBlobStore: Send + Sync {
    /// `Err` só em falha real de backend (ex.: `EncryptedSharedPreferences`/Keystore
    /// indisponível ou corrompido) — nunca usado pra "chave não existe ainda", que não é
    /// um erro. Distinguir os dois importa: se `load_identity` tratasse falha real como
    /// "identidade nunca existiu", o node mintaria uma identidade nova silenciosamente e
    /// quebraria todo o pareamento já feito sem avisar ninguém.
    fn save_blob(&self, key: String, value: Vec<u8>) -> Result<(), SecureBlobStoreError>;

    /// `Ok(None)` se a chave nunca foi salva (estado normal). `Err` só em falha real de
    /// backend — ver `save_blob`.
    fn load_blob(&self, key: String) -> Result<Option<Vec<u8>>, SecureBlobStoreError>;
}

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum SecureBlobStoreError {
    #[error("secure blob store access failed: {reason}")]
    AccessFailed { reason: String },
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiFileManifestEntry {
    pub comic_name: String,
    pub chapter: String,
    /// Nome de arquivo real (com extensão), lido do `DocumentFile` no Kotlin — necessário
    /// pra falar o mesmo schema de wire que o Desktop usa em `FileHeader`/`FileChapterInfo`.
    pub file_name: String,
    pub checksum: String,
    pub size_bytes: u64,
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiComicSummaryEntry {
    pub comic_name: String,
    pub chapter_count: u32,
    /// Reaproveita `ComicDirectory.lastModified`/`comic_directory.last_modified` — sem hash
    /// novo. O peer compara contra a versão já cacheada localmente pra decidir se precisa
    /// buscar uma capa nova via `acerola/browse-cover/1`.
    pub cover_version: i64,
}

/// Fonte/destino da capa (`cover.jpg`) de um quadrinho, usada pelo protocolo
/// `acerola/browse-cover/1`. Separada de `FileSyncProvider` de propósito: capas são pequenas o
/// bastante (thumbnail) pra trafegar como `Vec<u8>` inteiro numa chamada FFI só, sem precisar da
/// máquina de handles opacos que os capítulos (potencialmente centenas de MB) exigem.
#[uniffi::export(with_foreign)]
pub trait CoverBrowseProvider: Send + Sync {
    /// Capa local de `comic_name`, se existir. `bytes: None` cobre tanto "quadrinho não existe"
    /// quanto "existe mas não tem capa salva" — os dois casos resultam na mesma resposta
    /// `not_modified`/ausência pro peer, não há necessidade de distinguir no wire.
    fn get_local_cover(&self, comic_name: String) -> FfiCoverEntry;

    /// Grava a capa recebida de `peer_id` pra `comic_name` num cache local (nunca na árvore do
    /// usuário) e devolve o caminho/URI resultante, pra UI carregar via Coil. Chave de cache
    /// `(peer_id, comic_name, cover_version)` — quem chama decide se já tem essa versão cacheada
    /// antes de disparar a busca, isto aqui só persiste o que já foi baixado.
    fn save_remote_cover(
        &self,
        peer_id: String,
        comic_name: String,
        cover_version: i64,
        bytes: Vec<u8>,
    ) -> String;
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiCoverEntry {
    pub cover_version: i64,
    pub bytes: Option<Vec<u8>>,
}
